use std::{
    error::Error,
    ffi::OsStr,
    fmt,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{AuditEntry, AuditError, AuditLog, Decision, Permission, Policy, Request, ScopeKind};

/// Result of a committed governed file transaction. The snapshot is deliberately retained until
/// the caller explicitly rolls the transaction back or independently archives/removes it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionReceipt {
    pub transaction_id: String,
    pub target: PathBuf,
    pub snapshot_path: PathBuf,
    pub audit_entry: AuditEntry,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SnapshotPayload {
    target: PathBuf,
    existed: bool,
    contents: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RollbackSnapshot {
    payload: SnapshotPayload,
    sha256: String,
}

/// Errors produced by governed transactional filesystem writes.
#[derive(Debug)]
pub enum TransactionError {
    InvalidTransactionId,
    Unauthorized {
        reason: String,
    },
    ScopeMismatch {
        expected: String,
        actual: String,
    },
    SymlinkTarget(PathBuf),
    MissingFileName(PathBuf),
    Io(std::io::Error),
    Encode(serde_json::Error),
    Audit(AuditError),
    RollbackAfterAudit {
        audit: AuditError,
        rollback: std::io::Error,
    },
}

impl fmt::Display for TransactionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTransactionId => write!(formatter, "transaction id is not canonical"),
            Self::Unauthorized { reason } => write!(formatter, "transaction denied: {reason}"),
            Self::ScopeMismatch { expected, actual } => write!(
                formatter,
                "filesystem scope mismatch: expected '{expected}', got '{actual}'"
            ),
            Self::SymlinkTarget(path) => {
                write!(
                    formatter,
                    "refusing governed write through symlink '{}'",
                    path.display()
                )
            }
            Self::MissingFileName(path) => {
                write!(
                    formatter,
                    "transaction target '{}' has no file name",
                    path.display()
                )
            }
            Self::Io(error) => write!(formatter, "transaction I/O failed: {error}"),
            Self::Encode(error) => write!(formatter, "snapshot encoding failed: {error}"),
            Self::Audit(error) => write!(formatter, "transaction audit failed: {error}"),
            Self::RollbackAfterAudit { audit, rollback } => write!(
                formatter,
                "audit failed ({audit}) and automatic rollback also failed ({rollback})"
            ),
        }
    }
}

impl Error for TransactionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Encode(error) => Some(error),
            Self::Audit(error) => Some(error),
            Self::RollbackAfterAudit { audit, .. } => Some(audit),
            Self::InvalidTransactionId
            | Self::Unauthorized { .. }
            | Self::ScopeMismatch { .. }
            | Self::SymlinkTarget(_)
            | Self::MissingFileName(_) => None,
        }
    }
}

impl From<std::io::Error> for TransactionError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

/// Execute exact-path local filesystem writes through the verified policy boundary.
///
/// This initial transaction engine intentionally supports only ordinary files. It rejects symlink
/// targets, requires a direct `Allow` result for `Permission::Write` on a matching filesystem
/// scope, persists a durable pre-write snapshot, stages bytes in the target directory, renames the
/// staged file into place, and appends the decision to the verified audit log. If audit persistence
/// fails after replacement, the pre-write state is restored automatically.
#[derive(Debug, Clone, Copy, Default)]
pub struct GovernedFileTransaction;

impl GovernedFileTransaction {
    /// Commit one governed file replacement or creation.
    ///
    /// # Errors
    /// Returns [`TransactionError`] for invalid IDs, denied/non-matching authorization, symlink
    /// targets, snapshot/staging failures, audit failures, or rollback failures.
    pub fn commit(
        policy: &Policy,
        request: Request,
        target: impl Into<PathBuf>,
        contents: &[u8],
        transaction_id: &str,
        audit_log: &AuditLog,
    ) -> Result<TransactionReceipt, TransactionError> {
        validate_transaction_id(transaction_id)?;
        let target = target.into();
        validate_target(&target)?;

        let actual = target.to_string_lossy().into_owned();
        if request.permission != Permission::Write || request.scope.kind != ScopeKind::Filesystem {
            return Err(TransactionError::Unauthorized {
                reason: "transaction requires filesystem write permission".to_owned(),
            });
        }
        if request.scope.resource != actual {
            return Err(TransactionError::ScopeMismatch {
                expected: request.scope.resource.clone(),
                actual,
            });
        }

        let record = policy.evaluate(request);
        if !matches!(record.decision, Decision::Allow { .. }) {
            let reason = match &record.decision {
                Decision::Deny { reason } | Decision::RequireHumanApproval { reason } => {
                    reason.clone()
                }
                Decision::Allow { .. } => unreachable!(),
            };
            return Err(TransactionError::Unauthorized { reason });
        }

        let snapshot_path = sibling_path(&target, transaction_id, "rollback")?;
        let stage_path = sibling_path(&target, transaction_id, "stage")?;
        let snapshot = capture_snapshot(&target)?;
        persist_snapshot(&snapshot_path, &snapshot)?;

        if let Err(error) = persist_stage(&stage_path, contents).and_then(|()| {
            fs::rename(&stage_path, &target)?;
            Ok(())
        }) {
            let _ = fs::remove_file(&stage_path);
            return Err(TransactionError::Io(error));
        }

        let audit_entry = match audit_log.append(record) {
            Ok(entry) => entry,
            Err(audit) => match restore_snapshot(&snapshot_path, &target, transaction_id) {
                Ok(()) => return Err(TransactionError::Audit(audit)),
                Err(rollback) => {
                    return Err(TransactionError::RollbackAfterAudit { audit, rollback });
                }
            },
        };

        Ok(TransactionReceipt {
            transaction_id: transaction_id.to_owned(),
            target,
            snapshot_path,
            audit_entry,
        })
    }

    /// Restore a previously committed transaction from its durable snapshot.
    ///
    /// Successful rollback removes the consumed snapshot. The original authorization audit entry
    /// remains immutable evidence that the write was authorized and committed.
    ///
    /// # Errors
    /// Returns [`TransactionError`] when the snapshot is missing/corrupt, the target is a symlink,
    /// or restoration cannot be completed.
    pub fn rollback(receipt: &TransactionReceipt) -> Result<(), TransactionError> {
        validate_target(&receipt.target)?;
        restore_snapshot(
            &receipt.snapshot_path,
            &receipt.target,
            &receipt.transaction_id,
        )?;
        Ok(())
    }
}

fn validate_transaction_id(value: &str) -> Result<(), TransactionError> {
    let valid = !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'));
    if valid {
        Ok(())
    } else {
        Err(TransactionError::InvalidTransactionId)
    }
}

fn validate_target(target: &Path) -> Result<(), TransactionError> {
    if target.file_name().is_none() {
        return Err(TransactionError::MissingFileName(target.to_owned()));
    }
    match fs::symlink_metadata(target) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err(TransactionError::SymlinkTarget(target.to_owned()))
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(TransactionError::Io(error)),
    }
}

fn sibling_path(
    target: &Path,
    transaction_id: &str,
    kind: &str,
) -> Result<PathBuf, TransactionError> {
    let file_name = target
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| TransactionError::MissingFileName(target.to_owned()))?;
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    Ok(parent.join(format!(
        ".{file_name}.gameforge-{kind}-{transaction_id}.json"
    )))
}

fn capture_snapshot(target: &Path) -> Result<RollbackSnapshot, TransactionError> {
    let (existed, contents) = match fs::read(target) {
        Ok(contents) => (true, contents),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (false, Vec::new()),
        Err(error) => return Err(TransactionError::Io(error)),
    };
    let payload = SnapshotPayload {
        target: target.to_owned(),
        existed,
        contents,
    };
    let sha256 = snapshot_hash(&payload)?;
    Ok(RollbackSnapshot { payload, sha256 })
}

fn persist_snapshot(path: &Path, snapshot: &RollbackSnapshot) -> Result<(), TransactionError> {
    let encoded = serde_json::to_vec(snapshot).map_err(TransactionError::Encode)?;
    let mut file = OpenOptions::new().create_new(true).write(true).open(path)?;
    file.write_all(&encoded)?;
    file.sync_all()?;
    Ok(())
}

fn persist_stage(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    let mut file = OpenOptions::new().create_new(true).write(true).open(path)?;
    file.write_all(contents)?;
    file.sync_all()?;
    Ok(())
}

fn restore_snapshot(
    snapshot_path: &Path,
    target: &Path,
    transaction_id: &str,
) -> std::io::Result<()> {
    let encoded = fs::read(snapshot_path)?;
    let snapshot: RollbackSnapshot = serde_json::from_slice(&encoded)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
    let expected = snapshot_hash(&snapshot.payload)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string()))?;
    if snapshot.sha256 != expected || snapshot.payload.target != target {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "rollback snapshot integrity check failed",
        ));
    }

    if snapshot.payload.existed {
        let restore_path = sibling_path(target, transaction_id, "restore").map_err(|error| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, error.to_string())
        })?;
        persist_stage(&restore_path, &snapshot.payload.contents)?;
        fs::rename(&restore_path, target)?;
    } else {
        match fs::remove_file(target) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    fs::remove_file(snapshot_path)?;
    Ok(())
}

fn snapshot_hash(payload: &SnapshotPayload) -> Result<String, TransactionError> {
    let encoded = serde_json::to_vec(payload).map_err(TransactionError::Encode)?;
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
