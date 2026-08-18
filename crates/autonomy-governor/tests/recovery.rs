use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use autonomy_governor::{
    canonical_filesystem_resource, ApprovalRequirement, AuditLog, GovernanceRuntime, Grant,
    Permission, Policy, Principal, PrincipalId, RecoveryError, RecoveryIntegrity, RecoveryScanner,
    Request, RiskLevel, Scope, ScopeKind,
};

fn temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "gameforge-recovery-{label}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("temp directory should be created");
    path
}

fn policy_for(target: &Path) -> (Policy, Request) {
    let principal = PrincipalId("recovery-writer".to_owned());
    let scope = Scope {
        kind: ScopeKind::Filesystem,
        resource: canonical_filesystem_resource(target).expect("target should canonicalize"),
    };
    let policy = Policy {
        principals: vec![Principal {
            id: principal.clone(),
            role: "recovery-test".to_owned(),
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
fn scan_is_read_only_and_explicit_recovery_restores_verified_snapshot() {
    let dir = temp_dir("verified");
    let target = dir.join("state.txt");
    fs::write(&target, b"before").expect("seed should write");
    let audit = AuditLog::new(dir.join("audit.jsonl"));
    let (policy, request) = policy_for(&target);
    let runtime = GovernanceRuntime::new(&policy, &audit);

    runtime
        .write_file(request, &target, b"after", "recovery-tx")
        .expect("governed write should commit");
    let candidates = RecoveryScanner::scan_directory(&dir).expect("scan should succeed");
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].integrity, RecoveryIntegrity::Verified);
    assert_eq!(
        fs::read(&target).expect("scan must not mutate target"),
        b"after"
    );

    RecoveryScanner::recover(&candidates[0]).expect("explicit recovery should restore snapshot");
    assert_eq!(fs::read(&target).expect("target should read"), b"before");
    assert!(!candidates[0].snapshot_path.exists());
    fs::remove_dir_all(dir).expect("temp directory should remove");
}

#[test]
fn corrupt_snapshot_is_reported_and_never_recovered() {
    let dir = temp_dir("corrupt");
    let target = dir.join("state.txt");
    fs::write(&target, b"before").expect("seed should write");
    let audit = AuditLog::new(dir.join("audit.jsonl"));
    let (policy, request) = policy_for(&target);
    let runtime = GovernanceRuntime::new(&policy, &audit);

    let receipt = runtime
        .write_file(request, &target, b"after", "corrupt-tx")
        .expect("governed write should commit");
    let mut encoded = fs::read(&receipt.snapshot_path).expect("snapshot should read");
    let last = encoded.last_mut().expect("snapshot should not be empty");
    *last ^= 1;
    fs::write(&receipt.snapshot_path, encoded).expect("snapshot corruption should write");

    let candidates = RecoveryScanner::scan_directory(&dir).expect("scan should succeed");
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].integrity, RecoveryIntegrity::Corrupt);
    let error = RecoveryScanner::recover(&candidates[0])
        .expect_err("corrupt candidate must not be restored");
    assert!(matches!(error, RecoveryError::CorruptSnapshot(_)));
    assert_eq!(fs::read(&target).expect("target should read"), b"after");
    fs::remove_dir_all(dir).expect("temp directory should remove");
}

#[cfg(unix)]
#[test]
fn scanner_does_not_follow_symlink_directories() {
    use std::os::unix::fs::symlink;

    let dir = temp_dir("symlink-scan");
    let outside = temp_dir("outside");
    let target = outside.join("state.txt");
    fs::write(&target, b"before").expect("seed should write");
    let audit = AuditLog::new(outside.join("audit.jsonl"));
    let (policy, request) = policy_for(&target);
    let runtime = GovernanceRuntime::new(&policy, &audit);
    runtime
        .write_file(request, &target, b"after", "outside-tx")
        .expect("governed write should commit");

    symlink(&outside, dir.join("linked-outside")).expect("directory symlink should create");
    let candidates = RecoveryScanner::scan_directory(&dir).expect("scan should succeed");
    assert!(candidates.is_empty());

    fs::remove_dir_all(dir).expect("scan directory should remove");
    fs::remove_dir_all(outside).expect("outside directory should remove");
}
