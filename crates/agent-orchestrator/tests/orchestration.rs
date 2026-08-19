use std::{
    collections::BTreeSet,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

use agent_orchestrator::{
    AgentContract, AgentOrchestrator, AttemptResult, DispatchOutcome, EvidenceEvent,
    OrchestratorError, TaskContract, TaskGraph, TaskState,
};
use autonomy_governor::{
    ApprovalRequirement, AuditLog, GovernanceRuntime, Grant, Permission, Policy, Principal,
    PrincipalId, Request, RiskLevel, Scope, ScopeKind,
};
use model_router::{
    LocalProviderAdapter, ModelCapability, ModelDescriptor, PrivacyConstraint, ProviderClass,
    RoutingPolicy, RoutingRequest,
};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn strings(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn capabilities(values: &[ModelCapability]) -> BTreeSet<ModelCapability> {
    values.iter().copied().collect()
}

fn agent(id: &str, max_parallel_tasks: usize) -> AgentContract {
    AgentContract {
        id: id.to_owned(),
        principal: PrincipalId(id.to_owned()),
        role: "specialist".to_owned(),
        skills: strings(&["code", "review"]),
        max_parallel_tasks,
    }
}

fn routing(task_id: &str) -> RoutingRequest {
    RoutingRequest {
        request_id: task_id.to_owned(),
        required_capabilities: capabilities(&[
            ModelCapability::Text,
            ModelCapability::Code,
            ModelCapability::StructuredOutput,
        ]),
        required_context_tokens: 8_000,
        required_output_tokens: 2_000,
        expected_input_tokens: 2_000,
        expected_output_tokens: 1_000,
        privacy: PrivacyConstraint::LocalOnly,
        max_estimated_cost_microusd: None,
        max_expected_latency_ms: None,
        min_reliability_basis_points: Some(9_000),
    }
}

fn task(
    id: &str,
    dependencies: &[&str],
    assigned_agent: &str,
    max_attempts: u16,
) -> TaskContract {
    TaskContract {
        id: id.to_owned(),
        objective: format!("complete {id}"),
        dependencies: strings(dependencies),
        required_skills: strings(&["code"]),
        assigned_agent: assigned_agent.to_owned(),
        routing_request: routing(id),
        routing_policy: RoutingPolicy::default(),
        max_attempts,
        governed_requests: Vec::new(),
    }
}

fn local_adapter() -> LocalProviderAdapter {
    LocalProviderAdapter::new(
        "local-runtime",
        vec![ModelDescriptor {
            provider_id: "local-runtime".to_owned(),
            model_id: "coder".to_owned(),
            provider_class: ProviderClass::Local,
            capabilities: capabilities(&[
                ModelCapability::Text,
                ModelCapability::Code,
                ModelCapability::StructuredOutput,
            ]),
            context_window_tokens: 32_000,
            max_output_tokens: 8_000,
            expected_latency_ms: 100,
            input_cost_microusd_per_million_tokens: 0,
            output_cost_microusd_per_million_tokens: 0,
            reliability_basis_points: 9_900,
        }],
    )
    .expect("valid local adapter")
}

#[test]
fn plan_and_ready_order_are_deterministic_across_input_order() {
    let first = TaskGraph::new(
        vec![agent("agent-b", 1), agent("agent-a", 1)],
        vec![task("task-b", &[], "agent-b", 2), task("task-a", &[], "agent-a", 2)],
    )
    .expect("valid graph");
    let second = TaskGraph::new(
        vec![agent("agent-a", 1), agent("agent-b", 1)],
        vec![task("task-a", &[], "agent-a", 2), task("task-b", &[], "agent-b", 2)],
    )
    .expect("valid graph");

    assert_eq!(first.plan_sha256(), second.plan_sha256());
    assert_eq!(
        AgentOrchestrator::new(first).ready_task_ids(),
        vec!["task-a".to_owned(), "task-b".to_owned()]
    );
}

#[test]
fn graph_rejects_cycles_unknown_dependencies_and_skill_drift() {
    let cycle = TaskGraph::new(
        vec![agent("agent", 1)],
        vec![
            task("a", &["b"], "agent", 1),
            task("b", &["a"], "agent", 1),
        ],
    );
    assert!(matches!(cycle, Err(OrchestratorError::DependencyCycle)));

    let missing = TaskGraph::new(
        vec![agent("agent", 1)],
        vec![task("a", &["missing"], "agent", 1)],
    );
    assert!(matches!(
        missing,
        Err(OrchestratorError::UnknownDependency { .. })
    ));

    let mut unsupported = task("a", &[], "agent", 1);
    unsupported.required_skills = strings(&["animation"]);
    let mismatch = TaskGraph::new(vec![agent("agent", 1)], vec![unsupported]);
    assert!(matches!(
        mismatch,
        Err(OrchestratorError::AgentSkillMismatch { .. })
    ));
}

#[test]
fn dispatch_uses_model_router_and_respects_agent_parallelism() {
    let graph = TaskGraph::new(
        vec![agent("agent", 1)],
        vec![
            task("a", &[], "agent", 2),
            task("b", &[], "agent", 2),
        ],
    )
    .expect("valid graph");
    let mut orchestrator = AgentOrchestrator::new(graph);
    let adapter = local_adapter();

    let first = orchestrator
        .dispatch_next(&[&adapter])
        .expect("routing should succeed")
        .expect("one task should dispatch");
    assert!(matches!(
        first,
        DispatchOutcome::Selected {
            ref task_id,
            attempt: 1,
            ..
        } if task_id == "a"
    ));
    assert_eq!(orchestrator.state("a"), Some(TaskState::Running));
    assert!(orchestrator.ready_task_ids().is_empty());

    orchestrator
        .record_attempt_result("a", AttemptResult::Succeeded, "done")
        .expect("running task should complete");
    assert_eq!(orchestrator.ready_task_ids(), vec!["b".to_owned()]);
    assert_eq!(
        orchestrator.evidence("a")[0].event,
        EvidenceEvent::Dispatched
    );
}

#[test]
fn bounded_retry_escalates_after_final_allowed_attempt() {
    let graph = TaskGraph::new(
        vec![agent("agent", 1)],
        vec![task("task", &[], "agent", 2)],
    )
    .expect("valid graph");
    let mut orchestrator = AgentOrchestrator::new(graph);
    let adapter = local_adapter();

    orchestrator
        .dispatch_next(&[&adapter])
        .expect("first dispatch")
        .expect("selected task");
    assert_eq!(
        orchestrator
            .record_attempt_result("task", AttemptResult::RetryableFailure, "retry")
            .expect("first failure accepted"),
        TaskState::RetryScheduled
    );

    orchestrator
        .dispatch_next(&[&adapter])
        .expect("second dispatch")
        .expect("selected retry");
    assert_eq!(
        orchestrator
            .record_attempt_result("task", AttemptResult::RetryableFailure, "exhausted")
            .expect("second failure accepted"),
        TaskState::Escalated
    );
    assert_eq!(orchestrator.state("task"), Some(TaskState::Escalated));
    assert!(orchestrator
        .evidence("task")
        .iter()
        .any(|entry| entry.event == EvidenceEvent::Escalated));
}

#[test]
fn routing_abstention_escalates_without_consuming_attempt() {
    let graph = TaskGraph::new(
        vec![agent("agent", 1)],
        vec![task("task", &[], "agent", 2)],
    )
    .expect("valid graph");
    let mut orchestrator = AgentOrchestrator::new(graph);

    let outcome = orchestrator
        .dispatch_next(&[])
        .expect("structural routing should succeed")
        .expect("task should be evaluated");
    assert!(matches!(
        outcome,
        DispatchOutcome::Escalated { ref task_id, .. } if task_id == "task"
    ));
    assert_eq!(orchestrator.state("task"), Some(TaskState::Escalated));
    assert_eq!(orchestrator.evidence("task")[0].attempt, 0);
}

#[test]
fn failed_dependency_blocks_downstream_work() {
    let graph = TaskGraph::new(
        vec![agent("agent", 1)],
        vec![
            task("a", &[], "agent", 1),
            task("b", &["a"], "agent", 1),
        ],
    )
    .expect("valid graph");
    let mut orchestrator = AgentOrchestrator::new(graph);
    let adapter = local_adapter();

    orchestrator
        .dispatch_next(&[&adapter])
        .expect("dispatch")
        .expect("selected task");
    orchestrator
        .record_attempt_result("a", AttemptResult::PermanentFailure, "fatal")
        .expect("failure should be accepted");

    assert_eq!(orchestrator.state("a"), Some(TaskState::Failed));
    assert_eq!(orchestrator.state("b"), Some(TaskState::Blocked));
}

#[test]
fn governed_actions_require_exact_predeclared_request_and_governor_permit() {
    let governed = Request {
        principal: PrincipalId("agent".to_owned()),
        permission: Permission::Read,
        scope: Scope {
            kind: ScopeKind::Tool,
            resource: "read-only-status".to_owned(),
        },
        risk: RiskLevel::Low,
    };
    let mut governed_task = task("task", &[], "agent", 1);
    governed_task.governed_requests.push(governed.clone());
    let graph = TaskGraph::new(vec![agent("agent", 1)], vec![governed_task])
        .expect("valid governed graph");
    let mut orchestrator = AgentOrchestrator::new(graph);
    let adapter = local_adapter();
    orchestrator
        .dispatch_next(&[&adapter])
        .expect("dispatch")
        .expect("selected task");

    let policy = Policy {
        principals: vec![Principal {
            id: PrincipalId("agent".to_owned()),
            role: "specialist".to_owned(),
        }],
        grants: vec![Grant {
            principal: PrincipalId("agent".to_owned()),
            permission: Permission::Read,
            scope: governed.scope.clone(),
            approval: ApprovalRequirement::None,
        }],
    };
    let temp = temp_path("orchestrator-audit.jsonl");
    let audit = AuditLog::new(&temp);
    let runtime = GovernanceRuntime::new(&policy, &audit);

    let permit = orchestrator
        .authorize_governed_request(&runtime, "task", &governed, None)
        .expect("predeclared request should be authorized by governor");
    assert_eq!(permit.request, governed);

    let undeclared = Request {
        principal: PrincipalId("agent".to_owned()),
        permission: Permission::Execute,
        scope: Scope {
            kind: ScopeKind::Tool,
            resource: "read-only-status".to_owned(),
        },
        risk: RiskLevel::Low,
    };
    assert!(matches!(
        orchestrator.authorize_governed_request(&runtime, "task", &undeclared, None),
        Err(OrchestratorError::GovernedRequestNotDeclared(_))
    ));

    let _ = fs::remove_file(&temp);
    let _ = fs::remove_file(audit.lock_path());
}

fn temp_path(name: &str) -> PathBuf {
    let sequence = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "ntpx-phase05-{}-{sequence}-{name}",
        std::process::id()
    ))
}
