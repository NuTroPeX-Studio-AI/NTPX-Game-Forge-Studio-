use std::{
    error::Error,
    fmt,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::DecisionRecord;

/// Durable, hash-chained audit entry for one authorization decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEntry {
    pub sequence: u64,
    pub previous_hash: Option<String>,
    pub record: DecisionRecord,
    pub hash: String,
}

#[derive(Serialize)]
struct AuditPayload<'a> {
    sequence: u64,
    previous_hash: &'a Option<String>,
    record: &'a DecisionRecord,
}

/// Errors produced while persisting or verifying the audit log.
#[derive(Debug)]
pub enum AuditError {
    Io(std::io::Error),
    Encode(serde_json::Error),
    Decode {
        line: usize,
        source: serde_json::Error,
    },
    Integrity {
        line: usize,
        message: String,
    },
}

impl fmt::Display for AuditError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "audit I/O failed: {error}"),
            Self::Encode(error) => write!(formatter, "audit encoding failed: {error}"),
            Self::Decode { line, source } => {
                write!(formatter, "audit entry on line {line} is invalid JSON: {source}")
            }
            Self::Integrity { line, message } => {
                write!(formatter, "audit integrity failure on line {line}: {message}")
            }
        }
    }
}

impl Error for AuditError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Encode(error) => Some(error),
            Self::Decode { source, .. } => Some(source),
            Self::Integrity { .. } => None,
        }
    }
}

impl From<std::io::Error> for AuditError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

/// Append-only durable audit store.
///
/// Each entry is chained to the SHA-256 digest of the previous entry. The chain detects
/// accidental corruption and post-write edits, but it is not an authenticity mechanism: an
/// attacker with unrestricted write access could rewrite the entire file and recompute hashes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditLog {
    path: PathBuf,
}

impl AuditLog {
    /// Create an audit log handle for `path` without touching the filesystem.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Return the configured audit file path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Append one verified decision record and flush it to durable storage.
    ///
    /// Existing entries are fully verified before the append. Corrupt or malformed logs fail
    /// closed and are never extended.
    ///
    /// # Errors
    /// Returns [`AuditError`] if the existing log cannot be read or verified, if serialization
    /// fails, or if the append cannot be flushed to storage.
    pub fn append(&self, record: DecisionRecord) -> Result<AuditEntry, AuditError> {
        let entries = self.load_verified()?;
        let sequence = u64::try_from(entries.len()).map_err(|_| AuditError::Integrity {
            line: entries.len().saturating_add(1),
            message: "audit sequence exceeds u64 capacity".to_owned(),
        })?;
        let previous_hash = entries.last().map(|entry| entry.hash.clone());
        let hash = compute_hash(sequence, &previous_hash, &record)?;
        let entry = AuditEntry {
            sequence,
            previous_hash,
            record,
            hash,
        };

        let encoded = serde_json::to_string(&entry).map_err(AuditError::Encode)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        file.write_all(encoded.as_bytes())?;
        file.write_all(b"\n")?;
        file.sync_data()?;

        Ok(entry)
    }

    /// Load and verify every audit entry in order.
    ///
    /// Empty files and missing files are treated as empty logs. Any malformed line, sequence gap,
    /// chain mismatch, or digest mismatch fails closed.
    ///
    /// # Errors
    /// Returns [`AuditError`] when the file cannot be read or when any stored entry fails
    /// structural or integrity validation.
    pub fn load_verified(&self) -> Result<Vec<AuditEntry>, AuditError> {
        let file = match File::open(&self.path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(AuditError::Io(error)),
        };

        let mut entries = Vec::new();
        let mut expected_previous: Option<String> = None;

        for (index, line) in BufReader::new(file).lines().enumerate() {
            let line_number = index + 1;
            let line = line?;
            if line.trim().is_empty() {
                return Err(AuditError::Integrity {
                    line: line_number,
                    message: "empty audit line".to_owned(),
                });
            }

            let entry: AuditEntry = serde_json::from_str(&line).map_err(|source| {
                AuditError::Decode {
                    line: line_number,
                    source,
                }
            })?;
            let expected_sequence = u64::try_from(index).map_err(|_| AuditError::Integrity {
                line: line_number,
                message: "audit sequence exceeds u64 capacity".to_owned(),
            })?;

            if entry.sequence != expected_sequence {
                return Err(AuditError::Integrity {
                    line: line_number,
                    message: format!(
                        "expected sequence {expected_sequence}, found {}",
                        entry.sequence
                    ),
                });
            }

            if entry.previous_hash != expected_previous {
                return Err(AuditError::Integrity {
                    line: line_number,
                    message: "previous hash does not match prior entry".to_owned(),
                });
            }

            let expected_hash =
                compute_hash(entry.sequence, &entry.previous_hash, &entry.record)?;
            if entry.hash != expected_hash {
                return Err(AuditError::Integrity {
                    line: line_number,
                    message: "entry digest mismatch".to_owned(),
                });
            }

            expected_previous = Some(entry.hash.clone());
            entries.push(entry);
        }

        Ok(entries)
    }
}

fn compute_hash(
    sequence: u64,
    previous_hash: &Option<String>,
    record: &DecisionRecord,
) -> Result<String, AuditError> {
    let payload = AuditPayload {
        sequence,
        previous_hash,
        record,
    };
    let encoded = serde_json::to_vec(&payload).map_err(AuditError::Encode)?;
    let digest = Sha256::digest(encoded);
    Ok(to_lower_hex(&digest))
}

fn to_lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}
