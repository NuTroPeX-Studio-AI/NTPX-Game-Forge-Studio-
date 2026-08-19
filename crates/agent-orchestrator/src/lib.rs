//! Deterministic advisory task orchestration for NTPX `GameForge` Ω∞.
//!
//! This crate models agents, tasks, dependency graphs, state, and scheduling decisions. It does
//! not invoke models or tools, access credentials, mutate repositories/filesystems, or grant any
//! execution authority. Later execution adapters must consume the verified Autonomy Governor.

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

use serde::{Deserialize, Serialize};

/// Stable agent identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AgentId(pub String);

/// Stable task identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TaskId(pub String);

/// Provider-neutral capability required by an orchestration task.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TaskCapability(pub String);

/// Declarative agent capacity/capability metadata used only for scheduling.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentDescriptor {
    pub id: AgentId,
    pub capabilities: BTreeSet<TaskCapability>,
    pub max_concurrent_tasks: usize,
}

/// Declarative task contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskSpec {
    pub id: TaskId,
    pub dependencies: BTreeSet<TaskId>,
    pub required_capabilities: BTreeSet<TaskCapability>,
    pub max_attempts: u16,
}

/// Runtime-independent task state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum TaskState {
    Pending,
    RetryPending { attempts: u16 },
    Running { agent_id: AgentId, attempt: u16 },
    Succeeded,
    Failed { attempts: u16 },
    Escalated { reason: String },
}

/// Explicit state-machine input. These events record orchestration intent/outcome only; they do not
/// execute an agent, model, tool, or governed action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum TaskEvent {
    Start { agent_id: AgentId },
    Succeed,
    Fail,
    Escalate { reason: String },
}

/// Advisory assignment emitted by the scheduler.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assignment {
    pub task_id: TaskId,
    pub agent_id: AgentId,
}

/// Validated deterministic task graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskGraph {
    tasks: BTreeMap<TaskId, TaskSpec>,
    topological_order: Vec<TaskId>,
}

/// Task-graph validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphError {
    DuplicateTask(TaskId),
    EmptyTaskId,
    UnknownDependency {
        task_id: TaskId,
        dependency_id: TaskId,
    },
    SelfDependency(TaskId),
    ZeroAttempts(TaskId),
    CycleDetected(Vec<TaskId>),
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateTask(id) => write!(f, "duplicate task id '{}'", id.0),
            Self::EmptyTaskId => f.write_str("task ids must not be empty"),
            Self::UnknownDependency {
                task_id,
                dependency_id,
            } => write!(
                f,
                "task '{}' depends on unknown task '{}'",
                task_id.0, dependency_id.0
            ),
            Self::SelfDependency(id) => write!(f, "task '{}' cannot depend on itself", id.0),
            Self::ZeroAttempts(id) => write!(f, "task '{}' must allow at least one attempt", id.0),
            Self::CycleDetected(ids) => write!(
                f,
                "task dependency cycle detected across {} task(s)",
                ids.len()
            ),
        }
    }
}

impl Error for GraphError {}

/// Scheduling-input validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduleError {
    UnknownTaskState(TaskId),
    InvalidAgent(AgentId),
    InvalidTaskState(TaskId),
}

impl fmt::Display for ScheduleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownTaskState(id) => write!(f, "state supplied for unknown task '{}'", id.0),
            Self::InvalidAgent(id) => write!(
                f,
                "agent '{}' must have a non-empty id and positive concurrency",
                id.0
            ),
            Self::InvalidTaskState(id) => {
                write!(f, "task '{}' has state inconsistent with its retry budget", id.0)
            }
        }
    }
}

impl Error for ScheduleError {}

/// Fail-closed task state-transition failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionError {
    UnknownTask(TaskId),
    InvalidAgent(AgentId),
    InvalidEscalationReason,
    InvalidState {
        task_id: TaskId,
        state: TaskState,
    },
    InvalidTransition {
        task_id: TaskId,
        state: TaskState,
        event: TaskEvent,
    },
}

impl fmt::Display for TransitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownTask(id) => write!(f, "unknown task '{}'", id.0),
            Self::InvalidAgent(id) => write!(f, "agent id must not be empty: '{}'", id.0),
            Self::InvalidEscalationReason => f.write_str("escalation reason must not be empty"),
            Self::InvalidState { task_id, .. } => write!(
                f,
                "task '{}' has state inconsistent with its retry budget",
                task_id.0
            ),
            Self::InvalidTransition { task_id, .. } => {
                write!(f, "invalid state transition for task '{}'", task_id.0)
            }
        }
    }
}

impl Error for TransitionError {}

impl TaskGraph {
    /// Validate task identities/dependencies and compute a deterministic topological order.
    ///
    /// # Errors
    /// Returns [`GraphError`] for duplicate/empty IDs, invalid dependencies, zero attempts, or cycles.
    pub fn new(tasks: Vec<TaskSpec>) -> Result<Self, GraphError> {
        let mut by_id = BTreeMap::new();
        for task in tasks {
            if task.id.0.is_empty() {
                return Err(GraphError::EmptyTaskId);
            }
            if task.max_attempts == 0 {
                return Err(GraphError::ZeroAttempts(task.id));
            }
            let id = task.id.clone();
            if by_id.insert(id.clone(), task).is_some() {
                return Err(GraphError::DuplicateTask(id));
            }
        }
        for task in by_id.values() {
            for dep in &task.dependencies {
                if dep == &task.id {
                    return Err(GraphError::SelfDependency(task.id.clone()));
                }
                if !by_id.contains_key(dep) {
                    return Err(GraphError::UnknownDependency {
                        task_id: task.id.clone(),
                        dependency_id: dep.clone(),
                    });
                }
            }
        }

        let mut indegree: BTreeMap<TaskId, usize> =
            by_id.keys().cloned().map(|id| (id, 0)).collect();
        let mut dependents: BTreeMap<TaskId, BTreeSet<TaskId>> = BTreeMap::new();
        for task in by_id.values() {
            indegree.insert(task.id.clone(), task.dependencies.len());
            for dep in &task.dependencies {
                dependents
                    .entry(dep.clone())
                    .or_default()
                    .insert(task.id.clone());
            }
        }
        let mut ready: BTreeSet<TaskId> = indegree
            .iter()
            .filter(|(_, n)| **n == 0)
            .map(|(id, _)| id.clone())
            .collect();
        let mut order = Vec::with_capacity(by_id.len());
        while let Some(id) = ready.pop_first() {
            order.push(id.clone());
            if let Some(children) = dependents.get(&id) {
                for child in children {
                    if let Some(count) = indegree.get_mut(child) {
                        *count -= 1;
                        if *count == 0 {
                            ready.insert(child.clone());
                        }
                    }
                }
            }
        }
        if order.len() != by_id.len() {
            let unresolved = indegree
                .into_iter()
                .filter(|(_, n)| *n > 0)
                .map(|(id, _)| id)
                .collect();
            return Err(GraphError::CycleDetected(unresolved));
        }
        Ok(Self {
            tasks: by_id,
            topological_order: order,
        })
    }

    /// Return deterministic topological task order.
    #[must_use]
    pub fn topological_order(&self) -> &[TaskId] {
        &self.topological_order
    }

    /// Return an initial all-pending state map.
    #[must_use]
    pub fn initial_states(&self) -> BTreeMap<TaskId, TaskState> {
        self.tasks
            .keys()
            .cloned()
            .map(|id| (id, TaskState::Pending))
            .collect()
    }

    fn state_within_budget(task: &TaskSpec, state: &TaskState) -> bool {
        match state {
            TaskState::Pending | TaskState::Succeeded | TaskState::Escalated { .. } => true,
            TaskState::RetryPending { attempts } => *attempts > 0 && *attempts < task.max_attempts,
            TaskState::Running { attempt, .. } => *attempt > 0 && *attempt <= task.max_attempts,
            TaskState::Failed { attempts } => *attempts == task.max_attempts,
        }
    }

    /// Apply one explicit, deterministic orchestration state transition.
    ///
    /// A failed attempt becomes [`TaskState::RetryPending`] while budget remains and becomes
    /// [`TaskState::Failed`] exactly when the declared `max_attempts` budget is exhausted. Terminal
    /// states cannot be restarted. Escalation is explicit and requires a non-empty reason.
    ///
    /// # Errors
    /// Returns [`TransitionError`] for unknown tasks, malformed state, empty agent/reason values, or
    /// transitions that are not allowed by the state machine.
    pub fn transition(
        &self,
        task_id: &TaskId,
        state: &TaskState,
        event: TaskEvent,
    ) -> Result<TaskState, TransitionError> {
        let Some(task) = self.tasks.get(task_id) else {
            return Err(TransitionError::UnknownTask(task_id.clone()));
        };
        if !Self::state_within_budget(task, state) {
            return Err(TransitionError::InvalidState {
                task_id: task_id.clone(),
                state: state.clone(),
            });
        }
        if let TaskEvent::Start { agent_id } = &event {
            if agent_id.0.is_empty() {
                return Err(TransitionError::InvalidAgent(agent_id.clone()));
            }
        }
        if let TaskEvent::Escalate { reason } = &event {
            if reason.trim().is_empty() {
                return Err(TransitionError::InvalidEscalationReason);
            }
        }

        let next = match (state, &event) {
            (TaskState::Pending, TaskEvent::Start { agent_id }) => TaskState::Running {
                agent_id: agent_id.clone(),
                attempt: 1,
            },
            (TaskState::RetryPending { attempts }, TaskEvent::Start { agent_id }) => {
                TaskState::Running {
                    agent_id: agent_id.clone(),
                    attempt: attempts + 1,
                }
            }
            (TaskState::Running { .. }, TaskEvent::Succeed) => TaskState::Succeeded,
            (TaskState::Running { attempt, .. }, TaskEvent::Fail) if *attempt < task.max_attempts => {
                TaskState::RetryPending { attempts: *attempt }
            }
            (TaskState::Running { attempt, .. }, TaskEvent::Fail) => {
                TaskState::Failed { attempts: *attempt }
            }
            (
                TaskState::Pending | TaskState::RetryPending { .. } | TaskState::Running { .. },
                TaskEvent::Escalate { reason },
            ) => TaskState::Escalated {
                reason: reason.clone(),
            },
            _ => {
                return Err(TransitionError::InvalidTransition {
                    task_id: task_id.clone(),
                    state: state.clone(),
                    event,
                });
            }
        };
        Ok(next)
    }

    /// Produce deterministic advisory assignments for dependency-ready pending tasks.
    ///
    /// Running work consumes declared agent capacity. Tasks and agents are considered in stable ID
    /// order. Retry-pending tasks are schedulable only while their retained attempt count is within
    /// the validated task budget. Unassignable ready tasks remain pending; no hard constraint is
    /// silently relaxed.
    ///
    /// # Errors
    /// Returns [`ScheduleError`] for states referencing unknown tasks, states inconsistent with retry
    /// budgets, or invalid agent descriptors.
    pub fn plan_assignments(
        &self,
        states: &BTreeMap<TaskId, TaskState>,
        agents: &[AgentDescriptor],
    ) -> Result<Vec<Assignment>, ScheduleError> {
        for (id, state) in states {
            let Some(task) = self.tasks.get(id) else {
                return Err(ScheduleError::UnknownTaskState(id.clone()));
            };
            if !Self::state_within_budget(task, state) {
                return Err(ScheduleError::InvalidTaskState(id.clone()));
            }
        }
        let mut sorted_agents = agents.to_vec();
        sorted_agents.sort_by(|a, b| a.id.cmp(&b.id));
        for agent in &sorted_agents {
            if agent.id.0.is_empty() || agent.max_concurrent_tasks == 0 {
                return Err(ScheduleError::InvalidAgent(agent.id.clone()));
            }
        }
        let mut load: BTreeMap<AgentId, usize> =
            sorted_agents.iter().map(|a| (a.id.clone(), 0)).collect();
        for state in states.values() {
            if let TaskState::Running { agent_id, .. } = state {
                if let Some(n) = load.get_mut(agent_id) {
                    *n += 1;
                }
            }
        }
        let mut assignments = Vec::new();
        for task_id in &self.topological_order {
            let task = &self.tasks[task_id];
            let state = states.get(task_id).unwrap_or(&TaskState::Pending);
            if !matches!(state, TaskState::Pending | TaskState::RetryPending { .. }) {
                continue;
            }
            if !task
                .dependencies
                .iter()
                .all(|dep| matches!(states.get(dep), Some(TaskState::Succeeded)))
            {
                continue;
            }
            if let Some(agent) = sorted_agents.iter().find(|agent| {
                load.get(&agent.id).copied().unwrap_or_default() < agent.max_concurrent_tasks
                    && task.required_capabilities.is_subset(&agent.capabilities)
            }) {
                if let Some(agent_load) = load.get_mut(&agent.id) {
                    *agent_load += 1;
                    assignments.push(Assignment {
                        task_id: task_id.clone(),
                        agent_id: agent.id.clone(),
                    });
                }
            }
        }
        Ok(assignments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> TaskId {
        TaskId(value.to_owned())
    }
    fn capability(value: &str) -> TaskCapability {
        TaskCapability(value.to_owned())
    }
    fn task(value: &str, max_attempts: u16) -> TaskSpec {
        TaskSpec {
            id: id(value),
            dependencies: BTreeSet::new(),
            required_capabilities: BTreeSet::new(),
            max_attempts,
        }
    }

    #[test]
    fn rejects_cycles() {
        let a = TaskSpec {
            id: id("a"),
            dependencies: [id("b")].into(),
            required_capabilities: BTreeSet::new(),
            max_attempts: 1,
        };
        let b = TaskSpec {
            id: id("b"),
            dependencies: [id("a")].into(),
            required_capabilities: BTreeSet::new(),
            max_attempts: 1,
        };
        assert!(matches!(
            TaskGraph::new(vec![a, b]),
            Err(GraphError::CycleDetected(_))
        ));
    }

    #[test]
    fn schedules_only_dependency_ready_tasks_deterministically() {
        let prepare = TaskSpec {
            id: id("prepare"),
            dependencies: BTreeSet::new(),
            required_capabilities: [capability("code")].into(),
            max_attempts: 2,
        };
        let build = TaskSpec {
            id: id("build"),
            dependencies: [id("prepare")].into(),
            required_capabilities: [capability("code")].into(),
            max_attempts: 2,
        };
        let graph = TaskGraph::new(vec![build, prepare]).expect("graph should validate");
        let agents = vec![
            AgentDescriptor {
                id: AgentId("zeta".into()),
                capabilities: [capability("code")].into(),
                max_concurrent_tasks: 1,
            },
            AgentDescriptor {
                id: AgentId("alpha".into()),
                capabilities: [capability("code")].into(),
                max_concurrent_tasks: 1,
            },
        ];
        let mut states = graph.initial_states();
        let first = graph
            .plan_assignments(&states, &agents)
            .expect("schedule should validate");
        assert_eq!(
            first,
            vec![Assignment {
                task_id: id("prepare"),
                agent_id: AgentId("alpha".into())
            }]
        );
        states.insert(id("prepare"), TaskState::Succeeded);
        let second = graph
            .plan_assignments(&states, &agents)
            .expect("schedule should validate");
        assert_eq!(
            second,
            vec![Assignment {
                task_id: id("build"),
                agent_id: AgentId("alpha".into())
            }]
        );
    }

    #[test]
    fn respects_agent_capacity_and_capabilities() {
        let one = TaskSpec {
            id: id("one"),
            dependencies: BTreeSet::new(),
            required_capabilities: [capability("reason")].into(),
            max_attempts: 1,
        };
        let two = TaskSpec {
            id: id("two"),
            dependencies: BTreeSet::new(),
            required_capabilities: [capability("reason")].into(),
            max_attempts: 1,
        };
        let graph = TaskGraph::new(vec![one, two]).expect("graph should validate");
        let agents = vec![AgentDescriptor {
            id: AgentId("a".into()),
            capabilities: [capability("reason")].into(),
            max_concurrent_tasks: 1,
        }];
        let assignments = graph
            .plan_assignments(&graph.initial_states(), &agents)
            .expect("schedule should validate");
        assert_eq!(assignments.len(), 1);
        assert_eq!(assignments[0].task_id, id("one"));
    }

    #[test]
    fn bounds_retries_and_exhausts_exactly_at_max_attempts() {
        let graph = TaskGraph::new(vec![task("build", 2)]).expect("graph should validate");
        let task_id = id("build");
        let agent_id = AgentId("worker".into());

        let running_one = graph
            .transition(
                &task_id,
                &TaskState::Pending,
                TaskEvent::Start {
                    agent_id: agent_id.clone(),
                },
            )
            .expect("first attempt should start");
        assert_eq!(
            running_one,
            TaskState::Running {
                agent_id: agent_id.clone(),
                attempt: 1
            }
        );

        let retry = graph
            .transition(&task_id, &running_one, TaskEvent::Fail)
            .expect("first failure should retain retry budget state");
        assert_eq!(retry, TaskState::RetryPending { attempts: 1 });

        let running_two = graph
            .transition(
                &task_id,
                &retry,
                TaskEvent::Start {
                    agent_id: agent_id.clone(),
                },
            )
            .expect("second attempt should start");
        assert_eq!(
            running_two,
            TaskState::Running {
                agent_id,
                attempt: 2
            }
        );

        let failed = graph
            .transition(&task_id, &running_two, TaskEvent::Fail)
            .expect("second failure should exhaust the budget");
        assert_eq!(failed, TaskState::Failed { attempts: 2 });
        assert!(matches!(
            graph.transition(
                &task_id,
                &failed,
                TaskEvent::Start {
                    agent_id: AgentId("worker".into())
                }
            ),
            Err(TransitionError::InvalidTransition { .. })
        ));
    }

    #[test]
    fn retry_pending_work_is_schedulable_and_malformed_attempt_counts_fail_closed() {
        let graph = TaskGraph::new(vec![task("review", 3)]).expect("graph should validate");
        let agents = vec![AgentDescriptor {
            id: AgentId("agent".into()),
            capabilities: BTreeSet::new(),
            max_concurrent_tasks: 1,
        }];
        let mut states = graph.initial_states();
        states.insert(id("review"), TaskState::RetryPending { attempts: 1 });
        let assignments = graph
            .plan_assignments(&states, &agents)
            .expect("valid retry state should schedule");
        assert_eq!(assignments.len(), 1);

        states.insert(id("review"), TaskState::RetryPending { attempts: 3 });
        assert!(matches!(
            graph.plan_assignments(&states, &agents),
            Err(ScheduleError::InvalidTaskState(task_id)) if task_id == id("review")
        ));
    }

    #[test]
    fn success_and_escalation_are_explicit_terminal_transitions() {
        let graph = TaskGraph::new(vec![task("analyze", 2)]).expect("graph should validate");
        let task_id = id("analyze");
        let running = TaskState::Running {
            agent_id: AgentId("agent".into()),
            attempt: 1,
        };
        assert_eq!(
            graph
                .transition(&task_id, &running, TaskEvent::Succeed)
                .expect("running task should succeed"),
            TaskState::Succeeded
        );
        assert_eq!(
            graph
                .transition(
                    &task_id,
                    &TaskState::RetryPending { attempts: 1 },
                    TaskEvent::Escalate {
                        reason: "human review required".into()
                    }
                )
                .expect("active task should escalate"),
            TaskState::Escalated {
                reason: "human review required".into()
            }
        );
        assert!(matches!(
            graph.transition(
                &task_id,
                &TaskState::Pending,
                TaskEvent::Escalate { reason: "  ".into() }
            ),
            Err(TransitionError::InvalidEscalationReason)
        ));
    }
}
