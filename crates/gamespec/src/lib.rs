//! Deterministic, engine-neutral `GameSpec` data model and validation.

mod model;
mod validate;

use std::{error::Error, fmt};

pub use model::{EntitySpec, FeatureSpec, ModuleSpec, ProjectSpec};

/// Structured semantic validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

impl ValidationError {
    #[must_use]
    pub fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

/// Errors produced while decoding, validating, or encoding a `GameSpec`.
#[derive(Debug)]
pub enum GameSpecError {
    Decode(serde_json::Error),
    Encode(serde_json::Error),
    Validation(Vec<ValidationError>),
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
        }
    }
}

impl Error for GameSpecError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Decode(error) | Self::Encode(error) => Some(error),
            Self::Validation(_) => None,
        }
    }
}

/// Decode JSON and reject semantically invalid specifications.
///
/// # Errors
/// Returns [`GameSpecError::Decode`] for malformed JSON and
/// [`GameSpecError::Validation`] when the decoded specification violates semantic rules.
pub fn from_json(input: &str) -> Result<ProjectSpec, GameSpecError> {
    let spec = serde_json::from_str(input).map_err(GameSpecError::Decode)?;
    validate(&spec)?;
    Ok(spec)
}

/// Validate and serialize a specification using stable pretty JSON formatting.
///
/// # Errors
/// Returns [`GameSpecError::Validation`] for an invalid specification and
/// [`GameSpecError::Encode`] if JSON serialization fails.
pub fn to_pretty_json(spec: &ProjectSpec) -> Result<String, GameSpecError> {
    validate(spec)?;
    serde_json::to_string_pretty(spec).map_err(GameSpecError::Encode)
}

/// Run semantic validation independent of serialization.
///
/// # Errors
/// Returns [`GameSpecError::Validation`] with all discovered semantic validation failures.
pub fn validate(spec: &ProjectSpec) -> Result<(), GameSpecError> {
    let errors = validate::validate(spec);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(GameSpecError::Validation(errors))
    }
}
