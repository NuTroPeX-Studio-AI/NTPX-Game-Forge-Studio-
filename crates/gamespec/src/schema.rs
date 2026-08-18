use crate::GameSpecError;
use serde_json::Value;

const SCHEMA: &str = include_str!("../../../schemas/gamespec/v0.1.0/gamespec.schema.json");

/// Return the checked-in canonical JSON Schema text.
#[must_use]
pub const fn schema_json() -> &'static str {
    SCHEMA
}

/// Parse and return the canonical JSON Schema document.
///
/// # Errors
/// Returns [`GameSpecError::Decode`] if the checked-in schema is invalid JSON.
pub fn json_schema() -> Result<Value, GameSpecError> {
    serde_json::from_str(SCHEMA).map_err(GameSpecError::Decode)
}
