use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

/// Stable identifier used by `GameSpec` objects and references.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SpecId(String);

impl SpecId {
    /// Parse a canonical identifier.
    ///
    /// # Errors
    /// Returns [`SpecIdError`] when the value violates the canonical grammar.
    pub fn parse(value: impl Into<String>) -> Result<Self, SpecIdError> {
        let value = value.into();
        if is_valid_spec_id(&value) {
            Ok(Self(value))
        } else {
            Err(SpecIdError { value })
        }
    }

    /// Borrow the canonical string value.
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

/// Invalid `SpecId` error.
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

/// Produce a deterministic ID from a namespace and seed using a fixed FNV-1a hash.
#[must_use]
pub fn deterministic_id(namespace: &str, seed: &str) -> SpecId {
    let prefix = canonical_prefix(namespace);
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in seed.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    SpecId(format!("{prefix}-{hash:016x}"))
}

fn canonical_prefix(namespace: &str) -> String {
    let normalized: String = namespace
        .chars()
        .flat_map(char::to_lowercase)
        .map(|ch| {
            if ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '.' | '_' | '-') {
                ch
            } else {
                '-'
            }
        })
        .collect();
    let trimmed = normalized.trim_matches('-');
    if trimmed.is_empty() {
        "id".to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn is_valid_spec_id(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return false;
    }
    chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '.' | '_' | '-'))
}

#[cfg(test)]
mod tests {
    use super::{deterministic_id, SpecId};

    #[test]
    fn deterministic_ids_are_stable() {
        assert_eq!(
            deterministic_id("component", "Health"),
            deterministic_id("component", "Health")
        );
        assert_ne!(
            deterministic_id("component", "Health"),
            deterministic_id("component", "Inventory")
        );
    }

    #[test]
    fn spec_id_parser_rejects_noncanonical_values() {
        assert!(SpecId::parse("player-nema").is_ok());
        assert!(SpecId::parse("Player Nema").is_err());
    }
}
