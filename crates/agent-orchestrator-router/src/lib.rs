//! Deterministic advisory bridge between the Agent Orchestrator and Model Router.
//!
//! This crate binds a scheduler assignment to a provider-neutral routing request. It selects a
//! model candidate but deliberately performs no model invocation, network access, credential use,
//! tool execution, filesystem mutation, engine execution, publishing, signing, or release action.

use std::{error::Error, fmt};

use agent_orchestrator::{AgentId, Assignment, TaskId};
use model_router::{
    ModelRouter, ProviderAdapter, RouterError, RoutingDecision, RoutingPolicy, RoutingRequest,
};
use serde::{Deserialize, Serialize};

/// One task/agent assignment plus the model-routing constraints required for that task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssignmentRoutingRequest {
    pub task_id: TaskId,
    pub agent_id: AgentId,
    pub routing: RoutingRequest,
}

/// Deterministic evidence that a model-routing decision was produced for an exact assignment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssignmentRoutingDecision {
    pub task_id: TaskId,
    pub agent_id: AgentId,
    pub routing_decision: RoutingDecision,
}

/// Fail-closed integration failures.
#[derive(Debug)]
pub enum IntegrationError {
    AssignmentMismatch {
        expected_task: TaskId,
        expected_agent: AgentId,
        actual_task: TaskId,
        actual_agent: AgentId,
    },
    EmptyRoutingRequestId,
    Router(RouterError),
}

impl fmt::Display for IntegrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AssignmentMismatch {
                expected_task,
                expected_agent,
                actual_task,
                actual_agent,
            } => write!(
                formatter,
                "routing request assignment '{}/{}' does not match scheduler assignment '{}/{}'",
                expected_task.0, expected_agent.0, actual_task.0, actual_agent.0
            ),
            Self::EmptyRoutingRequestId => formatter.write_str("routing request id must not be empty"),
            Self::Router(error) => write!(formatter, "model routing failed: {error}"),
        }
    }
}

impl Error for IntegrationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Router(error) => Some(error),
            Self::AssignmentMismatch { .. } | Self::EmptyRoutingRequestId => None,
        }
    }
}

impl From<RouterError> for IntegrationError {
    fn from(value: RouterError) -> Self {
        Self::Router(value)
    }
}

/// Route a model for an exact scheduler assignment without invoking that model.
///
/// # Errors
/// Returns [`IntegrationError`] when the routing request is malformed, is bound to a different
/// assignment, or the underlying Model Router rejects the routing input.
pub fn route_for_assignment(
    assignment: &Assignment,
    request: &AssignmentRoutingRequest,
    adapters: &[&dyn ProviderAdapter],
    policy: &RoutingPolicy,
) -> Result<AssignmentRoutingDecision, IntegrationError> {
    if request.task_id != assignment.task_id || request.agent_id != assignment.agent_id {
        return Err(IntegrationError::AssignmentMismatch {
            expected_task: assignment.task_id.clone(),
            expected_agent: assignment.agent_id.clone(),
            actual_task: request.task_id.clone(),
            actual_agent: request.agent_id.clone(),
        });
    }
    if request.routing.request_id.trim().is_empty() {
        return Err(IntegrationError::EmptyRoutingRequestId);
    }

    let routing_decision = ModelRouter::route(adapters, &request.routing, policy)?;
    Ok(AssignmentRoutingDecision {
        task_id: assignment.task_id.clone(),
        agent_id: assignment.agent_id.clone(),
        routing_decision,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use agent_orchestrator::{AgentId, Assignment, TaskId};
    use model_router::{
        LocalProviderAdapter, ModelCapability, ModelDescriptor, PrivacyConstraint, ProviderClass,
        RoutingOutcome, RoutingPolicy, RoutingRequest,
    };

    use super::*;

    fn assignment() -> Assignment {
        Assignment {
            task_id: TaskId("task-code".to_owned()),
            agent_id: AgentId("agent-builder".to_owned()),
        }
    }

    fn routing_request() -> RoutingRequest {
        RoutingRequest {
            request_id: "route-task-code-attempt-1".to_owned(),
            required_capabilities: BTreeSet::from([
                ModelCapability::Code,
                ModelCapability::StructuredOutput,
            ]),
            required_context_tokens: 8_000,
            required_output_tokens: 2_000,
            expected_input_tokens: 2_000,
            expected_output_tokens: 1_000,
            privacy: PrivacyConstraint::LocalOnly,
            max_estimated_cost_microusd: None,
            max_expected_latency_ms: Some(5_000),
            min_reliability_basis_points: Some(9_000),
        }
    }

    fn local_adapter() -> LocalProviderAdapter {
        LocalProviderAdapter::new(
            "local-runtime",
            vec![ModelDescriptor {
                provider_id: "local-runtime".to_owned(),
                model_id: "code-model".to_owned(),
                provider_class: ProviderClass::Local,
                capabilities: BTreeSet::from([
                    ModelCapability::Code,
                    ModelCapability::StructuredOutput,
                ]),
                context_window_tokens: 32_000,
                max_output_tokens: 8_000,
                expected_latency_ms: 400,
                input_cost_microusd_per_million_tokens: 0,
                output_cost_microusd_per_million_tokens: 0,
                reliability_basis_points: 9_500,
            }],
        )
        .expect("valid local adapter")
    }

    #[test]
    fn binds_routing_to_exact_scheduler_assignment() {
        let assignment = assignment();
        let request = AssignmentRoutingRequest {
            task_id: assignment.task_id.clone(),
            agent_id: assignment.agent_id.clone(),
            routing: routing_request(),
        };
        let adapter = local_adapter();
        let decision = route_for_assignment(
            &assignment,
            &request,
            &[&adapter],
            &RoutingPolicy::default(),
        )
        .expect("routing should succeed");

        assert_eq!(decision.task_id, assignment.task_id);
        assert_eq!(decision.agent_id, assignment.agent_id);
        assert!(matches!(
            decision.routing_decision.evidence.outcome,
            RoutingOutcome::Selected { .. }
        ));
    }

    #[test]
    fn rejects_assignment_substitution() {
        let assignment = assignment();
        let request = AssignmentRoutingRequest {
            task_id: assignment.task_id.clone(),
            agent_id: AgentId("agent-other".to_owned()),
            routing: routing_request(),
        };
        let adapter = local_adapter();
        let error = route_for_assignment(
            &assignment,
            &request,
            &[&adapter],
            &RoutingPolicy::default(),
        )
        .expect_err("assignment substitution must fail closed");

        assert!(matches!(error, IntegrationError::AssignmentMismatch { .. }));
    }

    #[test]
    fn preserves_router_abstention_as_safe_outcome() {
        let assignment = assignment();
        let mut routing = routing_request();
        routing.required_capabilities.insert(ModelCapability::Vision);
        let request = AssignmentRoutingRequest {
            task_id: assignment.task_id.clone(),
            agent_id: assignment.agent_id.clone(),
            routing,
        };
        let adapter = local_adapter();
        let decision = route_for_assignment(
            &assignment,
            &request,
            &[&adapter],
            &RoutingPolicy::default(),
        )
        .expect("abstention is a valid routing decision");

        assert!(matches!(
            decision.routing_decision.evidence.outcome,
            RoutingOutcome::Abstained { .. }
        ));
    }
}
