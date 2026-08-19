//! Deterministic build, test, and repair planning contracts for NTPX GameForge Ω∞.

use std::{collections::BTreeMap, error::Error, fmt};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepKind {
    Build,
    UnitTest,
    IntegrationTest,
    SmokeTest,
    Repair,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildStep {
    pub id: String,
    pub kind: StepKind,
    pub dependencies: Vec<String>,
    pub command_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildGraph {
    pub steps: BTreeMap<String, BuildStep>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepResult {
    pub step_id: String,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub output_digest_sha256: String,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailureFingerprint {
    pub step_id: String,
    pub digest_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepairPlan {
    pub failed_step_id: String,
    pub attempt: u16,
    pub max_attempts: u16,
    pub targeted_paths: Vec<String>,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildRepairError {
    EmptyStepId,
    DuplicateStep(String),
    UnknownDependency { step: String, dependency: String },
    SelfDependency(String),
    EmptyCommandId(String),
    ZeroRepairBudget,
    RepairBudgetExhausted,
    EmptyRepairPaths,
    EmptyRepairRationale,
}

impl fmt::Display for BuildRepairError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyStepId => formatter.write_str("build step id must not be empty"),
            Self::DuplicateStep(id) => write!(formatter, "duplicate build step '{id}'"),
            Self::UnknownDependency { step, dependency } => {
                write!(formatter, "step '{step}' depends on unknown step '{dependency}'")
            }
            Self::SelfDependency(id) => write!(formatter, "step '{id}' cannot depend on itself"),
            Self::EmptyCommandId(id) => write!(formatter, "step '{id}' command id must not be empty"),
            Self::ZeroRepairBudget => formatter.write_str("repair budget must be positive"),
            Self::RepairBudgetExhausted => formatter.write_str("repair budget exhausted"),
            Self::EmptyRepairPaths => formatter.write_str("repair plan must target at least one path"),
            Self::EmptyRepairRationale => formatter.write_str("repair rationale must not be empty"),
        }
    }
}

impl Error for BuildRepairError {}

impl BuildGraph {
    /// Construct a validated deterministic build graph.
    ///
    /// # Errors
    /// Returns [`BuildRepairError`] for duplicate IDs, empty commands, or invalid dependencies.
    pub fn new(steps: Vec<BuildStep>) -> Result<Self, BuildRepairError> {
        let mut by_id = BTreeMap::new();
        for step in steps {
            if step.id.trim().is_empty() {
                return Err(BuildRepairError::EmptyStepId);
            }
            if step.command_id.trim().is_empty() {
                return Err(BuildRepairError::EmptyCommandId(step.id));
            }
            let id = step.id.clone();
            if by_id.insert(id.clone(), step).is_some() {
                return Err(BuildRepairError::DuplicateStep(id));
            }
        }
        for step in by_id.values() {
            for dependency in &step.dependencies {
                if dependency == &step.id {
                    return Err(BuildRepairError::SelfDependency(step.id.clone()));
                }
                if !by_id.contains_key(dependency) {
                    return Err(BuildRepairError::UnknownDependency {
                        step: step.id.clone(),
                        dependency: dependency.clone(),
                    });
                }
            }
        }
        Ok(Self { steps: by_id })
    }
}

#[must_use]
pub fn fingerprint_failure(result: &StepResult) -> FailureFingerprint {
    let mut hasher = Sha256::new();
    hasher.update(result.step_id.as_bytes());
    hasher.update([u8::from(result.success)]);
    hasher.update(result.exit_code.unwrap_or_default().to_le_bytes());
    for diagnostic in &result.diagnostics {
        hasher.update(diagnostic.as_bytes());
    }
    FailureFingerprint {
        step_id: result.step_id.clone(),
        digest_sha256: format!("{:x}", hasher.finalize()),
    }
}

/// Construct a bounded repair plan.
///
/// # Errors
/// Returns [`BuildRepairError`] for exhausted budgets or malformed plans.
pub fn plan_repair(
    failed_step_id: impl Into<String>,
    attempt: u16,
    max_attempts: u16,
    mut targeted_paths: Vec<String>,
    rationale: impl Into<String>,
) -> Result<RepairPlan, BuildRepairError> {
    if max_attempts == 0 {
        return Err(BuildRepairError::ZeroRepairBudget);
    }
    if attempt == 0 || attempt > max_attempts {
        return Err(BuildRepairError::RepairBudgetExhausted);
    }
    if targeted_paths.is_empty() {
        return Err(BuildRepairError::EmptyRepairPaths);
    }
    let rationale = rationale.into();
    if rationale.trim().is_empty() {
        return Err(BuildRepairError::EmptyRepairRationale);
    }
    targeted_paths.sort();
    targeted_paths.dedup();
    Ok(RepairPlan {
        failed_step_id: failed_step_id.into(),
        attempt,
        max_attempts,
        targeted_paths,
        rationale,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graph_rejects_unknown_dependencies() {
        let error = BuildGraph::new(vec![BuildStep {
            id: "test".to_owned(),
            kind: StepKind::UnitTest,
            dependencies: vec!["build".to_owned()],
            command_id: "cargo.test".to_owned(),
        }])
        .expect_err("unknown dependency must fail");
        assert!(matches!(error, BuildRepairError::UnknownDependency { .. }));
    }

    #[test]
    fn repair_plan_is_bounded_and_deduplicated() {
        let plan = plan_repair(
            "rust-test",
            1,
            2,
            vec!["src/lib.rs".to_owned(), "src/lib.rs".to_owned()],
            "repair compiler error",
        )
        .expect("valid repair plan");
        assert_eq!(plan.targeted_paths.len(), 1);
    }
}
