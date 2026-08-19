//! Governed exact-edit application boundary for NTPX `GameForge` Ω∞.
//!
//! Planning remains read-only in `code-intelligence-edits`. This crate performs the narrow bridge
//! from a fresh, evidence-bound exact edit plan to the verified Autonomy Governor runtime and its
//! rollback-capable local-file transaction. It grants no repository merge, network, credential,
//! model, tool, engine, publishing, signing, or release authority.

use std::{
    error::Error,
    fmt, fs,
    path::{Component, Path, PathBuf},
};

use autonomy_governor::{
    resolve_filesystem_target, ApprovalContext, GovernanceRuntime, Request, RuntimeError,
    TransactionReceipt,
};
use code_intelligence::SourceFile;
use code_intelligence_edits::{verify_edit_plan_fresh, EditPlanError, ExactEditPlan};
use sha2::{Digest, Sha256};

/// Evidence returned after a fresh exact plan is committed through the Governor runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedExactEdit {
    pub plan_evidence_sha256: String,
    pub repository_evidence_sha256: String,
    pub receipt: TransactionReceipt,
}

/// Fail-closed errors for exact-plan application.
#[derive(Debug)]
pub enum ApplyEditError {
    InvalidRepositoryRoot(PathBuf),
    NonCanonicalPlanPath(String),
    ReplacementEvidenceMismatch {
        expected: String,
        actual: String,
    },
    PlanEvidenceMismatch {
        expected: String,
        actual: String,
    },
    SourceRead {
        path: PathBuf,
        source: std::io::Error,
    },
    EditPlan(EditPlanError),
    Runtime(RuntimeError),
}

impl fmt::Display for ApplyEditError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRepositoryRoot(path) => write!(
                formatter,
                "repository root '{}' must be an existing directory",
                path.display()
            ),
            Self::NonCanonicalPlanPath(path) => write!(
                formatter,
                "edit plan path '{path}' must be a non-empty repository-relative normal path"
            ),
            Self::ReplacementEvidenceMismatch { expected, actual } => write!(
                formatter,
                "replacement-content evidence mismatch: expected {expected}, got {actual}"
            ),
            Self::PlanEvidenceMismatch { expected, actual } => write!(
                formatter,
                "edit-plan evidence mismatch: expected {expected}, got {actual}"
            ),
            Self::SourceRead { path, source } => write!(
                formatter,
                "failed to read current source '{}': {source}",
                path.display()
            ),
            Self::EditPlan(error) => write!(formatter, "edit plan is not fresh: {error}"),
            Self::Runtime(error) => write!(formatter, "governed edit application failed: {error}"),
        }
    }
}

impl Error for ApplyEditError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::SourceRead { source, .. } => Some(source),
            Self::EditPlan(error) => Some(error),
            Self::Runtime(error) => Some(error),
            Self::InvalidRepositoryRoot(_)
            | Self::NonCanonicalPlanPath(_)
            | Self::ReplacementEvidenceMismatch { .. }
            | Self::PlanEvidenceMismatch { .. } => None,
        }
    }
}

impl From<EditPlanError> for ApplyEditError {
    fn from(error: EditPlanError) -> Self {
        Self::EditPlan(error)
    }
}

impl From<RuntimeError> for ApplyEditError {
    fn from(error: RuntimeError) -> Self {
        Self::Runtime(error)
    }
}

/// Apply one exact whole-file replacement through a directly allowed Governor write.
///
/// The plan is integrity-checked, bound to an existing canonical repository root, refreshed from
/// the current on-disk source, and revalidated against the supplied repository evidence before the
/// Governor runtime is entered. The Governor then performs its own canonical path, policy, audit,
/// snapshot, replacement, and rollback-capable transaction checks.
///
/// # Errors
/// Returns [`ApplyEditError`] for malformed/tampered/stale plan evidence, unsafe repository-relative
/// paths, source read failures, denied authorization, or governed transaction failure.
pub fn apply_exact_file_replacement(
    runtime: &GovernanceRuntime<'_>,
    request: Request,
    repository_root: impl AsRef<Path>,
    plan: &ExactEditPlan,
    repository_evidence_sha256: &str,
    transaction_id: &str,
) -> Result<AppliedExactEdit, ApplyEditError> {
    let target = preflight(repository_root.as_ref(), plan, repository_evidence_sha256)?;
    let receipt = runtime.write_file(
        request,
        &target,
        plan.replacement_content.as_bytes(),
        transaction_id,
    )?;
    Ok(AppliedExactEdit {
        plan_evidence_sha256: plan.evidence_sha256.clone(),
        repository_evidence_sha256: repository_evidence_sha256.to_owned(),
        receipt,
    })
}

/// Apply one exact whole-file replacement through a human-approved Governor write.
///
/// A valid plan cannot weaken or satisfy a human approval gate. The approval proof is still bound
/// to the exact Governor request and is verified/consumed by the existing Autonomy Governor path.
///
/// # Errors
/// Returns [`ApplyEditError`] for plan/path/source freshness failures, missing/invalid/replayed
/// human approval, denied authorization, or governed transaction failure.
pub fn apply_exact_file_replacement_with_approval(
    runtime: &GovernanceRuntime<'_>,
    request: Request,
    repository_root: impl AsRef<Path>,
    plan: &ExactEditPlan,
    repository_evidence_sha256: &str,
    transaction_id: &str,
    approval: ApprovalContext<'_>,
) -> Result<AppliedExactEdit, ApplyEditError> {
    let target = preflight(repository_root.as_ref(), plan, repository_evidence_sha256)?;
    let receipt = runtime.write_file_with_approval(
        request,
        &target,
        plan.replacement_content.as_bytes(),
        transaction_id,
        approval,
    )?;
    Ok(AppliedExactEdit {
        plan_evidence_sha256: plan.evidence_sha256.clone(),
        repository_evidence_sha256: repository_evidence_sha256.to_owned(),
        receipt,
    })
}

fn preflight(
    repository_root: &Path,
    plan: &ExactEditPlan,
    repository_evidence_sha256: &str,
) -> Result<PathBuf, ApplyEditError> {
    verify_plan_integrity(plan)?;
    let canonical_root = fs::canonicalize(repository_root)
        .map_err(|_| ApplyEditError::InvalidRepositoryRoot(repository_root.to_owned()))?;
    if !canonical_root.is_dir() {
        return Err(ApplyEditError::InvalidRepositoryRoot(
            repository_root.to_owned(),
        ));
    }

    let relative = Path::new(&plan.path);
    let mut saw_component = false;
    for component in relative.components() {
        match component {
            Component::Normal(_) => saw_component = true,
            Component::Prefix(_)
            | Component::RootDir
            | Component::CurDir
            | Component::ParentDir => {
                return Err(ApplyEditError::NonCanonicalPlanPath(plan.path.clone()));
            }
        }
    }
    if !saw_component {
        return Err(ApplyEditError::NonCanonicalPlanPath(plan.path.clone()));
    }

    let lexical_target = canonical_root.join(relative);
    let resolved = resolve_filesystem_target(&lexical_target)?;
    let target = resolved.path().to_owned();
    let content = fs::read_to_string(&target).map_err(|source| ApplyEditError::SourceRead {
        path: target.clone(),
        source,
    })?;
    let current_source = SourceFile {
        path: plan.path.clone(),
        language: plan.language,
        content,
        generated: false,
    };
    verify_edit_plan_fresh(plan, repository_evidence_sha256, &current_source)?;
    Ok(target)
}

fn verify_plan_integrity(plan: &ExactEditPlan) -> Result<(), ApplyEditError> {
    let replacement = sha256_hex(plan.replacement_content.as_bytes());
    if replacement != plan.replacement_content_sha256 {
        return Err(ApplyEditError::ReplacementEvidenceMismatch {
            expected: plan.replacement_content_sha256.clone(),
            actual: replacement,
        });
    }
    let evidence = plan_evidence_sha256(plan);
    if evidence != plan.evidence_sha256 {
        return Err(ApplyEditError::PlanEvidenceMismatch {
            expected: plan.evidence_sha256.clone(),
            actual: evidence,
        });
    }
    Ok(())
}

fn plan_evidence_sha256(plan: &ExactEditPlan) -> String {
    let canonical = format!(
        "{}\n{:?}\n{}\n{}\n{}\n{}",
        plan.path,
        plan.language,
        plan.expected_content_sha256,
        plan.replacement_content_sha256,
        plan.repository_evidence_sha256,
        plan.replacement_content
    );
    sha256_hex(canonical.as_bytes())
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use autonomy_governor::{
        canonical_filesystem_resource, ApprovalRequirement, ApprovalSigner, ApprovalVerifier,
        AuditLog, GovernedFileTransaction, Grant, Permission, Policy, Principal, PrincipalId,
        RiskLevel, Scope, ScopeKind,
    };
    use code_intelligence::{build_repository_index, Language, LexicalLanguageDriver};
    use code_intelligence_edits::plan_exact_file_replacement;

    use super::*;

    const KEY: &[u8; 32] = b"0123456789abcdef0123456789abcdef";

    fn temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "gameforge-code-apply-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(path.join("src")).expect("temp repository should be created");
        path
    }

    fn source(content: &str) -> SourceFile {
        SourceFile {
            path: "src/player.rs".to_owned(),
            language: Language::Rust,
            content: content.to_owned(),
            generated: false,
        }
    }

    fn index_for(source: &SourceFile) -> code_intelligence::RepositoryIndex {
        let rust = LexicalLanguageDriver::rust();
        build_repository_index(std::slice::from_ref(source), &[&rust]).expect("valid index")
    }

    fn policy_request(target: &Path, approval: ApprovalRequirement) -> (Policy, Request) {
        let principal = PrincipalId("code-intelligence-writer".to_owned());
        let scope = Scope {
            kind: ScopeKind::Filesystem,
            resource: canonical_filesystem_resource(target).expect("target should canonicalize"),
        };
        let policy = Policy {
            principals: vec![Principal {
                id: principal.clone(),
                role: "code-intelligence-test".to_owned(),
            }],
            grants: vec![Grant {
                principal: principal.clone(),
                permission: Permission::Write,
                scope: scope.clone(),
                approval,
            }],
        };
        let request = Request {
            principal,
            permission: Permission::Write,
            scope,
            risk: RiskLevel::Low,
        };
        (policy, request)
    }

    #[test]
    fn fresh_plan_commits_through_governor_and_rolls_back() {
        let root = temp_dir("commit");
        let target = root.join("src/player.rs");
        let current = source("fn player() {}\n");
        fs::write(&target, &current.content).expect("seed source should write");
        let index = index_for(&current);
        let plan = plan_exact_file_replacement(&index, &current, "fn player() { tick(); }\n")
            .expect("valid plan");
        let (policy, request) = policy_request(&target, ApprovalRequirement::None);
        let audit = AuditLog::new(root.join("audit.jsonl"));
        let runtime = GovernanceRuntime::new(&policy, &audit);

        let applied = apply_exact_file_replacement(
            &runtime,
            request,
            &root,
            &plan,
            &index.evidence_sha256,
            "code-edit",
        )
        .expect("fresh governed plan should commit");
        assert_eq!(
            fs::read_to_string(&target).expect("source should read"),
            plan.replacement_content
        );
        assert_eq!(applied.plan_evidence_sha256, plan.evidence_sha256);
        assert_eq!(audit.load_verified().expect("audit should verify").len(), 1);

        GovernedFileTransaction::rollback(&applied.receipt)
            .expect("rollback should restore original source");
        assert_eq!(
            fs::read_to_string(&target).expect("source should read"),
            current.content
        );
        fs::remove_dir_all(root).expect("temp repository should remove");
    }

    #[test]
    fn on_disk_drift_rejects_stale_plan_before_governed_write() {
        let root = temp_dir("stale");
        let target = root.join("src/player.rs");
        let current = source("fn player() {}\n");
        fs::write(&target, &current.content).expect("seed source should write");
        let index = index_for(&current);
        let plan = plan_exact_file_replacement(&index, &current, "fn player() { tick(); }\n")
            .expect("valid plan");
        fs::write(&target, "fn player() { changed_elsewhere(); }\n")
            .expect("external change should write");
        let (policy, request) = policy_request(&target, ApprovalRequirement::None);
        let audit = AuditLog::new(root.join("audit.jsonl"));
        let runtime = GovernanceRuntime::new(&policy, &audit);

        let error = apply_exact_file_replacement(
            &runtime,
            request,
            &root,
            &plan,
            &index.evidence_sha256,
            "stale-edit",
        )
        .expect_err("stale plan must fail closed");
        assert!(matches!(
            error,
            ApplyEditError::EditPlan(EditPlanError::StaleSource { .. })
        ));
        assert!(audit
            .load_verified()
            .expect("empty audit should verify")
            .is_empty());
        assert_eq!(
            fs::read_to_string(&target).expect("source should read"),
            "fn player() { changed_elsewhere(); }\n"
        );
        fs::remove_dir_all(root).expect("temp repository should remove");
    }

    #[test]
    fn tampered_replacement_content_is_rejected_before_runtime() {
        let root = temp_dir("tampered");
        let target = root.join("src/player.rs");
        let current = source("fn player() {}\n");
        fs::write(&target, &current.content).expect("seed source should write");
        let index = index_for(&current);
        let mut plan = plan_exact_file_replacement(&index, &current, "fn player() { tick(); }\n")
            .expect("valid plan");
        plan.replacement_content = "fn player() { injected(); }\n".to_owned();
        let (policy, request) = policy_request(&target, ApprovalRequirement::None);
        let audit = AuditLog::new(root.join("audit.jsonl"));
        let runtime = GovernanceRuntime::new(&policy, &audit);

        let error = apply_exact_file_replacement(
            &runtime,
            request,
            &root,
            &plan,
            &index.evidence_sha256,
            "tampered-edit",
        )
        .expect_err("tampered replacement must fail closed");
        assert!(matches!(
            error,
            ApplyEditError::ReplacementEvidenceMismatch { .. }
        ));
        assert_eq!(
            fs::read_to_string(&target).expect("source should read"),
            current.content
        );
        fs::remove_dir_all(root).expect("temp repository should remove");
    }

    #[test]
    fn repository_escape_path_is_rejected_even_with_recomputed_plan_digest() {
        let root = temp_dir("escape");
        let target = root.join("src/player.rs");
        let current = source("fn player() {}\n");
        fs::write(&target, &current.content).expect("seed source should write");
        let index = index_for(&current);
        let mut plan = plan_exact_file_replacement(&index, &current, "fn player() { tick(); }\n")
            .expect("valid plan");
        plan.path = "../outside.rs".to_owned();
        plan.evidence_sha256 = plan_evidence_sha256(&plan);
        let (policy, request) = policy_request(&target, ApprovalRequirement::None);
        let audit = AuditLog::new(root.join("audit.jsonl"));
        let runtime = GovernanceRuntime::new(&policy, &audit);

        let error = apply_exact_file_replacement(
            &runtime,
            request,
            &root,
            &plan,
            &index.evidence_sha256,
            "escape-edit",
        )
        .expect_err("repository escape must fail closed");
        assert!(matches!(error, ApplyEditError::NonCanonicalPlanPath(_)));
        fs::remove_dir_all(root).expect("temp repository should remove");
    }

    #[test]
    fn human_gate_cannot_be_bypassed_and_approved_path_succeeds() {
        let root = temp_dir("approval");
        let target = root.join("src/player.rs");
        let current = source("fn player() {}\n");
        fs::write(&target, &current.content).expect("seed source should write");
        let index = index_for(&current);
        let plan = plan_exact_file_replacement(&index, &current, "fn player() { tick(); }\n")
            .expect("valid plan");
        let (policy, request) = policy_request(&target, ApprovalRequirement::Human);
        let audit = AuditLog::new(root.join("audit.jsonl"));
        let runtime = GovernanceRuntime::new(&policy, &audit);

        let blocked = apply_exact_file_replacement(
            &runtime,
            request.clone(),
            &root,
            &plan,
            &index.evidence_sha256,
            "approval-blocked",
        )
        .expect_err("human gate must block direct path");
        assert!(matches!(blocked, ApplyEditError::Runtime(_)));
        assert_eq!(
            fs::read_to_string(&target).expect("source should read"),
            current.content
        );

        let signer = ApprovalSigner::new(KEY).expect("test key should be valid");
        let verifier = ApprovalVerifier::new(KEY).expect("test key should be valid");
        let proof = signer
            .issue(&request, "human-owner", "code-apply-nonce", 100, 200)
            .expect("approval proof should issue");
        let approval = ApprovalContext {
            proof: &proof,
            verifier: &verifier,
            now_unix: 150,
        };
        let applied = apply_exact_file_replacement_with_approval(
            &runtime,
            request,
            &root,
            &plan,
            &index.evidence_sha256,
            "approval-edit",
            approval,
        )
        .expect("valid human approval should permit governed edit");
        assert_eq!(
            fs::read_to_string(&target).expect("source should read"),
            plan.replacement_content
        );
        assert!(applied.receipt.audit_entry.record.approval.is_some());
        fs::remove_dir_all(root).expect("temp repository should remove");
    }
}
