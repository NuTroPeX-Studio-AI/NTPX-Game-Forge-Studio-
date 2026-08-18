use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use autonomy_governor::{
    canonical_filesystem_resource, ApprovalRequirement, AuditLog, BatchFileWrite,
    BatchTransactionError, GovernanceRuntime, GovernedFileBatch, Grant, Permission, Policy,
    Principal, PrincipalId, Request, RiskLevel, Scope, ScopeKind,
};

fn temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "gameforge-batch-{label}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("temp directory should be created");
    path
}

fn request_for(principal: &PrincipalId, target: &Path) -> Request {
    Request {
        principal: principal.clone(),
        permission: Permission::Write,
        scope: Scope {
            kind: ScopeKind::Filesystem,
            resource: canonical_filesystem_resource(target).expect("target should canonicalize"),
        },
        risk: RiskLevel::Low,
    }
}

fn policy_for(principal: &PrincipalId, targets: &[&Path]) -> Policy {
    Policy {
        principals: vec![Principal {
            id: principal.clone(),
            role: "batch-test".to_owned(),
        }],
        grants: targets
            .iter()
            .map(|target| Grant {
                principal: principal.clone(),
                permission: Permission::Write,
                scope: Scope {
                    kind: ScopeKind::Filesystem,
                    resource: canonical_filesystem_resource(target)
                        .expect("target should canonicalize"),
                },
                approval: ApprovalRequirement::None,
            })
            .collect(),
    }
}

#[test]
fn two_file_batch_commits_and_explicit_rollback_restores_both() {
    let dir = temp_dir("commit");
    let first = dir.join("first.txt");
    let second = dir.join("second.txt");
    fs::write(&first, b"first-before").expect("first seed should write");
    fs::write(&second, b"second-before").expect("second seed should write");

    let principal = PrincipalId("batch-writer".to_owned());
    let policy = policy_for(&principal, &[&first, &second]);
    let audit = AuditLog::new(dir.join("audit.jsonl"));
    let runtime = GovernanceRuntime::new(&policy, &audit);
    let writes = vec![
        BatchFileWrite {
            request: request_for(&principal, &first),
            target: first.clone(),
            contents: b"first-after".to_vec(),
            transaction_id: "batch-first".to_owned(),
            approval: None,
        },
        BatchFileWrite {
            request: request_for(&principal, &second),
            target: second.clone(),
            contents: b"second-after".to_vec(),
            transaction_id: "batch-second".to_owned(),
            approval: None,
        },
    ];

    let receipt = GovernedFileBatch::commit(&runtime, "batch-commit", writes)
        .expect("authorized batch should commit");
    assert_eq!(receipt.receipts.len(), 2);
    assert_eq!(fs::read(&first).expect("first should read"), b"first-after");
    assert_eq!(
        fs::read(&second).expect("second should read"),
        b"second-after"
    );

    GovernedFileBatch::rollback(&receipt).expect("batch rollback should succeed");
    assert_eq!(
        fs::read(&first).expect("first should read"),
        b"first-before"
    );
    assert_eq!(
        fs::read(&second).expect("second should read"),
        b"second-before"
    );
    fs::remove_dir_all(dir).expect("temp directory should remove");
}

#[test]
fn later_denial_automatically_reverts_earlier_successful_write() {
    let dir = temp_dir("auto-rollback");
    let first = dir.join("first.txt");
    let second = dir.join("second.txt");
    fs::write(&first, b"first-before").expect("first seed should write");

    let principal = PrincipalId("batch-writer".to_owned());
    let policy = policy_for(&principal, &[&first]);
    let audit = AuditLog::new(dir.join("audit.jsonl"));
    let runtime = GovernanceRuntime::new(&policy, &audit);
    let writes = vec![
        BatchFileWrite {
            request: request_for(&principal, &first),
            target: first.clone(),
            contents: b"first-after".to_vec(),
            transaction_id: "rollback-first".to_owned(),
            approval: None,
        },
        BatchFileWrite {
            request: request_for(&principal, &second),
            target: second.clone(),
            contents: b"must-not-persist".to_vec(),
            transaction_id: "rollback-second".to_owned(),
            approval: None,
        },
    ];

    let error = GovernedFileBatch::commit(&runtime, "batch-auto-rollback", writes)
        .expect_err("second write must be denied");
    assert!(matches!(error, BatchTransactionError::Runtime(_)));
    assert_eq!(
        fs::read(&first).expect("first should be restored"),
        b"first-before"
    );
    assert!(!second.exists());
    fs::remove_dir_all(dir).expect("temp directory should remove");
}

#[test]
fn duplicate_canonical_target_is_rejected_before_any_mutation() {
    let dir = temp_dir("duplicate");
    let target = dir.join("same.txt");
    fs::write(&target, b"before").expect("seed should write");

    let principal = PrincipalId("batch-writer".to_owned());
    let policy = policy_for(&principal, &[&target]);
    let audit = AuditLog::new(dir.join("audit.jsonl"));
    let runtime = GovernanceRuntime::new(&policy, &audit);
    let request = request_for(&principal, &target);
    let writes = vec![
        BatchFileWrite {
            request: request.clone(),
            target: target.clone(),
            contents: b"one".to_vec(),
            transaction_id: "duplicate-one".to_owned(),
            approval: None,
        },
        BatchFileWrite {
            request,
            target: target.clone(),
            contents: b"two".to_vec(),
            transaction_id: "duplicate-two".to_owned(),
            approval: None,
        },
    ];

    let error = GovernedFileBatch::commit(&runtime, "batch-duplicate", writes)
        .expect_err("duplicate target must fail before mutation");
    assert!(matches!(error, BatchTransactionError::DuplicateTarget(_)));
    assert_eq!(fs::read(&target).expect("target should read"), b"before");
    assert!(audit
        .load_verified()
        .expect("empty audit should verify")
        .is_empty());
    fs::remove_dir_all(dir).expect("temp directory should remove");
}
