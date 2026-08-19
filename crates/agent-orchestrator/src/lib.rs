//! Deterministic advisory task orchestration for NTPX GameForge Ω∞.
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
    Running { agent_id: AgentId, attempt: u16 },
    Succeeded,
    Failed { attempts: u16 },
    Escalated { reason: String },
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
        }
    }
}

impl Error for ScheduleError {}

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
                    let count = indegree.get_mut(child).expect("validated child must exist");
                    *count -= 1;
                    if *count == 0 {
                        ready.insert(child.clone());
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

    /// Produce deterministic advisory assignments for dependency-ready pending tasks.
    ///
    /// Running work consumes declared agent capacity. Tasks and agents are considered in stable ID
    /// order. Unassignable ready tasks remain pending; no hard constraint is silently relaxed.
    ///
    /// # Errors
    /// Returns [`ScheduleError`] for states referencing unknown tasks or invalid agent descriptors.
    pub fn plan_assignments(
        &self,
        states: &BTreeMap<TaskId, TaskState>,
        agents: &[AgentDescriptor],
    ) -> Result<Vec<Assignment>, ScheduleError> {
        for id in states.keys() {
            if !self.tasks.contains_key(id) {
                return Err(ScheduleError::UnknownTaskState(id.clone()));
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
            if !matches!(
                states.get(task_id).unwrap_or(&TaskState::Pending),
                TaskState::Pending
            ) {
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
                *load
                    .get_mut(&agent.id)
                    .expect("validated agent load must exist") += 1;
                assignments.push(Assignment {
                    task_id: task_id.clone(),
                    agent_id: agent.id.clone(),
                });
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
}
