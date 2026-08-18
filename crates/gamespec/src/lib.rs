//! Canonical machine-readable game specification for NTPX GameForge Ω∞.

mod id;
mod model;
mod validation;

pub use id::{SpecId, SpecIdError};
pub use model::{
    AbilitySpec, ArchitectureSpec, AssetKind, AssetLicense, AssetOrigin, AssetProvenance,
    AssetSpec, BuildProfile, BuildSpec, ComponentSpec, DialogueNode, DialogueSpec, EngineTarget,
    EntitySpec, FeatureSpec, GameSpec, InputSpec, InputValueType, ModuleKind, ModuleSpec, NpcSpec,
    PlatformTarget, ProjectSpec, QuestSpec, QuestStep, TestKind, TestSpec, WorldRegion, WorldSpec,
};
pub use validation::{ValidationIssue, ValidationReport};

/// Current supported `GameSpec` schema version.
pub const SUPPORTED_SCHEMA_VERSION: &str = "0.1.0";
