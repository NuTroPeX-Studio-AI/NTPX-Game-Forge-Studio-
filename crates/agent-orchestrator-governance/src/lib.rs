//! Assignment-bound integration between the Agent Orchestrator and Autonomy Governor.
//!
//! This crate converts an exact scheduler assignment plus a declared governed-action intent into
//! an Autonomy Governor request. It evaluates authorization only; it does not execute the action.

use std::{error::Error, fmt};

use agent_orchestrator::{AgentId, Assignment, TaskId};
use autonomy_governor::{
    DecisionRecord, Permission, Policy, PrincipalId, Request, RiskLevel, Scope,
};
use serde::{Deserialize, Serialize};

/// Governed action requested on behalf of one exact scheduler assignment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernedActionIntent {
    pub task_id: TaskId,
    pub agent_id: AgentId,
    pub permission: Permission,
    pub scope: Scope,
    pub risk: RiskLevel,
}

/// Authorization evidence bound back to the assignment that requested it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernedAssignmentDecision {
    pub task_id: TaskId,
    pub agent_id: AgentId,
    pub decision: DecisionRecord,
}

/// Fail-closed integration failures before governor evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegrationError {
    AssignmentMismatch {
        expected_task: TaskId,
        expected_agent: AgentId,
        actual_task: TaskId,
        actual_agent: AgentId,
    },
    EmptyAgentId,
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
                "governed action assignment '{}/{}' does not match scheduler assignment '{}/{}'",
                actual_task.0, actual_agent.0, expected_task.0, expected_agent.0
            ),
            Self::EmptyAgentId => formatter.write_str("scheduler agent id must not be empty"),
        }
    }
}

impl Error for IntegrationError {}

/// Evaluate the governed action for an exact assignment using the verified deny-by-default policy.
///
/// The scheduler agent ID becomes the governor principal ID; therefore grants remain explicit and
/// exact. Protected actions continue to require human approval according to the governor policy.
///
/// # Errors
/// Returns [`IntegrationError`] if caller-supplied intent attempts to substitute a task or agent.
pub fn evaluate_assignment_action(
    policy: &Policy,
    assignment: &Assignment,
    intent: &GovernedActionIntent,
) -> Result<GovernedAssignmentDecision, IntegrationError> {
    if intent.task_id != assignment.task_id || intent.agent_id != assignment.agent_id {
        return Err(IntegrationError::AssignmentMismatch {
            expected_task: assignment.task_id.clone(),
            expected_agent: assignment.agent_id.clone(),
            actual_task: intent.task_id.clone(),
            actual_agent: intent.agent_id.clone(),
        });
    }
    if assignment.agent_id.0.trim().is_empty() {
        return Err(IntegrationError::EmptyAgentId);
    }

    let request = Request {
        principal: PrincipalId(assignment.agent_id.0.clone()),
        permission: intent.permission,
        scope: intent.scope.clone(),
        risk: intent.risk,
    };
    Ok(GovernedAssignmentDecision {
        task_id: assignment.task_id.clone(),
        agent_id: assignment.agent_id.clone(),
        decision: policy.evaluate(request),
    })
}

#[cfg(test)]
mod tests {
    use agent_orchestrator::{AgentId, Assignment, TaskId};
    use autonomy_governor::{
        ApprovalRequirement, Decision, Grant, Permission, Policy, Principal, PrincipalId,
        RiskLevel, Scope, ScopeKind,
    };

    use super::*;

    fn assignment() -> Assignment {
        Assignment {
            task_id: TaskId("task-build".to_owned()),
            agent_id: AgentId("builder".to_owned()),
        }
    }

    fn policy(permission: Permission, scope: Scope) -> Policy {
        let principal = PrincipalId("builder".to_owned());
        Policy {
            principals: vec![Principal {
                id: principal.clone(),
                role: "builder".to_owned(),
            }],
            grants: vec![Grant {
                principal,
                permission,
                scope,
                approval: ApprovalRequirement::None,
            }],
        }
    }

    #[test]
    fn exact_low_risk_grant_is_allowed() {
        let assignment = assignment();
        let scope = Scope {
            kind: ScopeKind::Tool,
            resource: "compiler.check".to_owned(),
        };
        let intent = GovernedActionIntent {
            task_id: assignment.task_id.clone(),
            agent_id: assignment.agent_id.clone(),
            permission: Permission::Execute,
            scope: scope.clone(),
            risk: RiskLevel::Low,
        };
        let result =
            evaluate_assignment_action(&policy(Permission::Execute, scope), &assignment, &intent)
                .expect("exact assignment should evaluate");
        assert!(matches!(result.decision.decision, Decision::Allow { .. }));
    }

    #[test]
    fn protected_action_still_requires_human_approval() {
        let assignment = assignment();
        let scope = Scope {
            kind: ScopeKind::Release,
            resource: "gameforge.production".to_owned(),
        };
        let intent = GovernedActionIntent {
            task_id: assignment.task_id.clone(),
            agent_id: assignment.agent_id.clone(),
            permission: Permission::Publish,
            scope: scope.clone(),
            risk: RiskLevel::Protected,
        };
        let result =
            evaluate_assignment_action(&policy(Permission::Publish, scope), &assignment, &intent)
                .expect("protected action should evaluate");
        assert!(matches!(
            result.decision.decision,
            Decision::RequireHumanApproval { .. }
        ));
    }

    #[test]
    fn assignment_substitution_is_rejected_before_policy_evaluation() {
        let assignment = assignment();
        let scope = Scope {
            kind: ScopeKind::Tool,
            resource: "compiler.check".to_owned(),
        };
        let intent = GovernedActionIntent {
            task_id: TaskId("task-other".to_owned()),
            agent_id: assignment.agent_id.clone(),
            permission: Permission::Execute,
            scope: scope.clone(),
            risk: RiskLevel::Low,
        };
        let error =
            evaluate_assignment_action(&policy(Permission::Execute, scope), &assignment, &intent)
                .expect_err("task substitution must fail closed");
        assert!(matches!(error, IntegrationError::AssignmentMismatch { .. }));
    }
}
