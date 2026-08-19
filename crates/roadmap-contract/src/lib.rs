//! Executable contract for the complete NTPX GameForge Ω∞ roadmap.
//!
//! The manifest is intentionally data-driven so CI can detect omitted phases/components and status
//! drift without turning roadmap prose into an implicit source of truth.

use std::{collections::BTreeSet, error::Error, fmt};

use serde::{Deserialize, Serialize};

/// Global non-negotiable roadmap rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoadmapRules {
    pub engine_neutral_core: bool,
    pub provider_neutral_core: bool,
    pub offline_first: bool,
    pub deny_by_default: bool,
    pub protected_actions_human_gated: bool,
    pub auditable_writes: bool,
    pub reversible_automated_writes: bool,
    pub verification_required_for_phase_completion: bool,
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
    RequiredRuleDisabled(&'static str),
    Parse(String),
}

impl fmt::Display for RoadmapError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSchemaVersion => formatter.write_str("roadmap schema version must be 1"),
            Self::WrongPhaseCount(count) => write!(formatter, "roadmap must contain 18 phases, found {count}"),
            Self::NonContiguousPhase { expected, actual } => write!(
                formatter,
                "roadmap phase IDs must be contiguous; expected {expected}, found {actual}"
            ),
            Self::EmptyPhaseName(phase) => write!(formatter, "phase {phase} name must not be empty"),
            Self::InvalidStatus { phase, status } => {
                write!(formatter, "phase {phase} has invalid status '{status}'")
            }
            Self::EmptyComponents(phase) => write!(formatter, "phase {phase} must declare components"),
            Self::EmptyComponent { phase } => write!(formatter, "phase {phase} contains an empty component"),
            Self::DuplicateComponent { phase, component } => write!(
                formatter,
                "phase {phase} contains duplicate component '{component}'"
            ),
            Self::RequiredRuleDisabled(rule) => write!(formatter, "required roadmap rule '{rule}' is disabled"),
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
        let manifest: Self = serde_json::from_str(raw).map_err(|error| RoadmapError::Parse(error.to_string()))?;
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
        for (index, phase) in self.phases.iter().enumerate() {
            let expected = u8::try_from(index + 1).expect("18 phases fit in u8");
            if phase.id != expected {
                return Err(RoadmapError::NonContiguousPhase {
                    expected,
                    actual: phase.id,
                });
            }
            if phase.name.trim().is_empty() {
                return Err(RoadmapError::EmptyPhaseName(phase.id));
            }
            if !matches!(phase.status.as_str(), "verified" | "in_progress" | "planned" | "blocked") {
                return Err(RoadmapError::InvalidStatus {
                    phase: phase.id,
                    status: phase.status.clone(),
                });
            }
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
        }
        let required = [
            (self.rules.engine_neutral_core, "engineNeutralCore"),
            (self.rules.provider_neutral_core, "providerNeutralCore"),
            (self.rules.offline_first, "offlineFirst"),
            (self.rules.deny_by_default, "denyByDefault"),
            (
                self.rules.protected_actions_human_gated,
                "protectedActionsHumanGated",
            ),
            (self.rules.auditable_writes, "auditableWrites"),
            (
                self.rules.reversible_automated_writes,
                "reversibleAutomatedWrites",
            ),
            (
                self.rules.verification_required_for_phase_completion,
                "verificationRequiredForPhaseCompletion",
            ),
        ];
        for (enabled, name) in required {
            if !enabled {
                return Err(RoadmapError::RequiredRuleDisabled(name));
            }
        }
        Ok(())
    }
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
        assert!(manifest.phases[..4].iter().all(|phase| phase.status == "verified"));
        assert_eq!(manifest.phases[4].status, "in_progress");
        assert!(manifest.phases[5..].iter().all(|phase| phase.status == "planned"));
    }

    #[test]
    fn release_phase_keeps_signing_and_human_approval_boundaries() {
        let manifest = RoadmapManifest::embedded().expect("embedded roadmap must be valid");
        let release = &manifest.phases[17];
        assert!(release.components.iter().any(|item| item == "signing-boundary"));
        assert!(release
            .components
            .iter()
            .any(|item| item == "human-release-approval"));
    }
}
