//! Executable contract for the complete NTPX `GameForge` Ω∞ roadmap.
//!
//! The manifest is intentionally data-driven so CI can detect omitted phases/components and status
//! drift without turning roadmap prose into an implicit source of truth.

use std::{collections::BTreeSet, error::Error, fmt};

use serde::{Deserialize, Serialize};

const REQUIRED_RULES: [&str; 8] = [
    "engineNeutralCore",
    "providerNeutralCore",
    "offlineFirst",
    "denyByDefault",
    "protectedActionsHumanGated",
    "auditableWrites",
    "reversibleAutomatedWrites",
    "verificationRequiredForPhaseCompletion",
];

/// Global non-negotiable roadmap rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoadmapRules {
    pub required: BTreeSet<String>,
}

/// One required roadmap phase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhaseContract {
    pub id: u8,
    pub name: String,
    pub status: String,
    pub components: Vec<String>,
}

/// Complete roadmap manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoadmapManifest {
    pub schema_version: u16,
    pub product: String,
    pub rules: RoadmapRules,
    pub phases: Vec<PhaseContract>,
}

/// Structural manifest validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoadmapError {
    InvalidSchemaVersion,
    WrongPhaseCount(usize),
    NonContiguousPhase { expected: u8, actual: u8 },
    EmptyPhaseName(u8),
    InvalidStatus { phase: u8, status: String },
    EmptyComponents(u8),
    EmptyComponent { phase: u8 },
    DuplicateComponent { phase: u8, component: String },
    MissingRequiredRule(&'static str),
    UnexpectedRequiredRule(String),
    Parse(String),
}

impl fmt::Display for RoadmapError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSchemaVersion => formatter.write_str("roadmap schema version must be 1"),
            Self::WrongPhaseCount(count) => {
                write!(formatter, "roadmap must contain 18 phases, found {count}")
            }
            Self::NonContiguousPhase { expected, actual } => write!(
                formatter,
                "roadmap phase IDs must be contiguous; expected {expected}, found {actual}"
            ),
            Self::EmptyPhaseName(phase) => {
                write!(formatter, "phase {phase} name must not be empty")
            }
            Self::InvalidStatus { phase, status } => {
                write!(formatter, "phase {phase} has invalid status '{status}'")
            }
            Self::EmptyComponents(phase) => {
                write!(formatter, "phase {phase} must declare components")
            }
            Self::EmptyComponent { phase } => {
                write!(formatter, "phase {phase} contains an empty component")
            }
            Self::DuplicateComponent { phase, component } => write!(
                formatter,
                "phase {phase} contains duplicate component '{component}'"
            ),
            Self::MissingRequiredRule(rule) => {
                write!(formatter, "required roadmap rule '{rule}' is missing")
            }
            Self::UnexpectedRequiredRule(rule) => {
                write!(formatter, "unexpected roadmap rule '{rule}' is declared")
            }
            Self::Parse(reason) => write!(formatter, "roadmap manifest parse failed: {reason}"),
        }
    }
}

impl Error for RoadmapError {}

impl RoadmapManifest {
    /// Parse the repository's canonical embedded manifest.
    ///
    /// # Errors
    /// Returns [`RoadmapError`] if JSON decoding or structural validation fails.
    pub fn embedded() -> Result<Self, RoadmapError> {
        let raw = include_str!("../../../orchestration/roadmap.manifest.json");
        let manifest: Self =
            serde_json::from_str(raw).map_err(|error| RoadmapError::Parse(error.to_string()))?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Validate roadmap completeness and non-negotiable safety rules.
    ///
    /// # Errors
    /// Returns [`RoadmapError`] when a phase, component, status, or global rule is malformed.
    pub fn validate(&self) -> Result<(), RoadmapError> {
        if self.schema_version != 1 {
            return Err(RoadmapError::InvalidSchemaVersion);
        }
        if self.phases.len() != 18 {
            return Err(RoadmapError::WrongPhaseCount(self.phases.len()));
        }
        validate_phases(&self.phases)?;
        validate_required_rules(&self.rules.required)
    }
}

fn validate_phases(phases: &[PhaseContract]) -> Result<(), RoadmapError> {
    for (expected, phase) in (1_u8..=18).zip(phases) {
        if phase.id != expected {
            return Err(RoadmapError::NonContiguousPhase {
                expected,
                actual: phase.id,
            });
        }
        if phase.name.trim().is_empty() {
            return Err(RoadmapError::EmptyPhaseName(phase.id));
        }
        if !matches!(
            phase.status.as_str(),
            "verified" | "in_progress" | "planned" | "blocked"
        ) {
            return Err(RoadmapError::InvalidStatus {
                phase: phase.id,
                status: phase.status.clone(),
            });
        }
        validate_components(phase)?;
    }
    Ok(())
}

fn validate_components(phase: &PhaseContract) -> Result<(), RoadmapError> {
    if phase.components.is_empty() {
        return Err(RoadmapError::EmptyComponents(phase.id));
    }
    let mut seen = BTreeSet::new();
    for component in &phase.components {
        if component.trim().is_empty() {
            return Err(RoadmapError::EmptyComponent { phase: phase.id });
        }
        if !seen.insert(component) {
            return Err(RoadmapError::DuplicateComponent {
                phase: phase.id,
                component: component.clone(),
            });
        }
    }
    Ok(())
}

fn validate_required_rules(rules: &BTreeSet<String>) -> Result<(), RoadmapError> {
    for required in REQUIRED_RULES {
        if !rules.contains(required) {
            return Err(RoadmapError::MissingRequiredRule(required));
        }
    }
    if let Some(unexpected) = rules
        .iter()
        .find(|rule| !REQUIRED_RULES.contains(&rule.as_str()))
    {
        return Err(RoadmapError::UnexpectedRequiredRule(unexpected.clone()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_manifest_covers_all_eighteen_phases() {
        let manifest = RoadmapManifest::embedded().expect("embedded roadmap must be valid");
        assert_eq!(manifest.phases.len(), 18);
        assert_eq!(manifest.phases.first().map(|phase| phase.id), Some(1));
        assert_eq!(manifest.phases.last().map(|phase| phase.id), Some(18));
    }

    #[test]
    fn current_verified_and_in_progress_statuses_are_explicit() {
        let manifest = RoadmapManifest::embedded().expect("embedded roadmap must be valid");
        assert!(manifest.phases[..4]
            .iter()
            .all(|phase| phase.status == "verified"));
        assert_eq!(manifest.phases[4].status, "in_progress");
        assert!(manifest.phases[5..]
            .iter()
            .all(|phase| phase.status == "planned"));
    }

    #[test]
    fn release_phase_keeps_signing_and_human_approval_boundaries() {
        let manifest = RoadmapManifest::embedded().expect("embedded roadmap must be valid");
        let release = &manifest.phases[17];
        assert!(release
            .components
            .iter()
            .any(|item| item == "signing-boundary"));
        assert!(release
            .components
            .iter()
            .any(|item| item == "human-release-approval"));
    }

    #[test]
    fn required_rule_set_is_exact() {
        let manifest = RoadmapManifest::embedded().expect("embedded roadmap must be valid");
        assert_eq!(manifest.rules.required.len(), REQUIRED_RULES.len());
    }
}
