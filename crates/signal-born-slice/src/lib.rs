//! Signal Born vertical-slice feature contract for NTPX GameForge Ω∞.
//!
//! This crate declares the required gameplay/runtime capabilities independently of one engine.

use std::{collections::BTreeSet, error::Error, fmt};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameplayCapability {
    Walk,
    Sprint,
    Crouch,
    Jump,
    Vault,
    Aim,
    Shoot,
    Interact,
    FirstPersonCamera,
    ThirdPersonCamera,
    CameraSwitch,
    InputRemap,
    KeyboardMouse,
    Gamepad,
    VrInput,
    Combat,
    EnemyAi,
    NpcInteraction,
    OpenMicConversation,
    ObjectiveSystem,
    QuestObjective,
    Level,
    WorldStreaming,
    Hud,
    Menus,
    GameState,
    SaveLoad,
    Checkpoint,
    Animation,
    Audio,
    Physics,
    Rendering,
    Networking,
    Accessibility,
    PerformanceBudgets,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerticalSliceContract {
    pub project_id: String,
    pub gamespec_path: String,
    pub required: BTreeSet<GameplayCapability>,
    pub performance_target_fps: u16,
    pub save_schema_version: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeCapabilityReport {
    pub implemented: BTreeSet<GameplayCapability>,
    pub tested: BTreeSet<GameplayCapability>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SliceReadiness {
    pub missing_implementation: Vec<GameplayCapability>,
    pub missing_tests: Vec<GameplayCapability>,
    pub ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SliceError {
    EmptyProjectId,
    EmptyGameSpecPath,
    EmptyRequiredCapabilities,
    ZeroPerformanceTarget,
    ZeroSaveSchemaVersion,
    TestedButNotImplemented(GameplayCapability),
}

impl fmt::Display for SliceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "vertical-slice validation failed: {self:?}")
    }
}

impl Error for SliceError {}

impl VerticalSliceContract {
    #[must_use]
    pub fn signal_born_default() -> Self {
        Self {
            project_id: "signal-born-first-thought".to_owned(),
            gamespec_path: "examples/signal-born-first-thought.gamespec.json".to_owned(),
            required: BTreeSet::from([
                GameplayCapability::Walk,
                GameplayCapability::Sprint,
                GameplayCapability::Crouch,
                GameplayCapability::Jump,
                GameplayCapability::Vault,
                GameplayCapability::Aim,
                GameplayCapability::Shoot,
                GameplayCapability::Interact,
                GameplayCapability::FirstPersonCamera,
                GameplayCapability::ThirdPersonCamera,
                GameplayCapability::CameraSwitch,
                GameplayCapability::InputRemap,
                GameplayCapability::KeyboardMouse,
                GameplayCapability::Gamepad,
                GameplayCapability::VrInput,
                GameplayCapability::Combat,
                GameplayCapability::EnemyAi,
                GameplayCapability::NpcInteraction,
                GameplayCapability::OpenMicConversation,
                GameplayCapability::ObjectiveSystem,
                GameplayCapability::QuestObjective,
                GameplayCapability::Level,
                GameplayCapability::WorldStreaming,
                GameplayCapability::Hud,
                GameplayCapability::Menus,
                GameplayCapability::GameState,
                GameplayCapability::SaveLoad,
                GameplayCapability::Checkpoint,
                GameplayCapability::Animation,
                GameplayCapability::Audio,
                GameplayCapability::Physics,
                GameplayCapability::Rendering,
                GameplayCapability::Networking,
                GameplayCapability::Accessibility,
                GameplayCapability::PerformanceBudgets,
            ]),
            performance_target_fps: 60,
            save_schema_version: 1,
        }
    }

    /// Compare implemented/tested runtime capabilities against the complete slice contract.
    ///
    /// # Errors
    /// Returns [`SliceError`] for malformed contracts or reports that claim tests for unimplemented
    /// capabilities.
    pub fn assess(&self, report: &RuntimeCapabilityReport) -> Result<SliceReadiness, SliceError> {
        if self.project_id.trim().is_empty() {
            return Err(SliceError::EmptyProjectId);
        }
        if self.gamespec_path.trim().is_empty() {
            return Err(SliceError::EmptyGameSpecPath);
        }
        if self.required.is_empty() {
            return Err(SliceError::EmptyRequiredCapabilities);
        }
        if self.performance_target_fps == 0 {
            return Err(SliceError::ZeroPerformanceTarget);
        }
        if self.save_schema_version == 0 {
            return Err(SliceError::ZeroSaveSchemaVersion);
        }
        if let Some(capability) = report.tested.difference(&report.implemented).next() {
            return Err(SliceError::TestedButNotImplemented(*capability));
        }
        let missing_implementation: Vec<_> = self
            .required
            .difference(&report.implemented)
            .copied()
            .collect();
        let missing_tests: Vec<_> = self.required.difference(&report.tested).copied().collect();
        let ready = missing_implementation.is_empty() && missing_tests.is_empty();
        Ok(SliceReadiness {
            missing_implementation,
            missing_tests,
            ready,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_runtime_report_is_not_ready_and_lists_everything() {
        let contract = VerticalSliceContract::signal_born_default();
        let readiness = contract
            .assess(&RuntimeCapabilityReport {
                implemented: BTreeSet::new(),
                tested: BTreeSet::new(),
            })
            .expect("valid report");
        assert!(!readiness.ready);
        assert_eq!(
            readiness.missing_implementation.len(),
            contract.required.len()
        );
    }

    #[test]
    fn tests_cannot_exist_without_implementation() {
        let contract = VerticalSliceContract::signal_born_default();
        let error = contract
            .assess(&RuntimeCapabilityReport {
                implemented: BTreeSet::new(),
                tested: BTreeSet::from([GameplayCapability::Walk]),
            })
            .expect_err("test-only claim must fail closed");
        assert_eq!(
            error,
            SliceError::TestedButNotImplemented(GameplayCapability::Walk)
        );
    }
}
