//! Deterministic escalation routing for orchestrated tasks.
//!
//! Escalation is advisory metadata only. A route never grants permission or executes an action.

use std::{collections::BTreeSet, error::Error, fmt};

use agent_orchestrator::{TaskId, TaskState};
use serde::{Deserialize, Serialize};

/// Escalation destination type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscalationTargetKind {
    Human,
    SupervisorAgent,
    SafetyReview,
}

/// Stable escalation target.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EscalationTarget {
    pub id: String,
    pub kind: EscalationTargetKind,
}

/// One deterministic escalation rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EscalationRule {
    pub reason_prefix: String,
    pub targets: BTreeSet<EscalationTarget>,
}

/// Ordered escalation policy. The first matching prefix wins; fallback targets are required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EscalationPolicy {
    pub rules: Vec<EscalationRule>,
    pub fallback_targets: BTreeSet<EscalationTarget>,
}

/// Escalation routing result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EscalationRoute {
    pub task_id: TaskId,
    pub reason: String,
    pub targets: Vec<EscalationTarget>,
}

/// Fail-closed escalation failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EscalationError {
    TaskNotEscalated(TaskId),
    EmptyTargetId,
    EmptyFallback,
    EmptyRulePrefix,
    EmptyRuleTargets,
}

impl fmt::Display for EscalationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TaskNotEscalated(id) => write!(formatter, "task '{}' is not escalated", id.0),
            Self::EmptyTargetId => formatter.write_str("escalation target id must not be empty"),
            Self::EmptyFallback => {
                formatter.write_str("escalation fallback targets must not be empty")
            }
            Self::EmptyRulePrefix => {
                formatter.write_str("escalation rule prefix must not be empty")
            }
            Self::EmptyRuleTargets => {
                formatter.write_str("escalation rule targets must not be empty")
            }
        }
    }
}

impl Error for EscalationError {}

fn validate_targets(targets: &BTreeSet<EscalationTarget>) -> Result<(), EscalationError> {
    if targets.iter().any(|target| target.id.trim().is_empty()) {
        return Err(EscalationError::EmptyTargetId);
    }
    Ok(())
}

impl EscalationPolicy {
    /// Validate policy structure before routing.
    ///
    /// # Errors
    /// Returns [`EscalationError`] for empty fallback/rules/target IDs.
    pub fn validate(&self) -> Result<(), EscalationError> {
        if self.fallback_targets.is_empty() {
            return Err(EscalationError::EmptyFallback);
        }
        validate_targets(&self.fallback_targets)?;
        for rule in &self.rules {
            if rule.reason_prefix.trim().is_empty() {
                return Err(EscalationError::EmptyRulePrefix);
            }
            if rule.targets.is_empty() {
                return Err(EscalationError::EmptyRuleTargets);
            }
            validate_targets(&rule.targets)?;
        }
        Ok(())
    }

    /// Route an already-escalated task deterministically.
    ///
    /// # Errors
    /// Returns [`EscalationError`] when the policy is invalid or the supplied state is not escalated.
    pub fn route(
        &self,
        task_id: &TaskId,
        state: &TaskState,
    ) -> Result<EscalationRoute, EscalationError> {
        self.validate()?;
        let TaskState::Escalated { reason } = state else {
            return Err(EscalationError::TaskNotEscalated(task_id.clone()));
        };
        let targets = self
            .rules
            .iter()
            .find(|rule| reason.starts_with(&rule.reason_prefix))
            .map_or(&self.fallback_targets, |rule| &rule.targets)
            .iter()
            .cloned()
            .collect();
        Ok(EscalationRoute {
            task_id: task_id.clone(),
            reason: reason.clone(),
            targets,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(id: &str, kind: EscalationTargetKind) -> EscalationTarget {
        EscalationTarget {
            id: id.to_owned(),
            kind,
        }
    }

    #[test]
    fn routes_matching_reason_in_stable_target_order() {
        let policy = EscalationPolicy {
            rules: vec![EscalationRule {
                reason_prefix: "security:".to_owned(),
                targets: BTreeSet::from([
                    target("human-owner", EscalationTargetKind::Human),
                    target("safety-review", EscalationTargetKind::SafetyReview),
                ]),
            }],
            fallback_targets: BTreeSet::from([target("human-owner", EscalationTargetKind::Human)]),
        };
        let route = policy
            .route(
                &TaskId("task-1".to_owned()),
                &TaskState::Escalated {
                    reason: "security: protected action requested".to_owned(),
                },
            )
            .expect("valid escalation");
        assert_eq!(route.targets.len(), 2);
        assert_eq!(route.targets[0].id, "human-owner");
    }

    #[test]
    fn uses_fallback_when_no_rule_matches() {
        let policy = EscalationPolicy {
            rules: Vec::new(),
            fallback_targets: BTreeSet::from([target("human-owner", EscalationTargetKind::Human)]),
        };
        let route = policy
            .route(
                &TaskId("task-2".to_owned()),
                &TaskState::Escalated {
                    reason: "unknown failure".to_owned(),
                },
            )
            .expect("fallback route");
        assert_eq!(route.targets[0].id, "human-owner");
    }

    #[test]
    fn rejects_non_escalated_state() {
        let policy = EscalationPolicy {
            rules: Vec::new(),
            fallback_targets: BTreeSet::from([target("human-owner", EscalationTargetKind::Human)]),
        };
        let error = policy
            .route(&TaskId("task-3".to_owned()), &TaskState::Pending)
            .expect_err("pending task must not route as escalation");
        assert!(matches!(error, EscalationError::TaskNotEscalated(_)));
    }
}
