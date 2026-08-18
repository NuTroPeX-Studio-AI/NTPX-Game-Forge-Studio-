use std::{
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    ApprovalContext, AuditEntry, AuditError, AuditLog, Decision, GovernanceError,
    GovernedFileTransaction, Permission, Policy, Request, ScopeKind, TransactionError,
    TransactionReceipt,
};

/// A runtime authorization permit bound to the exact audited request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizationPermit {
    pub request: Request,
    pub audit_entry: AuditEntry,
}

/// Canonical filesystem target resolved by the runtime boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedFilesystemTarget {
    path: PathBuf,
    parent: ParentIdentity,
}

impl ResolvedFilesystemTarget {
    /// Return the canonical absolute target path used for policy and mutation checks.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Return the canonical scope resource string for this target.
    #[must_use]
    pub fn resource(&self) -> String {
        self.path.to_string_lossy().into_owned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParentIdentity {
    canonical: PathBuf,
    #[cfg(unix)]
    device: u64,
    #[cfg(unix)]
    inode: u64,
}

/// Errors produced by the Phase 03 runtime enforcement boundary.
#[derive(Debug)]
pub enum RuntimeError {
    AbsolutePathRequired(PathBuf),
    MissingFileName(PathBuf),
    SymlinkAncestor(PathBuf),
    SymlinkTarget(PathBuf),
    ParentChanged(PathBuf),
    ScopeMismatch { expected: String, actual: String },
    Denied { reason: String },
    MissingApproval,
    Io(std::io::Error),
    Audit(AuditError),
    Governance(GovernanceError),
    Transaction(TransactionError),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AbsolutePathRequired(path) => write!(
                formatter,
                "governed filesystem target must be absolute: '{}'",
                path.display()
            ),
            Self::MissingFileName(path) => write!(
                formatter,
                "governed filesystem target '{}' has no file name",
                path.display()
            ),
            Self::SymlinkAncestor(path) => write!(
                formatter,
                "governed filesystem target crosses symlink ancestor '{}'",
                path.display()
            ),
            Self::SymlinkTarget(path) => write!(
                formatter,
                "governed filesystem target is a symlink: '{}'",
                path.display()
            ),
            Self::ParentChanged(path) => write!(
                formatter,
                "governed filesystem parent changed during authorization: '{}'",
                path.display()
            ),
            Self::ScopeMismatch { expected, actual } => write!(
                formatter,
                "canonical filesystem scope mismatch: expected '{expected}', got '{actual}'"
            ),
            Self::Denied { reason } => write!(formatter, "runtime request denied: {reason}"),
            Self::MissingApproval => write!(formatter, "runtime request requires human approval"),
            Self::Io(error) => write!(formatter, "runtime filesystem check failed: {error}"),
            Self::Audit(error) => write!(formatter, "runtime audit failed: {error}"),
            Self::Governance(error) => write!(formatter, "runtime governance failed: {error}"),
            Self::Transaction(error) => write!(formatter, "runtime transaction failed: {error}"),
        }
    }
}

impl Error for RuntimeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Audit(error) => Some(error),
            Self::Governance(error) => Some(error),
            Self::Transaction(error) => Some(error),
            Self::AbsolutePathRequired(_)
            | Self::MissingFileName(_)
            | Self::SymlinkAncestor(_)
            | Self::SymlinkTarget(_)
            | Self::ParentChanged(_)
            | Self::ScopeMismatch { .. }
            | Self::Denied { .. }
            | Self::MissingApproval => None,
        }
    }
}

impl From<std::io::Error> for RuntimeError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<AuditError> for RuntimeError {
    fn from(error: AuditError) -> Self {
        Self::Audit(error)
    }
}

impl From<GovernanceError> for RuntimeError {
    fn from(error: GovernanceError) -> Self {
        Self::Governance(error)
    }
}

impl From<TransactionError> for RuntimeError {
    fn from(error: TransactionError) -> Self {
        Self::Transaction(error)
    }
}

/// Resolve an ordinary-file target into the canonical scope representation required by the
/// runtime enforcement boundary.
///
/// The target must be absolute. Every existing ancestor is inspected with `symlink_metadata` and
/// symlink ancestors are rejected rather than followed. The parent is canonicalized once, its
/// identity is captured, and the final file name is appended without resolving a nonexistent
/// target through an attacker-controlled alias.
///
/// # Errors
/// Returns [`RuntimeError`] for relative paths, missing file names, symlink ancestors/targets,
/// missing parents, or other filesystem failures.
pub fn resolve_filesystem_target(
    target: impl AsRef<Path>,
) -> Result<ResolvedFilesystemTarget, RuntimeError> {
    let target = target.as_ref();
    if !target.is_absolute() {
        return Err(RuntimeError::AbsolutePathRequired(target.to_owned()));
    }
    let file_name = target
        .file_name()
        .ok_or_else(|| RuntimeError::MissingFileName(target.to_owned()))?;
    let parent = target
        .parent()
        .ok_or_else(|| RuntimeError::MissingFileName(target.to_owned()))?;

    reject_symlink_ancestors(parent)?;
    let parent_identity = capture_parent_identity(parent)?;
    let resolved = parent_identity.canonical.join(file_name);
    match fs::symlink_metadata(&resolved) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(RuntimeError::SymlinkTarget(resolved));
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(RuntimeError::Io(error)),
    }

    Ok(ResolvedFilesystemTarget {
        path: resolved,
        parent: parent_identity,
    })
}

/// Return the canonical scope resource string required for a governed filesystem request.
///
/// # Errors
/// Returns [`RuntimeError`] when the target cannot pass canonical runtime path checks.
pub fn canonical_filesystem_resource(target: impl AsRef<Path>) -> Result<String, RuntimeError> {
    resolve_filesystem_target(target).map(|resolved| resolved.resource())
}

/// Phase 03 in-process enforcement boundary. Operations supplied to [`Self::execute`] are invoked
/// only after policy evaluation and durable authorization evidence. Governed file writes also pass
/// canonical path checks before entering the rollback-capable transaction primitive.
pub struct GovernanceRuntime<'a> {
    policy: &'a Policy,
    audit_log: &'a AuditLog,
}

impl<'a> GovernanceRuntime<'a> {
    /// Construct a runtime bound to one immutable policy and durable audit log.
    #[must_use]
    pub const fn new(policy: &'a Policy, audit_log: &'a AuditLog) -> Self {
        Self { policy, audit_log }
    }

    /// Authorize one exact request and persist the resulting decision before returning a permit.
    /// Denials are also durably audited. Human-gated requests require a matching approval context.
    ///
    /// # Errors
    /// Returns [`RuntimeError`] for denial, missing/invalid/replayed approval, or audit failure.
    pub fn authorize(
        &self,
        request: Request,
        approval: Option<ApprovalContext<'_>>,
    ) -> Result<AuthorizationPermit, RuntimeError> {
        let base = self.policy.evaluate(request.clone());
        match &base.decision {
            Decision::Deny { reason } => {
                self.audit_log.append(base)?;
                Err(RuntimeError::Denied {
                    reason: reason.clone(),
                })
            }
            Decision::Allow { .. } => {
                let audit_entry = self.audit_log.append(base)?;
                Ok(AuthorizationPermit {
                    request,
                    audit_entry,
                })
            }
            Decision::RequireHumanApproval { .. } => {
                let approval = approval.ok_or(RuntimeError::MissingApproval)?;
                let audit_entry = self.policy.evaluate_and_audit_with_approval(
                    request.clone(),
                    Some(approval.proof),
                    approval.verifier,
                    self.audit_log,
                    approval.now_unix,
                )?;
                Ok(AuthorizationPermit {
                    request,
                    audit_entry,
                })
            }
        }
    }

    /// Execute an in-process operation only after a runtime permit has been durably established.
    /// The operation receives the exact permit so future adapters can require it explicitly.
    ///
    /// # Errors
    /// Returns [`RuntimeError`] when authorization fails or the operation itself returns an error.
    pub fn execute<T>(
        &self,
        request: Request,
        approval: Option<ApprovalContext<'_>>,
        operation: impl FnOnce(&AuthorizationPermit) -> Result<T, RuntimeError>,
    ) -> Result<T, RuntimeError> {
        let permit = self.authorize(request, approval)?;
        operation(&permit)
    }

    /// Commit one directly allowed local file write through canonical path checks and the verified
    /// rollback-capable transaction primitive.
    ///
    /// # Errors
    /// Returns [`RuntimeError`] for path, scope, authorization, transaction, audit, or rollback
    /// failures.
    pub fn write_file(
        &self,
        request: Request,
        target: impl AsRef<Path>,
        contents: &[u8],
        transaction_id: &str,
    ) -> Result<TransactionReceipt, RuntimeError> {
        let resolved = resolve_filesystem_target(target)?;
        validate_canonical_request(&request, &resolved)?;
        ensure_parent_unchanged(&resolved.parent)?;
        GovernedFileTransaction::commit(
            self.policy,
            request,
            resolved.path,
            contents,
            transaction_id,
            self.audit_log,
        )
        .map_err(RuntimeError::Transaction)
    }

    /// Commit one human-approved local file write through canonical path checks and the verified
    /// request-bound approval/rollback transaction primitive.
    ///
    /// # Errors
    /// Returns [`RuntimeError`] for path, scope, approval, transaction, audit, or rollback failures.
    pub fn write_file_with_approval(
        &self,
        request: Request,
        target: impl AsRef<Path>,
        contents: &[u8],
        transaction_id: &str,
        approval: ApprovalContext<'_>,
    ) -> Result<TransactionReceipt, RuntimeError> {
        let resolved = resolve_filesystem_target(target)?;
        validate_canonical_request(&request, &resolved)?;
        ensure_parent_unchanged(&resolved.parent)?;
        GovernedFileTransaction::commit_with_approval(
            self.policy,
            request,
            resolved.path,
            contents,
            transaction_id,
            approval,
            self.audit_log,
        )
        .map_err(RuntimeError::Transaction)
    }
}

fn validate_canonical_request(
    request: &Request,
    target: &ResolvedFilesystemTarget,
) -> Result<(), RuntimeError> {
    if request.permission != Permission::Write || request.scope.kind != ScopeKind::Filesystem {
        return Err(RuntimeError::Denied {
            reason: "runtime file mutation requires filesystem write permission".to_owned(),
        });
    }
    let actual = target.resource();
    if request.scope.resource != actual {
        return Err(RuntimeError::ScopeMismatch {
            expected: request.scope.resource.clone(),
            actual,
        });
    }
    Ok(())
}

fn reject_symlink_ancestors(parent: &Path) -> Result<(), RuntimeError> {
    for ancestor in parent.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(RuntimeError::SymlinkAncestor(ancestor.to_owned()));
            }
            Ok(_) => {}
            Err(error) => return Err(RuntimeError::Io(error)),
        }
    }
    Ok(())
}

fn capture_parent_identity(parent: &Path) -> Result<ParentIdentity, RuntimeError> {
    let canonical = fs::canonicalize(parent)?;
    let metadata = fs::metadata(&canonical)?;
    if !metadata.is_dir() {
        return Err(RuntimeError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "governed target parent is not a directory",
        )));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Ok(ParentIdentity {
            canonical,
            device: metadata.dev(),
            inode: metadata.ino(),
        })
    }

    #[cfg(not(unix))]
    {
        Ok(ParentIdentity { canonical })
    }
}

fn ensure_parent_unchanged(expected: &ParentIdentity) -> Result<(), RuntimeError> {
    let current = capture_parent_identity(&expected.canonical)?;
    if current == *expected {
        Ok(())
    } else {
        Err(RuntimeError::ParentChanged(expected.canonical.clone()))
    }
}
