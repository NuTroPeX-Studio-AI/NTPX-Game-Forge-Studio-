use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use autonomy_governor::{
    ApprovalRequirement, AuditLog, GovernedFileTransaction, Grant, Permission, Policy, Principal,
    PrincipalId, Request, RiskLevel, Scope, ScopeKind, TransactionError,
};

fn temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "gameforge-transaction-{label}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("temp directory should be created");
    path
}

fn policy_for(target: &Path) -> (Policy, Request) {
    let principal = PrincipalId("writer".to_owned());
    let scope = Scope {
        kind: ScopeKind::Filesystem,
        resource: target.to_string_lossy().into_owned(),
    };
    let policy = Policy {
        principals: vec![Principal {
            id: principal.clone(),
            role: "transaction-test".to_owned(),
        }],
        grants: vec![Grant {
            principal: principal.clone(),
            permission: Permission::Write,
            scope: scope.clone(),
            approval: ApprovalRequirement::None,
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
fn authorized_existing_file_commit_and_rollback_are_reversible() {
    let dir = temp_dir("existing");
    let target = dir.join("state.json");
    fs::write(&target, b"before").expect("seed file should write");
    let audit = AuditLog::new(dir.join("audit.jsonl"));
    let (policy, request) = policy_for(&target);

    let receipt =
        GovernedFileTransaction::commit(&policy, request, &target, b"after", "tx-existing", &audit)
            .expect("authorized transaction should commit");

    assert_eq!(fs::read(&target).expect("target should exist"), b"after");
    assert!(receipt.snapshot_path.exists());
    assert_eq!(audit.load_verified().expect("audit should verify").len(), 1);

    GovernedFileTransaction::rollback(&receipt).expect("rollback should restore preimage");
    assert_eq!(fs::read(&target).expect("target should exist"), b"before");
    assert!(!receipt.snapshot_path.exists());
    fs::remove_dir_all(dir).expect("temp directory should remove");
}

#[test]
fn authorized_new_file_rollback_removes_created_file() {
    let dir = temp_dir("new-file");
    let target = dir.join("created.txt");
    let audit = AuditLog::new(dir.join("audit.jsonl"));
    let (policy, request) = policy_for(&target);

    let receipt =
        GovernedFileTransaction::commit(&policy, request, &target, b"created", "tx-create", &audit)
            .expect("authorized create should commit");
    assert_eq!(
        fs::read(&target).expect("created target should exist"),
        b"created"
    );

    GovernedFileTransaction::rollback(&receipt).expect("rollback should remove created file");
    assert!(!target.exists());
    fs::remove_dir_all(dir).expect("temp directory should remove");
}

#[test]
fn denied_request_creates_no_file_snapshot_or_audit_entry() {
    let dir = temp_dir("denied");
    let target = dir.join("denied.txt");
    let audit = AuditLog::new(dir.join("audit.jsonl"));
    let (_, request) = policy_for(&target);
    let policy = Policy::default();

    let error =
        GovernedFileTransaction::commit(&policy, request, &target, b"blocked", "tx-denied", &audit)
            .expect_err("unknown principal must be denied");

    assert!(matches!(error, TransactionError::Unauthorized { .. }));
    assert!(!target.exists());
    assert!(audit
        .load_verified()
        .expect("empty audit should verify")
        .is_empty());
    fs::remove_dir_all(dir).expect("temp directory should remove");
}

#[test]
fn scope_mismatch_fails_before_mutation() {
    let dir = temp_dir("scope");
    let target = dir.join("target.txt");
    let other = dir.join("other.txt");
    let audit = AuditLog::new(dir.join("audit.jsonl"));
    let (policy, request) = policy_for(&other);

    let error =
        GovernedFileTransaction::commit(&policy, request, &target, b"blocked", "tx-scope", &audit)
            .expect_err("scope mismatch must fail");

    assert!(matches!(error, TransactionError::ScopeMismatch { .. }));
    assert!(!target.exists());
    fs::remove_dir_all(dir).expect("temp directory should remove");
}

#[test]
fn audit_failure_automatically_restores_prewrite_state() {
    let dir = temp_dir("audit-failure");
    let target = dir.join("state.txt");
    fs::write(&target, b"before").expect("seed file should write");
    let audit_path = dir.join("audit.jsonl");
    fs::write(&audit_path, b"not-json\n").expect("corrupt audit should write");
    let audit = AuditLog::new(audit_path);
    let (policy, request) = policy_for(&target);

    let error = GovernedFileTransaction::commit(
        &policy,
        request,
        &target,
        b"after",
        "tx-audit-failure",
        &audit,
    )
    .expect_err("corrupt audit must fail closed");

    assert!(matches!(error, TransactionError::Audit(_)));
    assert_eq!(
        fs::read(&target).expect("target should be restored"),
        b"before"
    );
    fs::remove_dir_all(dir).expect("temp directory should remove");
}

#[test]
fn human_approval_gate_is_not_bypassed_by_transaction_engine() {
    let dir = temp_dir("approval-gate");
    let target = dir.join("protected.txt");
    let audit = AuditLog::new(dir.join("audit.jsonl"));
    let (mut policy, mut request) = policy_for(&target);
    policy.grants[0].approval = ApprovalRequirement::Human;
    request.risk = RiskLevel::High;

    let error = GovernedFileTransaction::commit(
        &policy,
        request,
        &target,
        b"blocked",
        "tx-approval",
        &audit,
    )
    .expect_err("transaction engine must not bypass human approval gate");

    assert!(matches!(error, TransactionError::Unauthorized { .. }));
    assert!(!target.exists());
    fs::remove_dir_all(dir).expect("temp directory should remove");
}

#[cfg(unix)]
#[test]
fn symlink_ancestor_is_rejected_before_mutation() {
    use std::os::unix::fs::symlink;

    let dir = temp_dir("symlink-ancestor");
    let real_parent = dir.join("real");
    let linked_parent = dir.join("linked");
    fs::create_dir_all(&real_parent).expect("real parent should be created");
    symlink(&real_parent, &linked_parent).expect("symlink parent should be created");

    let target = linked_parent.join("state.txt");
    let real_target = real_parent.join("state.txt");
    let audit = AuditLog::new(dir.join("audit.jsonl"));
    let (policy, request) = policy_for(&target);

    let error = GovernedFileTransaction::commit(
        &policy,
        request,
        &target,
        b"blocked",
        "tx-symlink-ancestor",
        &audit,
    )
    .expect_err("symlink ancestor must fail closed");

    assert!(matches!(
        error,
        TransactionError::SymlinkAncestor(ref path) if path == &linked_parent
    ));
    assert!(!real_target.exists());
    assert!(audit
        .load_verified()
        .expect("empty audit should verify")
        .is_empty());
    fs::remove_dir_all(dir).expect("temp directory should remove");
}
