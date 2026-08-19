//! Deterministic agent/task orchestration for NTPX `GameForge` Ω∞.
//!
//! The orchestrator owns workflow state and evidence. It does not invoke models, execute tools,
//! open network connections, use credentials, mutate repositories, or bypass authorization.
//! Model selection is delegated to the verified `ModelRouter`; governed capabilities are submitted
//! to the verified `GovernanceRuntime` as exact predeclared requests.

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

use autonomy_governor::{
    ApprovalContext, AuthorizationPermit, GovernanceRuntime, PrincipalId, Request, RuntimeError,
};
use model_router::{
    ModelRouter, ProviderAdapter, RouterError, RoutingDecision, RoutingOutcome, RoutingPolicy,
    RoutingRequest,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Immutable contract for one orchestrated agent identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentContract {
    pub id: String,
    pub principal: PrincipalId,
    pub role: String,
    pub skills: BTreeSet<String>,
    pub max_parallel_tasks: usize,
}

/// Immutable contract for one unit of orchestrated work.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskContract {
    pub id: String,
    pub objective: String,
    pub dependencies: BTreeSet<String>,
    pub required_skills: BTreeSet<String>,
    pub assigned_agent: String,
    pub routing_request: RoutingRequest,
    pub routing_policy: RoutingPolicy,
    pub max_attempts: u16,
    pub governed_requests: Vec<Request>,
}

/// Persistent workflow state for one task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    Pending,
    Running,
    RetryScheduled,
    Succeeded,
    Failed,
    Escalated,
    Blocked,
}

impl TaskState {
    const fn is_failure_terminal(self) -> bool {
        matches!(self, Self::Failed | Self::Escalated | Self::Blocked)
    }
}

/// Result reported for a task attempt that was previously dispatched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttemptResult {
    Succeeded,
    RetryableFailure,
    PermanentFailure,
}

/// Evidence event emitted by the orchestrator state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceEvent {
    Dispatched,
    RoutingAbstained,
    Succeeded,
    RetryScheduled,
    Failed,
    Escalated,
    Blocked,
    GovernedActionAuthorized,
}

/// Immutable evidence entry attached to one task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskEvidence {
    pub task_id: String,
    pub attempt: u16,
    pub event: EvidenceEvent,
    pub routing_evidence_sha256: Option<String>,
    pub message: String,
}

/// Deterministic dispatch outcome. Selection still performs no model invocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "dispatch", rename_all = "snake_case")]
pub enum DispatchOutcome {
    Selected {
        task_id: String,
        agent_id: String,
        attempt: u16,
        routing: RoutingDecision,
    },
    Escalated {
        task_id: String,
        agent_id: String,
        routing: RoutingDecision,
        reason: String,
    },
}

/// Validation, routing, state-transition, or authorization failure.
#[derive(Debug)]
pub enum OrchestratorError {
    DuplicateAgentId(String),
    DuplicateTaskId(String),
    InvalidAgent {
        agent_id: String,
        reason: String,
    },
    InvalidTask {
        task_id: String,
        reason: String,
    },
    UnknownAssignedAgent {
        task_id: String,
        agent_id: String,
    },
    UnknownDependency {
        task_id: String,
        dependency_id: String,
    },
    DependencyCycle,
    AgentSkillMismatch {
        task_id: String,
        agent_id: String,
        skill: String,
    },
    GovernedPrincipalMismatch {
        task_id: String,
        agent_id: String,
    },
    UnknownTask(String),
    TaskNotRunning(String),
    GovernedRequestNotDeclared(String),
    Encode(serde_json::Error),
    Router(RouterError),
    Runtime(RuntimeError),
}

impl fmt::Display for OrchestratorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateAgentId(id) => write!(formatter, "duplicate agent id '{id}'"),
            Self::DuplicateTaskId(id) => write!(formatter, "duplicate task id '{id}'"),
            Self::InvalidAgent { agent_id, reason } => {
                write!(formatter, "invalid agent '{agent_id}': {reason}")
            }
            Self::InvalidTask { task_id, reason } => {
                write!(formatter, "invalid task '{task_id}': {reason}")
            }
            Self::UnknownAssignedAgent { task_id, agent_id } => write!(
                formatter,
                "task '{task_id}' references unknown assigned agent '{agent_id}'"
            ),
            Self::UnknownDependency {
                task_id,
                dependency_id,
            } => write!(
                formatter,
                "task '{task_id}' references unknown dependency '{dependency_id}'"
            ),
            Self::DependencyCycle => write!(formatter, "task dependency graph contains a cycle"),
            Self::AgentSkillMismatch {
                task_id,
                agent_id,
                skill,
            } => write!(
                formatter,
                "task '{task_id}' requires skill '{skill}' not declared by agent '{agent_id}'"
            ),
            Self::GovernedPrincipalMismatch { task_id, agent_id } => write!(
                formatter,
                "task '{task_id}' declares a governed request for a principal other than agent '{agent_id}'"
            ),
            Self::UnknownTask(id) => write!(formatter, "unknown task '{id}'"),
            Self::TaskNotRunning(id) => write!(formatter, "task '{id}' is not running"),
            Self::GovernedRequestNotDeclared(id) => write!(
                formatter,
                "task '{id}' attempted a governed request not declared in its immutable contract"
            ),
            Self::Encode(error) => write!(formatter, "orchestrator evidence encoding failed: {error}"),
            Self::Router(error) => write!(formatter, "model routing failed: {error}"),
            Self::Runtime(error) => write!(formatter, "governed authorization failed: {error}"),
        }
    }
}

impl Error for OrchestratorError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Encode(error) => Some(error),
            Self::Router(error) => Some(error),
            Self::Runtime(error) => Some(error),
            Self::DuplicateAgentId(_)
            | Self::DuplicateTaskId(_)
            | Self::InvalidAgent { .. }
            | Self::InvalidTask { .. }
            | Self::UnknownAssignedAgent { .. }
            | Self::UnknownDependency { .. }
            | Self::DependencyCycle
            | Self::AgentSkillMismatch { .. }
            | Self::GovernedPrincipalMismatch { .. }
            | Self::UnknownTask(_)
            | Self::TaskNotRunning(_)
            | Self::GovernedRequestNotDeclared(_) => None,
        }
    }
}

impl From<RouterError> for OrchestratorError {
    fn from(error: RouterError) -> Self {
        Self::Router(error)
    }
}

impl From<RuntimeError> for OrchestratorError {
    fn from(error: RuntimeError) -> Self {
        Self::Runtime(error)
    }
}

impl From<serde_json::Error> for OrchestratorError {
    fn from(error: serde_json::Error) -> Self {
        Self::Encode(error)
    }
}

/// Validated immutable project orchestration plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskGraph {
    agents: BTreeMap<String, AgentContract>,
    tasks: BTreeMap<String, TaskContract>,
    plan_sha256: String,
}

impl TaskGraph {
    /// Validate and canonicalize agent/task contracts into a deterministic dependency graph.
    ///
    /// Input enumeration order does not affect the resulting map order, ready-task order, or plan
    /// digest. Task dependency cycles, dangling references, skill mismatches, and principal drift
    /// fail closed.
    ///
    /// # Errors
    /// Returns [`OrchestratorError`] when a contract is malformed or the graph is invalid.
    pub fn new(
        agents: Vec<AgentContract>,
        tasks: Vec<TaskContract>,
    ) -> Result<Self, OrchestratorError> {
        let agents = collect_agents(agents)?;
        let tasks = collect_tasks(tasks)?;
        validate_task_bindings(&agents, &tasks)?;
        validate_acyclic(&tasks)?;
        let plan_sha256 = plan_digest(&agents, &tasks)?;
        Ok(Self {
            agents,
            tasks,
            plan_sha256,
        })
    }

    /// Return the stable SHA-256 digest of canonical agent/task contracts.
    #[must_use]
    pub fn plan_sha256(&self) -> &str {
        &self.plan_sha256
    }

    /// Return one validated task contract.
    #[must_use]
    pub fn task(&self, task_id: &str) -> Option<&TaskContract> {
        self.tasks.get(task_id)
    }

    /// Return one validated agent contract.
    #[must_use]
    pub fn agent(&self, agent_id: &str) -> Option<&AgentContract> {
        self.agents.get(agent_id)
    }

    /// Return validated tasks in stable identifier order.
    pub fn tasks(&self) -> impl Iterator<Item = &TaskContract> {
        self.tasks.values()
    }
}

/// Deterministic in-memory workflow state machine for one validated [`TaskGraph`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentOrchestrator {
    graph: TaskGraph,
    states: BTreeMap<String, TaskState>,
    attempts: BTreeMap<String, u16>,
    evidence: BTreeMap<String, Vec<TaskEvidence>>,
}

impl AgentOrchestrator {
    /// Start orchestration with every task pending and no execution authority.
    #[must_use]
    pub fn new(graph: TaskGraph) -> Self {
        let states = graph
            .tasks
            .keys()
            .map(|task_id| (task_id.clone(), TaskState::Pending))
            .collect();
        let attempts = graph
            .tasks
            .keys()
            .map(|task_id| (task_id.clone(), 0_u16))
            .collect();
        Self {
            graph,
            states,
            attempts,
            evidence: BTreeMap::new(),
        }
    }

    /// Return the immutable validated graph.
    #[must_use]
    pub fn graph(&self) -> &TaskGraph {
        &self.graph
    }

    /// Return the current state of one task.
    #[must_use]
    pub fn state(&self, task_id: &str) -> Option<TaskState> {
        self.states.get(task_id).copied()
    }

    /// Return evidence for one task in append order.
    #[must_use]
    pub fn evidence(&self, task_id: &str) -> &[TaskEvidence] {
        self.evidence.get(task_id).map_or(&[], Vec::as_slice)
    }

    /// Return task identifiers eligible for dispatch in deterministic order.
    #[must_use]
    pub fn ready_task_ids(&self) -> Vec<String> {
        self.graph
            .tasks
            .iter()
            .filter(|(task_id, task)| self.is_ready(task_id, task))
            .map(|(task_id, _)| task_id.clone())
            .collect()
    }

    /// Mark tasks blocked when any dependency has reached a failure-terminal state.
    pub fn reconcile_blocked(&mut self) {
        loop {
            let blocked = self
                .graph
                .tasks
                .iter()
                .filter_map(|(task_id, task)| {
                    let state = self.states.get(task_id).copied()?;
                    if !matches!(state, TaskState::Pending | TaskState::RetryScheduled) {
                        return None;
                    }
                    task.dependencies
                        .iter()
                        .find(|dependency| {
                            self.states
                                .get(*dependency)
                                .copied()
                                .is_some_and(TaskState::is_failure_terminal)
                        })
                        .map(|dependency| (task_id.clone(), dependency.clone()))
                })
                .collect::<Vec<_>>();
            if blocked.is_empty() {
                return;
            }
            for (task_id, dependency) in blocked {
                self.states.insert(task_id.clone(), TaskState::Blocked);
                let attempt = self.attempts.get(&task_id).copied().unwrap_or_default();
                self.push_evidence(TaskEvidence {
                    task_id,
                    attempt,
                    event: EvidenceEvent::Blocked,
                    routing_evidence_sha256: None,
                    message: format!("dependency '{dependency}' reached a terminal failure state"),
                });
            }
        }
    }

    /// Select the next schedulable task and ask the verified `ModelRouter` for a model decision.
    ///
    /// This method performs no model invocation. A selected model becomes dispatch evidence only.
    /// If routing abstains, the task escalates without consuming an execution attempt.
    ///
    /// # Errors
    /// Returns [`OrchestratorError`] if routing fails structurally or the selected task cannot be
    /// resolved from the validated graph.
    pub fn dispatch_next(
        &mut self,
        adapters: &[&dyn ProviderAdapter],
    ) -> Result<Option<DispatchOutcome>, OrchestratorError> {
        self.reconcile_blocked();
        let Some(task_id) = self.ready_task_ids().into_iter().next() else {
            return Ok(None);
        };
        let task = self
            .graph
            .tasks
            .get(&task_id)
            .ok_or_else(|| OrchestratorError::UnknownTask(task_id.clone()))?
            .clone();
        let routing = ModelRouter::route(adapters, &task.routing_request, &task.routing_policy)?;
        match &routing.evidence.outcome {
            RoutingOutcome::Selected { .. } => {
                let attempt = self.next_attempt(&task_id);
                self.states.insert(task_id.clone(), TaskState::Running);
                self.push_evidence(TaskEvidence {
                    task_id: task_id.clone(),
                    attempt,
                    event: EvidenceEvent::Dispatched,
                    routing_evidence_sha256: Some(routing.evidence_sha256.clone()),
                    message: "task selected for execution by its assigned agent".to_owned(),
                });
                Ok(Some(DispatchOutcome::Selected {
                    task_id,
                    agent_id: task.assigned_agent,
                    attempt,
                    routing,
                }))
            }
            RoutingOutcome::Abstained { reason } => {
                let reason = reason.clone();
                self.states.insert(task_id.clone(), TaskState::Escalated);
                let attempt = self.attempts.get(&task_id).copied().unwrap_or_default();
                self.push_evidence(TaskEvidence {
                    task_id: task_id.clone(),
                    attempt,
                    event: EvidenceEvent::RoutingAbstained,
                    routing_evidence_sha256: Some(routing.evidence_sha256.clone()),
                    message: reason.clone(),
                });
                Ok(Some(DispatchOutcome::Escalated {
                    task_id,
                    agent_id: task.assigned_agent,
                    routing,
                    reason,
                }))
            }
        }
    }

    /// Record the outcome of a running task attempt and apply bounded retry/escalation rules.
    ///
    /// # Errors
    /// Returns [`OrchestratorError`] for an unknown task or a task not currently running.
    pub fn record_attempt_result(
        &mut self,
        task_id: &str,
        result: AttemptResult,
        message: impl Into<String>,
    ) -> Result<TaskState, OrchestratorError> {
        let task = self
            .graph
            .tasks
            .get(task_id)
            .ok_or_else(|| OrchestratorError::UnknownTask(task_id.to_owned()))?;
        if self.state(task_id) != Some(TaskState::Running) {
            return Err(OrchestratorError::TaskNotRunning(task_id.to_owned()));
        }
        let attempt = self.attempts.get(task_id).copied().unwrap_or_default();
        let (state, event) = match result {
            AttemptResult::Succeeded => (TaskState::Succeeded, EvidenceEvent::Succeeded),
            AttemptResult::PermanentFailure => (TaskState::Failed, EvidenceEvent::Failed),
            AttemptResult::RetryableFailure if attempt < task.max_attempts => {
                (TaskState::RetryScheduled, EvidenceEvent::RetryScheduled)
            }
            AttemptResult::RetryableFailure => (TaskState::Escalated, EvidenceEvent::Escalated),
        };
        self.states.insert(task_id.to_owned(), state);
        self.push_evidence(TaskEvidence {
            task_id: task_id.to_owned(),
            attempt,
            event,
            routing_evidence_sha256: None,
            message: message.into(),
        });
        self.reconcile_blocked();
        Ok(state)
    }

    /// Submit one exact predeclared governed request through the verified `GovernanceRuntime`.
    ///
    /// The orchestrator performs no protected action itself. Authorization succeeds only when the
    /// task is running, the request exactly matches its immutable contract, and the Governor grants
    /// a permit (including any required human approval).
    ///
    /// # Errors
    /// Returns [`OrchestratorError`] for invalid task state, undeclared requests, or Governor
    /// denial/approval/audit failures.
    pub fn authorize_governed_request(
        &mut self,
        runtime: &GovernanceRuntime<'_>,
        task_id: &str,
        request: &Request,
        approval: Option<ApprovalContext<'_>>,
    ) -> Result<AuthorizationPermit, OrchestratorError> {
        let task = self
            .graph
            .tasks
            .get(task_id)
            .ok_or_else(|| OrchestratorError::UnknownTask(task_id.to_owned()))?;
        if self.state(task_id) != Some(TaskState::Running) {
            return Err(OrchestratorError::TaskNotRunning(task_id.to_owned()));
        }
        if !task
            .governed_requests
            .iter()
            .any(|declared| declared == request)
        {
            return Err(OrchestratorError::GovernedRequestNotDeclared(
                task_id.to_owned(),
            ));
        }
        let permit = runtime.authorize(request.clone(), approval)?;
        let attempt = self.attempts.get(task_id).copied().unwrap_or_default();
        self.push_evidence(TaskEvidence {
            task_id: task_id.to_owned(),
            attempt,
            event: EvidenceEvent::GovernedActionAuthorized,
            routing_evidence_sha256: None,
            message: "exact predeclared governed request authorized by Autonomy Governor"
                .to_owned(),
        });
        Ok(permit)
    }

    fn is_ready(&self, task_id: &str, task: &TaskContract) -> bool {
        let state = self.states.get(task_id).copied();
        if !matches!(state, Some(TaskState::Pending | TaskState::RetryScheduled)) {
            return false;
        }
        if !task
            .dependencies
            .iter()
            .all(|dependency| self.state(dependency) == Some(TaskState::Succeeded))
        {
            return false;
        }
        let running = self
            .graph
            .tasks
            .iter()
            .filter(|(other_id, other)| {
                other.assigned_agent == task.assigned_agent
                    && self.state(other_id) == Some(TaskState::Running)
            })
            .count();
        self.graph
            .agents
            .get(&task.assigned_agent)
            .is_some_and(|agent| running < agent.max_parallel_tasks)
    }

    fn next_attempt(&mut self, task_id: &str) -> u16 {
        let attempt = self.attempts.entry(task_id.to_owned()).or_default();
        *attempt = attempt.saturating_add(1);
        *attempt
    }

    fn push_evidence(&mut self, evidence: TaskEvidence) {
        self.evidence
            .entry(evidence.task_id.clone())
            .or_default()
            .push(evidence);
    }
}

fn collect_agents(
    agents: Vec<AgentContract>,
) -> Result<BTreeMap<String, AgentContract>, OrchestratorError> {
    let mut collected = BTreeMap::new();
    for agent in agents {
        validate_agent(&agent)?;
        let id = agent.id.clone();
        if collected.insert(id.clone(), agent).is_some() {
            return Err(OrchestratorError::DuplicateAgentId(id));
        }
    }
    Ok(collected)
}

fn collect_tasks(
    tasks: Vec<TaskContract>,
) -> Result<BTreeMap<String, TaskContract>, OrchestratorError> {
    let mut collected = BTreeMap::new();
    for task in tasks {
        validate_task(&task)?;
        let id = task.id.clone();
        if collected.insert(id.clone(), task).is_some() {
            return Err(OrchestratorError::DuplicateTaskId(id));
        }
    }
    Ok(collected)
}

fn validate_agent(agent: &AgentContract) -> Result<(), OrchestratorError> {
    if agent.id.trim().is_empty() {
        return Err(OrchestratorError::InvalidAgent {
            agent_id: agent.id.clone(),
            reason: "identifier must not be blank".to_owned(),
        });
    }
    if agent.role.trim().is_empty() {
        return Err(OrchestratorError::InvalidAgent {
            agent_id: agent.id.clone(),
            reason: "role must not be blank".to_owned(),
        });
    }
    if agent.principal.0 != agent.id {
        return Err(OrchestratorError::InvalidAgent {
            agent_id: agent.id.clone(),
            reason: "governor principal id must equal agent id".to_owned(),
        });
    }
    if agent.max_parallel_tasks == 0 {
        return Err(OrchestratorError::InvalidAgent {
            agent_id: agent.id.clone(),
            reason: "max_parallel_tasks must be at least one".to_owned(),
        });
    }
    if agent.skills.iter().any(|skill| skill.trim().is_empty()) {
        return Err(OrchestratorError::InvalidAgent {
            agent_id: agent.id.clone(),
            reason: "skills must not contain blank values".to_owned(),
        });
    }
    Ok(())
}

fn validate_task(task: &TaskContract) -> Result<(), OrchestratorError> {
    if task.id.trim().is_empty() {
        return Err(invalid_task(task, "identifier must not be blank"));
    }
    if task.objective.trim().is_empty() {
        return Err(invalid_task(task, "objective must not be blank"));
    }
    if task.assigned_agent.trim().is_empty() {
        return Err(invalid_task(task, "assigned_agent must not be blank"));
    }
    if task.routing_request.request_id != task.id {
        return Err(invalid_task(task, "routing request id must equal task id"));
    }
    if task.max_attempts == 0 {
        return Err(invalid_task(task, "max_attempts must be at least one"));
    }
    if task.dependencies.contains(&task.id) {
        return Err(invalid_task(task, "task must not depend on itself"));
    }
    if task
        .required_skills
        .iter()
        .any(|skill| skill.trim().is_empty())
    {
        return Err(invalid_task(
            task,
            "required_skills must not contain blank values",
        ));
    }
    Ok(())
}

fn invalid_task(task: &TaskContract, reason: &str) -> OrchestratorError {
    OrchestratorError::InvalidTask {
        task_id: task.id.clone(),
        reason: reason.to_owned(),
    }
}

fn validate_task_bindings(
    agents: &BTreeMap<String, AgentContract>,
    tasks: &BTreeMap<String, TaskContract>,
) -> Result<(), OrchestratorError> {
    for task in tasks.values() {
        let agent = agents.get(&task.assigned_agent).ok_or_else(|| {
            OrchestratorError::UnknownAssignedAgent {
                task_id: task.id.clone(),
                agent_id: task.assigned_agent.clone(),
            }
        })?;
        for dependency in &task.dependencies {
            if !tasks.contains_key(dependency) {
                return Err(OrchestratorError::UnknownDependency {
                    task_id: task.id.clone(),
                    dependency_id: dependency.clone(),
                });
            }
        }
        for skill in &task.required_skills {
            if !agent.skills.contains(skill) {
                return Err(OrchestratorError::AgentSkillMismatch {
                    task_id: task.id.clone(),
                    agent_id: agent.id.clone(),
                    skill: skill.clone(),
                });
            }
        }
        if task
            .governed_requests
            .iter()
            .any(|request| request.principal != agent.principal)
        {
            return Err(OrchestratorError::GovernedPrincipalMismatch {
                task_id: task.id.clone(),
                agent_id: agent.id.clone(),
            });
        }
    }
    Ok(())
}

fn validate_acyclic(tasks: &BTreeMap<String, TaskContract>) -> Result<(), OrchestratorError> {
    let mut remaining = tasks
        .iter()
        .map(|(task_id, task)| (task_id.clone(), task.dependencies.len()))
        .collect::<BTreeMap<_, _>>();
    let mut ready = remaining
        .iter()
        .filter_map(|(task_id, count)| (*count == 0).then_some(task_id.clone()))
        .collect::<BTreeSet<_>>();
    let mut visited = 0_usize;

    while let Some(task_id) = ready.pop_first() {
        visited += 1;
        for (candidate_id, task) in tasks {
            if !task.dependencies.contains(&task_id) {
                continue;
            }
            let count = remaining
                .get_mut(candidate_id)
                .expect("validated task id must exist in remaining dependency map");
            *count -= 1;
            if *count == 0 {
                ready.insert(candidate_id.clone());
            }
        }
    }

    if visited == tasks.len() {
        Ok(())
    } else {
        Err(OrchestratorError::DependencyCycle)
    }
}

fn plan_digest(
    agents: &BTreeMap<String, AgentContract>,
    tasks: &BTreeMap<String, TaskContract>,
) -> Result<String, OrchestratorError> {
    #[derive(Serialize)]
    struct CanonicalPlan<'a> {
        agents: Vec<&'a AgentContract>,
        tasks: Vec<&'a TaskContract>,
    }

    let encoded = serde_json::to_vec(&CanonicalPlan {
        agents: agents.values().collect(),
        tasks: tasks.values().collect(),
    })?;
    Ok(to_lower_hex(&Sha256::digest(encoded)))
}

fn to_lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}
