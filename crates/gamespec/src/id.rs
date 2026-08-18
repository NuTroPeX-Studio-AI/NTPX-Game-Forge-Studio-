use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

/// Stable identifier used by `GameSpec` objects and references.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SpecId(String);

impl SpecId {
    /// Parses and validates a canonical identifier.
    ///
    /// IDs must begin with a lowercase ASCII letter or digit and may then contain
    /// lowercase ASCII letters, digits, dots, underscores, or hyphens.
    pub fn parse(value: impl Into<String>) -> Result<Self, SpecIdError> {
        let value = value.into();
        if is_valid_spec_id(&value) {
            Ok(Self(value))
        } else {
            Err(SpecIdError { value })
        }
    }

    /// Returns the identifier as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn is_valid(&self) -> bool {
        is_valid_spec_id(&self.0)
    }
}

impl fmt::Display for SpecId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Error returned when a `SpecId` violates the canonical ID grammar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecIdError {
    value: String,
}

impl fmt::Display for SpecIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid GameSpec id: {}", self.value)
    }
}

impl Error for SpecIdError {}

fn is_valid_spec_id(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return false;
    }

    chars.all(|ch| {
        ch.is_ascii_lowercase()
            || ch.is_ascii_digit()
            || matches!(ch, '.' | '_' | '-')
    })
}

#[cfg(test)]
mod tests {
    use super::SpecId;

    #[test]
    fn accepts_canonical_ids() {
        assert!(SpecId::parse("player-nema").is_ok());
        assert!(SpecId::parse("ability.monad.phase_shift").is_ok());
        assert!(SpecId::parse("m01").is_ok());
    }

    #[test]
    fn rejects_noncanonical_ids() {
        assert!(SpecId::parse("").is_err());
        assert!(SpecId::parse("Player").is_err());
        assert!(SpecId::parse("has space").is_err());
        assert!(SpecId::parse("/root").is_err());
    }
}
