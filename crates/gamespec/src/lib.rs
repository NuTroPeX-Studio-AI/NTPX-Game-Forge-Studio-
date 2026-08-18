//! Deterministic, engine-neutral `GameSpec` model, migration, validation, and compilation metadata.

mod graph;
mod id;
mod migrate;
mod model;
mod schema;
mod validate;

use std::{error::Error, fmt};

pub use graph::{DependencyEdge, DependencyEdgeKind, DependencyGraph};
pub use id::{deterministic_id, SpecId, SpecIdError};
pub use migrate::migrate_json;
pub use model::{
    AbilitySpec, ArchitectureSpec, AssetKind, AssetLicense, AssetOrigin, AssetProvenance,
    AssetSpec, BuildProfile, BuildSpec, ComponentSpec, DialogueNode, DialogueSpec, EngineTarget,
    EntitySpec, FeatureSpec, GameSpec, InputSpec, InputValueType, ModuleKind, ModuleSpec, NpcSpec,
    PlatformTarget, ProjectMetadata, QuestSpec, QuestStep, TestKind, TestSpec, WorldRegion,
    WorldSpec,
};
pub use schema::{json_schema, schema_json};
pub use validate::{ValidationError, ValidationReport};

/// Compatibility alias retained for the Phase 02 core API.
pub type ProjectSpec = GameSpec;

/// Current supported `GameSpec` schema version.
pub const CURRENT_SCHEMA_VERSION: &str = "0.1.0";

/// Errors produced while decoding, validating, migrating, or encoding a `GameSpec`.
#[derive(Debug)]
pub enum GameSpecError {
    Decode(serde_json::Error),
    Encode(serde_json::Error),
    Validation(Vec<ValidationError>),
    Migration(String),
}

impl fmt::Display for GameSpecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Decode(error) => write!(formatter, "failed to decode GameSpec: {error}"),
            Self::Encode(error) => write!(formatter, "failed to encode GameSpec: {error}"),
            Self::Validation(errors) => write!(
                formatter,
                "GameSpec validation failed with {} error(s)",
                errors.len()
            ),
            Self::Migration(message) => write!(formatter, "GameSpec migration failed: {message}"),
        }
    }
}

impl Error for GameSpecError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Decode(error) | Self::Encode(error) => Some(error),
            Self::Validation(_) | Self::Migration(_) => None,
        }
    }
}

/// Decode JSON and reject semantically invalid specifications.
///
/// # Errors
/// Returns [`GameSpecError::Decode`] for malformed JSON and
/// [`GameSpecError::Validation`] for semantic failures.
pub fn from_json(input: &str) -> Result<GameSpec, GameSpecError> {
    let spec = serde_json::from_str(input).map_err(GameSpecError::Decode)?;
    validate(&spec)?;
    Ok(spec)
}

/// Validate and serialize a specification using stable pretty JSON formatting.
///
/// # Errors
/// Returns [`GameSpecError::Validation`] for invalid input and
/// [`GameSpecError::Encode`] if serialization fails.
pub fn to_pretty_json(spec: &GameSpec) -> Result<String, GameSpecError> {
    validate(spec)?;
    serde_json::to_string_pretty(spec).map_err(GameSpecError::Encode)
}

/// Run semantic validation independent of serialization.
///
/// # Errors
/// Returns [`GameSpecError::Validation`] containing all findings.
pub fn validate(spec: &GameSpec) -> Result<(), GameSpecError> {
    let report = spec.validation_report();
    if report.is_valid() {
        Ok(())
    } else {
        Err(GameSpecError::Validation(report.into_errors()))
    }
}
