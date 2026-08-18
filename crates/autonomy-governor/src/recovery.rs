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

use crate::{resolve_filesystem_target, RuntimeError};

const ROLLBACK_MARKER: &str = ".gameforge-rollback-";
const JSON_SUFFIX: &str = ".json";

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

/// Integrity classification for a discovered rollback artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryIntegrity {
    Verified,
    Corrupt,
}

/// Rollback artifact discovered by the conservative crash-recovery scanner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryCandidate {
    pub snapshot_path: PathBuf,
    pub transaction_id: Option<String>,
    pub target: Option<PathBuf>,
    pub existed_before: Option<bool>,
    pub integrity: RecoveryIntegrity,
}

/// Errors produced by crash-recovery discovery or explicit restoration.
#[derive(Debug)]
pub enum RecoveryError {
    InvalidRoot(PathBuf),
    InvalidSnapshotName(PathBuf),
    CorruptSnapshot(PathBuf),
    SnapshotTargetMismatch { expected: PathBuf, actual: PathBuf },
    Runtime(RuntimeError),
    Io(std::io::Error),
    Decode(serde_json::Error),
}

impl fmt::Display for RecoveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRoot(path) => write!(
                formatter,
                "recovery scan root must be an absolute existing ordinary directory: '{}'",
                path.display()
            ),
            Self::InvalidSnapshotName(path) => write!(
                formatter,
                "rollback artifact has invalid GameForge name: '{}'",
                path.display()
            ),
            Self::CorruptSnapshot(path) => write!(
                formatter,
                "rollback artifact failed integrity verification: '{}'",
                path.display()
            ),
            Self::SnapshotTargetMismatch { expected, actual } => write!(
                formatter,
                "rollback artifact target mismatch: expected '{}', got '{}'",
                expected.display(),
                actual.display()
            ),
            Self::Runtime(error) => write!(formatter, "recovery path validation failed: {error}"),
            Self::Io(error) => write!(formatter, "recovery I/O failed: {error}"),
            Self::Decode(error) => write!(formatter, "recovery snapshot decode failed: {error}"),
        }
    }
}

impl Error for RecoveryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Runtime(error) => Some(error),
            Self::Io(error) => Some(error),
            Self::Decode(error) => Some(error),
            Self::InvalidRoot(_)
            | Self::InvalidSnapshotName(_)
            | Self::CorruptSnapshot(_)
            | Self::SnapshotTargetMismatch { .. } => None,
        }
    }
}

impl From<std::io::Error> for RecoveryError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<RuntimeError> for RecoveryError {
    fn from(error: RuntimeError) -> Self {
        Self::Runtime(error)
    }
}

/// Conservative scanner for retained or crash-left rollback artifacts.
///
/// Scanning is read-only and never guesses whether a valid rollback snapshot represents a
/// successfully committed transaction or an interrupted one. A caller must explicitly select a
/// verified candidate for recovery. Directory symlinks are not followed.
#[derive(Debug, Clone, Copy, Default)]
pub struct RecoveryScanner;

impl RecoveryScanner {
    /// Recursively scan one absolute directory tree for GameForge rollback artifacts.
    ///
    /// # Errors
    /// Returns [`RecoveryError`] when the root is relative/invalid or directory traversal fails.
    pub fn scan_directory(root: impl AsRef<Path>) -> Result<Vec<RecoveryCandidate>, RecoveryError> {
        let root = root.as_ref();
        if !root.is_absolute() {
            return Err(RecoveryError::InvalidRoot(root.to_owned()));
        }
        let metadata = fs::symlink_metadata(root).map_err(RecoveryError::Io)?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(RecoveryError::InvalidRoot(root.to_owned()));
        }

        let mut candidates = Vec::new();
        scan_tree(root, &mut candidates)?;
        candidates.sort_by(|left, right| left.snapshot_path.cmp(&right.snapshot_path));
        Ok(candidates)
    }

    /// Explicitly restore one verified rollback candidate.
    ///
    /// Recovery re-reads and re-verifies the artifact, requires its encoded target to match the
    /// candidate, requires the target to pass the canonical runtime path boundary, and requires the
    /// rollback artifact to be the expected same-directory sibling for the target/transaction ID.
    /// The snapshot is removed only after successful restoration.
    ///
    /// # Errors
    /// Returns [`RecoveryError`] for corrupt/mismatched artifacts, path-hardening failures, or I/O
    /// failures. Corrupt candidates are never restored.
    pub fn recover(candidate: &RecoveryCandidate) -> Result<(), RecoveryError> {
        if candidate.integrity != RecoveryIntegrity::Verified {
            return Err(RecoveryError::CorruptSnapshot(
                candidate.snapshot_path.clone(),
            ));
        }
        let transaction_id = candidate
            .transaction_id
            .as_deref()
            .ok_or_else(|| RecoveryError::InvalidSnapshotName(candidate.snapshot_path.clone()))?;
        let target = candidate
            .target
            .as_ref()
            .ok_or_else(|| RecoveryError::CorruptSnapshot(candidate.snapshot_path.clone()))?;

        let encoded = fs::read(&candidate.snapshot_path)?;
        let snapshot: RollbackSnapshot =
            serde_json::from_slice(&encoded).map_err(RecoveryError::Decode)?;
        verify_snapshot(&snapshot, &candidate.snapshot_path)?;
        if snapshot.payload.target != *target {
            return Err(RecoveryError::SnapshotTargetMismatch {
                expected: target.clone(),
                actual: snapshot.payload.target,
            });
        }

        let resolved = resolve_filesystem_target(target)?;
        if resolved.path() != target {
            return Err(RecoveryError::SnapshotTargetMismatch {
                expected: resolved.path().to_owned(),
                actual: target.clone(),
            });
        }
        let expected_snapshot = rollback_path(target, transaction_id)?;
        if expected_snapshot != candidate.snapshot_path {
            return Err(RecoveryError::InvalidSnapshotName(
                candidate.snapshot_path.clone(),
            ));
        }

        if snapshot.payload.existed {
            let restore_path = recovery_restore_path(target, transaction_id)?;
            persist_new_file(&restore_path, &snapshot.payload.contents)?;
            if let Err(error) = replace_file(&restore_path, target) {
                let _ = fs::remove_file(&restore_path);
                return Err(RecoveryError::Io(error));
            }
        } else {
            match fs::remove_file(target) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(RecoveryError::Io(error)),
            }
        }

        fs::remove_file(&candidate.snapshot_path)?;
        Ok(())
    }
}

fn scan_tree(root: &Path, candidates: &mut Vec<RecoveryCandidate>) -> Result<(), RecoveryError> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            scan_tree(&path, candidates)?;
        } else if metadata.is_file() && parse_transaction_id(&path).is_some() {
            candidates.push(inspect_snapshot(&path));
        }
    }
    Ok(())
}

fn inspect_snapshot(path: &Path) -> RecoveryCandidate {
    let transaction_id = parse_transaction_id(path);
    match fs::read(path)
        .ok()
        .and_then(|encoded| serde_json::from_slice::<RollbackSnapshot>(&encoded).ok())
    {
        Some(snapshot) => {
            let verified = verify_snapshot(&snapshot, path).is_ok();
            RecoveryCandidate {
                snapshot_path: path.to_owned(),
                transaction_id,
                target: Some(snapshot.payload.target),
                existed_before: Some(snapshot.payload.existed),
                integrity: if verified {
                    RecoveryIntegrity::Verified
                } else {
                    RecoveryIntegrity::Corrupt
                },
            }
        }
        None => RecoveryCandidate {
            snapshot_path: path.to_owned(),
            transaction_id,
            target: None,
            existed_before: None,
            integrity: RecoveryIntegrity::Corrupt,
        },
    }
}

fn verify_snapshot(snapshot: &RollbackSnapshot, path: &Path) -> Result<(), RecoveryError> {
    let encoded = serde_json::to_vec(&snapshot.payload).map_err(RecoveryError::Decode)?;
    let digest = Sha256::digest(encoded);
    if snapshot.sha256 == to_lower_hex(&digest) {
        Ok(())
    } else {
        Err(RecoveryError::CorruptSnapshot(path.to_owned()))
    }
}

fn parse_transaction_id(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_str()?;
    let marker = name.find(ROLLBACK_MARKER)?;
    let suffix = name.strip_suffix(JSON_SUFFIX)?;
    let id = &suffix[marker + ROLLBACK_MARKER.len()..];
    if valid_id(id) {
        Some(id.to_owned())
    } else {
        None
    }
}

fn rollback_path(target: &Path, transaction_id: &str) -> Result<PathBuf, RecoveryError> {
    sibling_path(target, transaction_id, "rollback")
}

fn recovery_restore_path(target: &Path, transaction_id: &str) -> Result<PathBuf, RecoveryError> {
    sibling_path(target, transaction_id, "recovery-restore")
}

fn sibling_path(target: &Path, transaction_id: &str, kind: &str) -> Result<PathBuf, RecoveryError> {
    if !valid_id(transaction_id) {
        return Err(RecoveryError::InvalidSnapshotName(target.to_owned()));
    }
    let file_name = target
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| RecoveryError::InvalidSnapshotName(target.to_owned()))?;
    let parent = target
        .parent()
        .ok_or_else(|| RecoveryError::InvalidSnapshotName(target.to_owned()))?;
    Ok(parent.join(format!(
        ".{file_name}.gameforge-{kind}-{transaction_id}.json"
    )))
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn persist_new_file(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    let mut file = OpenOptions::new().create_new(true).write(true).open(path)?;
    file.write_all(contents)?;
    file.sync_all()?;
    Ok(())
}

#[cfg(not(windows))]
fn replace_file(stage: &Path, target: &Path) -> std::io::Result<()> {
    fs::rename(stage, target)
}

#[cfg(windows)]
fn replace_file(stage: &Path, target: &Path) -> std::io::Result<()> {
    let first_error = match fs::rename(stage, target) {
        Ok(()) => return Ok(()),
        Err(error) => error,
    };
    if !target.exists() {
        return Err(first_error);
    }

    let backup = stage.with_extension("backup");
    if backup.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "recovery replacement backup already exists",
        ));
    }
    fs::rename(target, &backup)?;
    match fs::rename(stage, target) {
        Ok(()) => {
            fs::remove_file(backup)?;
            Ok(())
        }
        Err(error) => {
            let _ = fs::rename(&backup, target);
            Err(error)
        }
    }
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
