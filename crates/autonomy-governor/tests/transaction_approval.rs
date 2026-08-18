use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use autonomy_governor::{
    ApprovalContext, ApprovalRequirement, ApprovalSigner, ApprovalVerifier, AuditLog,
    GovernedFileTransaction, Grant, Permission, Policy, Principal, PrincipalId, Request, RiskLevel,
    Scope, ScopeKind, TransactionError,
};

const KEY: &[u8; 32] = b"0123456789abcdef0123456789abcdef";

fn temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "gameforge-approved-transaction-{label}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("temp directory should be created");
    path
}

fn approval_policy(target: &Path) -> (Policy, Request) {
    let principal = PrincipalId("approved-writer".to_owned());
    let scope = Scope {
        kind: ScopeKind::Filesystem,
        resource: target.to_string_lossy().into_owned(),
    };
    let policy = Policy {
        principals: vec![Principal {
            id: principal.clone(),
            role: "approval-transaction-test".to_owned(),
        }],
        grants: vec![Grant {
            principal: principal.clone(),
            permission: Permission::Write,
            scope: scope.clone(),
            approval: ApprovalRequirement::Human,
        }],
    };
    let request = Request {
        principal,
        permission: Permission::Write,
        scope,
        risk: RiskLevel::High,
    };
    (policy, request)
}

fn approval_context<'a>(
    proof: &'a autonomy_governor::ApprovalProof,
    verifier: &'a ApprovalVerifier,
    now_unix: u64,
) -> ApprovalContext<'a> {
    ApprovalContext {
        proof,
        verifier,
        now_unix,
    }
}

#[test]
fn valid_human_approval_commits_and_persists_evidence() {
    let dir = temp_dir("valid");
    let target = dir.join("state.txt");
    fs::write(&target, b"before").expect("seed file should write");
    let audit = AuditLog::new(dir.join("audit.jsonl"));
    let (policy, request) = approval_policy(&target);
    let signer = ApprovalSigner::new(KEY).expect("test key should be valid");
    let verifier = ApprovalVerifier::new(KEY).expect("test key should be valid");
    let proof = signer
        .issue(&request, "human-owner", "nonce-approved-1", 100, 200)
        .expect("proof should issue");

    let receipt = GovernedFileTransaction::commit_with_approval(
        &policy,
        request,
        &target,
        b"after",
        "tx-approved",
        approval_context(&proof, &verifier, 150),
        &audit,
    )
    .expect("approved transaction should commit");

    assert_eq!(fs::read(&target).expect("target should exist"), b"after");
    let approval = receipt
        .audit_entry
        .record
        .approval
        .as_ref()
        .expect("approval evidence should persist");
    assert_eq!(approval.authority, "human-owner");
    assert_eq!(approval.nonce, "nonce-approved-1");
    assert_eq!(audit.load_verified().expect("audit should verify").len(), 1);

    GovernedFileTransaction::rollback(&receipt).expect("rollback should restore preimage");
    assert_eq!(fs::read(&target).expect("target should exist"), b"before");
    fs::remove_dir_all(dir).expect("temp directory should remove");
}

#[test]
fn replayed_approval_fails_closed_and_restores_winner_state() {
    let dir = temp_dir("replay");
    let target = dir.join("state.txt");
    fs::write(&target, b"before").expect("seed file should write");
    let audit = AuditLog::new(dir.join("audit.jsonl"));
    let (policy, request) = approval_policy(&target);
    let signer = ApprovalSigner::new(KEY).expect("test key should be valid");
    let verifier = ApprovalVerifier::new(KEY).expect("test key should be valid");
    let proof = signer
        .issue(&request, "human-owner", "nonce-replay-1", 100, 200)
        .expect("proof should issue");

    GovernedFileTransaction::commit_with_approval(
        &policy,
        request.clone(),
        &target,
        b"winner",
        "tx-winner",
        approval_context(&proof, &verifier, 150),
        &audit,
    )
    .expect("first approved transaction should commit");

    let error = GovernedFileTransaction::commit_with_approval(
        &policy,
        request,
        &target,
        b"replay",
        "tx-replay",
        approval_context(&proof, &verifier, 150),
        &audit,
    )
    .expect_err("replayed proof must fail");

    assert!(matches!(
        error,
        TransactionError::ReplayDetected { nonce } if nonce == "nonce-replay-1"
    ));
    assert_eq!(fs::read(&target).expect("target should remain"), b"winner");
    assert_eq!(audit.load_verified().expect("audit should verify").len(), 1);
    fs::remove_dir_all(dir).expect("temp directory should remove");
}

#[test]
fn mismatched_or_expired_proof_never_mutates_target() {
    let dir = temp_dir("invalid-proof");
    let target = dir.join("state.txt");
    let other = dir.join("other.txt");
    fs::write(&target, b"before").expect("seed file should write");
    let audit = AuditLog::new(dir.join("audit.jsonl"));
    let (policy, request) = approval_policy(&target);
    let (_, other_request) = approval_policy(&other);
    let signer = ApprovalSigner::new(KEY).expect("test key should be valid");
    let verifier = ApprovalVerifier::new(KEY).expect("test key should be valid");
    let mismatched = signer
        .issue(&other_request, "human-owner", "nonce-mismatch", 100, 200)
        .expect("proof should issue");

    let mismatch = GovernedFileTransaction::commit_with_approval(
        &policy,
        request.clone(),
        &target,
        b"blocked",
        "tx-mismatch",
        approval_context(&mismatched, &verifier, 150),
        &audit,
    )
    .expect_err("mismatched proof must fail");
    assert!(matches!(mismatch, TransactionError::Approval(_)));
    assert_eq!(fs::read(&target).expect("target should exist"), b"before");

    let expired = signer
        .issue(&request, "human-owner", "nonce-expired", 100, 120)
        .expect("proof should issue");
    let expired_error = GovernedFileTransaction::commit_with_approval(
        &policy,
        request,
        &target,
        b"blocked",
        "tx-expired",
        approval_context(&expired, &verifier, 120),
        &audit,
    )
    .expect_err("expired proof must fail");
    assert!(matches!(expired_error, TransactionError::Approval(_)));
    assert_eq!(fs::read(&target).expect("target should exist"), b"before");
    assert!(audit
        .load_verified()
        .expect("empty audit should verify")
        .is_empty());
    fs::remove_dir_all(dir).expect("temp directory should remove");
}

#[test]
fn approval_api_cannot_create_authority_for_direct_allow_or_denial() {
    let dir = temp_dir("no-escalation");
    let target = dir.join("state.txt");
    let audit = AuditLog::new(dir.join("audit.jsonl"));
    let (mut policy, request) = approval_policy(&target);
    let signer = ApprovalSigner::new(KEY).expect("test key should be valid");
    let verifier = ApprovalVerifier::new(KEY).expect("test key should be valid");
    let proof = signer
        .issue(&request, "human-owner", "nonce-no-escalation", 100, 200)
        .expect("proof should issue");

    policy.grants[0].approval = ApprovalRequirement::None;
    let direct_allow_error = GovernedFileTransaction::commit_with_approval(
        &policy,
        request.clone(),
        &target,
        b"blocked",
        "tx-direct-allow",
        approval_context(&proof, &verifier, 150),
        &audit,
    )
    .expect_err("approval API should require a real approval gate");
    assert!(matches!(
        direct_allow_error,
        TransactionError::Unauthorized { .. }
    ));

    let denial_error = GovernedFileTransaction::commit_with_approval(
        &Policy::default(),
        request,
        &target,
        b"blocked",
        "tx-denied",
        approval_context(&proof, &verifier, 150),
        &audit,
    )
    .expect_err("approval proof must not override policy denial");
    assert!(matches!(
        denial_error,
        TransactionError::Unauthorized { .. }
    ));
    assert!(!target.exists());
    fs::remove_dir_all(dir).expect("temp directory should remove");
}
