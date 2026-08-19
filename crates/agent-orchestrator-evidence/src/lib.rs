//! Deterministic advisory delegation and orchestration evidence for NTPX `GameForge` Ω∞.
//!
//! This crate binds explicit delegation directives to scheduler assignments and emits canonical,
//! reproducible SHA-256 evidence. It does not invoke agents, models, tools, network resources, or
//! governed actions and does not grant execution authority.

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

use agent_orchestrator::{AgentId, Assignment, TaskId, TaskState};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Explicit advisory delegation bound to a scheduler assignment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegationDirective {
    pub task_id: TaskId,
    pub delegated_by: AgentId,
    pub delegated_to: AgentId,
    pub reason: String,
}

/// Canonical task-state entry used inside deterministic evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskStateEvidence {
    pub task_id: TaskId,
    pub state: TaskState,
}

/// Canonical orchestration payload covered by the evidence digest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrchestrationEvidencePayload {
    pub schema_version: String,
    pub task_order: Vec<TaskId>,
    pub states: Vec<TaskStateEvidence>,
    pub assignments: Vec<Assignment>,
    pub delegations: Vec<DelegationDirective>,
}

/// Deterministic orchestration evidence and its SHA-256 correlation digest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrchestrationEvidence {
    pub payload: OrchestrationEvidencePayload,
    pub digest_sha256: String,
}

/// Fail-closed evidence validation or serialization failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceError {
    EmptyTaskId,
    DuplicateTask(TaskId),
    MissingTaskState(TaskId),
    UnknownTaskState(TaskId),
    UnknownAssignmentTask(TaskId),
    InvalidAssignmentAgent(AgentId),
    DuplicateAssignment(TaskId),
    UnknownDelegationTask(TaskId),
    InvalidDelegator(AgentId),
    InvalidDelegate(AgentId),
    SelfDelegation(TaskId),
    EmptyDelegationReason(TaskId),
    UnboundDelegation(TaskId),
    Serialization(String),
}

impl fmt::Display for EvidenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTaskId => f.write_str("task order contains an empty task id"),
            Self::DuplicateTask(id) => write!(f, "task order contains duplicate task '{}'", id.0),
            Self::MissingTaskState(id) => write!(f, "task '{}' is missing state evidence", id.0),
            Self::UnknownTaskState(id) => write!(f, "state supplied for unknown task '{}'", id.0),
            Self::UnknownAssignmentTask(id) => {
                write!(f, "assignment supplied for unknown task '{}'", id.0)
            }
            Self::InvalidAssignmentAgent(id) => {
                write!(f, "assignment contains empty agent id '{}'", id.0)
            }
            Self::DuplicateAssignment(id) => {
                write!(f, "task '{}' has more than one advisory assignment", id.0)
            }
            Self::UnknownDelegationTask(id) => {
                write!(f, "delegation supplied for unknown task '{}'", id.0)
            }
            Self::InvalidDelegator(id) => write!(f, "delegator id must not be empty: '{}'", id.0),
            Self::InvalidDelegate(id) => write!(f, "delegate id must not be empty: '{}'", id.0),
            Self::SelfDelegation(id) => {
                write!(f, "task '{}' cannot be delegated to the delegator", id.0)
            }
            Self::EmptyDelegationReason(id) => {
                write!(f, "task '{}' delegation reason must not be empty", id.0)
            }
            Self::UnboundDelegation(id) => write!(
                f,
                "task '{}' delegation is not bound to the same scheduler assignment",
                id.0
            ),
            Self::Serialization(message) => {
                write!(f, "failed to serialize orchestration evidence: {message}")
            }
        }
    }
}

impl Error for EvidenceError {}

/// Validate advisory delegation inputs, canonicalize ordering, and emit deterministic evidence.
///
/// Every task in `task_order` must have exactly one state entry. Each task may have at most one
/// advisory assignment. A delegation is valid only when its delegate exactly matches that task's
/// scheduler assignment, preventing delegation evidence from becoming a parallel scheduling path.
/// Input vector order does not affect the emitted payload or digest.
///
/// # Errors
/// Returns [`EvidenceError`] for malformed task/state/assignment/delegation inputs or serialization
/// failure.
pub fn collect_evidence(
    task_order: &[TaskId],
    states: &BTreeMap<TaskId, TaskState>,
    assignments: &[Assignment],
    delegations: &[DelegationDirective],
) -> Result<OrchestrationEvidence, EvidenceError> {
    let (known_tasks, canonical_states) = canonicalize_task_states(task_order, states)?;
    let (canonical_assignments, assignment_by_task) =
        canonicalize_assignments(&known_tasks, assignments)?;
    let canonical_delegations =
        canonicalize_delegations(&known_tasks, &assignment_by_task, delegations)?;

    let payload = OrchestrationEvidencePayload {
        schema_version: "1".to_owned(),
        task_order: task_order.to_vec(),
        states: canonical_states,
        assignments: canonical_assignments,
        delegations: canonical_delegations,
    };
    let encoded = serde_json::to_vec(&payload)
        .map_err(|error| EvidenceError::Serialization(error.to_string()))?;
    let digest_sha256 = encode_hex(&Sha256::digest(encoded));

    Ok(OrchestrationEvidence {
        payload,
        digest_sha256,
    })
}

fn canonicalize_task_states(
    task_order: &[TaskId],
    states: &BTreeMap<TaskId, TaskState>,
) -> Result<(BTreeSet<TaskId>, Vec<TaskStateEvidence>), EvidenceError> {
    let mut known_tasks = BTreeSet::new();
    for task_id in task_order {
        if task_id.0.is_empty() {
            return Err(EvidenceError::EmptyTaskId);
        }
        if !known_tasks.insert(task_id.clone()) {
            return Err(EvidenceError::DuplicateTask(task_id.clone()));
        }
    }

    for task_id in states.keys() {
        if !known_tasks.contains(task_id) {
            return Err(EvidenceError::UnknownTaskState(task_id.clone()));
        }
    }

    let mut canonical_states = Vec::with_capacity(task_order.len());
    for task_id in task_order {
        let Some(state) = states.get(task_id) else {
            return Err(EvidenceError::MissingTaskState(task_id.clone()));
        };
        canonical_states.push(TaskStateEvidence {
            task_id: task_id.clone(),
            state: state.clone(),
        });
    }
    Ok((known_tasks, canonical_states))
}

fn canonicalize_assignments(
    known_tasks: &BTreeSet<TaskId>,
    assignments: &[Assignment],
) -> Result<(Vec<Assignment>, BTreeMap<TaskId, AgentId>), EvidenceError> {
    let mut canonical_assignments = assignments.to_vec();
    canonical_assignments.sort_by(|left, right| {
        left.task_id
            .cmp(&right.task_id)
            .then_with(|| left.agent_id.cmp(&right.agent_id))
    });

    let mut assignment_by_task = BTreeMap::new();
    for assignment in &canonical_assignments {
        if !known_tasks.contains(&assignment.task_id) {
            return Err(EvidenceError::UnknownAssignmentTask(
                assignment.task_id.clone(),
            ));
        }
        if assignment.agent_id.0.is_empty() {
            return Err(EvidenceError::InvalidAssignmentAgent(
                assignment.agent_id.clone(),
            ));
        }
        if assignment_by_task
            .insert(assignment.task_id.clone(), assignment.agent_id.clone())
            .is_some()
        {
            return Err(EvidenceError::DuplicateAssignment(
                assignment.task_id.clone(),
            ));
        }
    }
    Ok((canonical_assignments, assignment_by_task))
}

fn canonicalize_delegations(
    known_tasks: &BTreeSet<TaskId>,
    assignment_by_task: &BTreeMap<TaskId, AgentId>,
    delegations: &[DelegationDirective],
) -> Result<Vec<DelegationDirective>, EvidenceError> {
    let mut canonical_delegations = delegations.to_vec();
    canonical_delegations.sort_by(|left, right| {
        left.task_id
            .cmp(&right.task_id)
            .then_with(|| left.delegated_by.cmp(&right.delegated_by))
            .then_with(|| left.delegated_to.cmp(&right.delegated_to))
            .then_with(|| left.reason.cmp(&right.reason))
    });

    for delegation in &canonical_delegations {
        if !known_tasks.contains(&delegation.task_id) {
            return Err(EvidenceError::UnknownDelegationTask(
                delegation.task_id.clone(),
            ));
        }
        if delegation.delegated_by.0.is_empty() {
            return Err(EvidenceError::InvalidDelegator(
                delegation.delegated_by.clone(),
            ));
        }
        if delegation.delegated_to.0.is_empty() {
            return Err(EvidenceError::InvalidDelegate(
                delegation.delegated_to.clone(),
            ));
        }
        if delegation.delegated_by == delegation.delegated_to {
            return Err(EvidenceError::SelfDelegation(delegation.task_id.clone()));
        }
        if delegation.reason.trim().is_empty() {
            return Err(EvidenceError::EmptyDelegationReason(
                delegation.task_id.clone(),
            ));
        }
        if assignment_by_task.get(&delegation.task_id) != Some(&delegation.delegated_to) {
            return Err(EvidenceError::UnboundDelegation(delegation.task_id.clone()));
        }
    }
    Ok(canonical_delegations)
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task_id(value: &str) -> TaskId {
        TaskId(value.to_owned())
    }

    fn agent_id(value: &str) -> AgentId {
        AgentId(value.to_owned())
    }

    #[test]
    fn canonicalizes_assignment_and_delegation_input_order() {
        let order = vec![task_id("prepare"), task_id("build")];
        let states = BTreeMap::from([
            (task_id("build"), TaskState::Pending),
            (task_id("prepare"), TaskState::Succeeded),
        ]);
        let assignments_a = vec![
            Assignment {
                task_id: task_id("build"),
                agent_id: agent_id("worker-b"),
            },
            Assignment {
                task_id: task_id("prepare"),
                agent_id: agent_id("worker-a"),
            },
        ];
        let assignments_b = assignments_a.iter().cloned().rev().collect::<Vec<_>>();
        let delegations_a = vec![
            DelegationDirective {
                task_id: task_id("build"),
                delegated_by: agent_id("director"),
                delegated_to: agent_id("worker-b"),
                reason: "build implementation".to_owned(),
            },
            DelegationDirective {
                task_id: task_id("prepare"),
                delegated_by: agent_id("director"),
                delegated_to: agent_id("worker-a"),
                reason: "prepare inputs".to_owned(),
            },
        ];
        let delegations_b = delegations_a.iter().cloned().rev().collect::<Vec<_>>();

        let first = collect_evidence(&order, &states, &assignments_a, &delegations_a)
            .expect("evidence should validate");
        let second = collect_evidence(&order, &states, &assignments_b, &delegations_b)
            .expect("reordered evidence should validate");

        assert_eq!(first, second);
        assert_eq!(first.digest_sha256.len(), 64);
    }

    #[test]
    fn rejects_delegation_that_bypasses_scheduler_assignment() {
        let order = vec![task_id("build")];
        let states = BTreeMap::from([(task_id("build"), TaskState::Pending)]);
        let assignments = vec![Assignment {
            task_id: task_id("build"),
            agent_id: agent_id("worker-a"),
        }];
        let delegations = vec![DelegationDirective {
            task_id: task_id("build"),
            delegated_by: agent_id("director"),
            delegated_to: agent_id("worker-b"),
            reason: "alternate worker".to_owned(),
        }];

        assert!(matches!(
            collect_evidence(&order, &states, &assignments, &delegations),
            Err(EvidenceError::UnboundDelegation(id)) if id == task_id("build")
        ));
    }

    #[test]
    fn rejects_multiple_assignments_for_one_task() {
        let order = vec![task_id("review")];
        let states = BTreeMap::from([(task_id("review"), TaskState::Pending)]);
        let assignments = vec![
            Assignment {
                task_id: task_id("review"),
                agent_id: agent_id("worker-a"),
            },
            Assignment {
                task_id: task_id("review"),
                agent_id: agent_id("worker-b"),
            },
        ];

        assert!(matches!(
            collect_evidence(&order, &states, &assignments, &[]),
            Err(EvidenceError::DuplicateAssignment(id)) if id == task_id("review")
        ));
    }

    #[test]
    fn rejects_missing_state_and_invalid_delegation_metadata() {
        let order = vec![task_id("analyze")];
        assert!(matches!(
            collect_evidence(&order, &BTreeMap::new(), &[], &[]),
            Err(EvidenceError::MissingTaskState(id)) if id == task_id("analyze")
        ));

        let states = BTreeMap::from([(task_id("analyze"), TaskState::Pending)]);
        let assignments = vec![Assignment {
            task_id: task_id("analyze"),
            agent_id: agent_id("worker"),
        }];
        let delegations = vec![DelegationDirective {
            task_id: task_id("analyze"),
            delegated_by: agent_id("worker"),
            delegated_to: agent_id("worker"),
            reason: " ".to_owned(),
        }];

        assert!(matches!(
            collect_evidence(&order, &states, &assignments, &delegations),
            Err(EvidenceError::SelfDelegation(id)) if id == task_id("analyze")
        ));
    }
}
