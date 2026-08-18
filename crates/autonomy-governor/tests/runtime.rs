use std::{
    cell::Cell,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use autonomy_governor::{
    canonical_filesystem_resource, ApprovalContext, ApprovalRequirement, ApprovalSigner,
    ApprovalVerifier, AuditLog, GovernanceRuntime, Grant, Permission, Policy, Principal,
    PrincipalId, Request, RiskLevel, RuntimeError, Scope, ScopeKind,
};

const KEY: &[u8; 32] = b"0123456789abcdef0123456789abcdef";

fn temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "gameforge-runtime-{label}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("temp directory should be created");
    path
}

fn file_policy(target: &Path, approval: ApprovalRequirement) -> (Policy, Request) {
    let principal = PrincipalId("runtime-writer".to_owned());
    let scope = Scope {
        kind: ScopeKind::Filesystem,
        resource: canonical_filesystem_resource(target).expect("target should canonicalize"),
    };
    let policy = Policy {
        principals: vec![Principal {
            id: principal.clone(),
            role: "runtime-test".to_owned(),
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

fn tool_policy(approval: ApprovalRequirement) -> (Policy, Request) {
    let principal = PrincipalId("runtime-tool".to_owned());
    let scope = Scope {
        kind: ScopeKind::Tool,
        resource: "gamespec-compiler".to_owned(),
    };
    let policy = Policy {
        principals: vec![Principal {
            id: principal.clone(),
            role: "runtime-tool-test".to_owned(),
        }],
        grants: vec![Grant {
            principal: principal.clone(),
            permission: Permission::Execute,
            scope: scope.clone(),
            approval,
        }],
    };
    let request = Request {
        principal,
        permission: Permission::Execute,
        scope,
        risk: RiskLevel::Low,
    };
    (policy, request)
}

#[test]
fn canonical_scope_write_is_committed_and_reversible() {
    let dir = temp_dir("write");
    let target = dir.join("state.txt");
    fs::write(&target, b"before").expect("seed file should write");
    let audit = AuditLog::new(dir.join("audit.jsonl"));
    let (policy, request) = file_policy(&target, ApprovalRequirement::None);
    let runtime = GovernanceRuntime::new(&policy, &audit);

    let receipt = runtime
        .write_file(request, &target, b"after", "runtime-write")
        .expect("canonical governed write should commit");
    assert_eq!(fs::read(&target).expect("target should read"), b"after");

    autonomy_governor::GovernedFileTransaction::rollback(&receipt)
        .expect("rollback should restore preimage");
    assert_eq!(fs::read(&target).expect("target should read"), b"before");
    fs::remove_dir_all(dir).expect("temp directory should remove");
}

#[test]
fn relative_target_is_rejected_before_mutation() {
    let dir = temp_dir("relative");
    let target = dir.join("state.txt");
    let audit = AuditLog::new(dir.join("audit.jsonl"));
    let (policy, request) = file_policy(&target, ApprovalRequirement::None);
    let runtime = GovernanceRuntime::new(&policy, &audit);

    let error = runtime
        .write_file(request, Path::new("relative-state.txt"), b"blocked", "relative")
        .expect_err("relative target must fail closed");
    assert!(matches!(error, RuntimeError::AbsolutePathRequired(_)));
    assert!(!Path::new("relative-state.txt").exists());
    assert!(audit
        .load_verified()
        .expect("empty audit should verify")
        .is_empty());
    fs::remove_dir_all(dir).expect("temp directory should remove");
}

#[test]
fn lexical_alias_scope_is_rejected_even_when_policy_matches_alias() {
    let dir = temp_dir("alias");
    let target = dir.join("state.txt");
    let alias = dir.join(".").join("state.txt");
    let principal = PrincipalId("alias-writer".to_owned());
    let scope = Scope {
        kind: ScopeKind::Filesystem,
        resource: alias.to_string_lossy().into_owned(),
    };
    let request = Request {
        principal: principal.clone(),
        permission: Permission::Write,
        scope: scope.clone(),
        risk: RiskLevel::Low,
    };
    let policy = Policy {
        principals: vec![Principal {
            id: principal.clone(),
            role: "alias-test".to_owned(),
        }],
        grants: vec![Grant {
            principal,
            permission: Permission::Write,
            scope,
            approval: ApprovalRequirement::None,
        }],
    };
    let audit = AuditLog::new(dir.join("audit.jsonl"));
    let runtime = GovernanceRuntime::new(&policy, &audit);

    let error = runtime
        .write_file(request, &target, b"blocked", "alias")
        .expect_err("noncanonical scope must not authorize canonical target");
    assert!(matches!(error, RuntimeError::ScopeMismatch { .. }));
    assert!(!target.exists());
    fs::remove_dir_all(dir).expect("temp directory should remove");
}

#[cfg(unix)]
#[test]
fn symlink_ancestor_is_rejected_instead_of_followed() {
    use std::os::unix::fs::symlink;

    let dir = temp_dir("symlink-ancestor");
    let real = dir.join("real");
    let alias = dir.join("alias");
    fs::create_dir(&real).expect("real directory should exist");
    symlink(&real, &alias).expect("symlink should be created");
    let target = alias.join("state.txt");

    let error = canonical_filesystem_resource(&target)
        .expect_err("symlink ancestor must be rejected before policy construction");
    assert!(matches!(error, RuntimeError::SymlinkAncestor(_)));
    assert!(!real.join("state.txt").exists());
    fs::remove_dir_all(dir).expect("temp directory should remove");
}

#[test]
fn denied_runtime_request_is_audited_and_operation_is_not_invoked() {
    let dir = temp_dir("denied-execute");
    let audit = AuditLog::new(dir.join("audit.jsonl"));
    let (_, request) = tool_policy(ApprovalRequirement::None);
    let policy = Policy::default();
    let runtime = GovernanceRuntime::new(&policy, &audit);
    let called = Cell::new(false);

    let error = runtime
        .execute(request, None, |_| {
            called.set(true);
            Ok(())
        })
        .expect_err("denied request must not invoke operation");
    assert!(matches!(error, RuntimeError::Denied { .. }));
    assert!(!called.get());
    assert_eq!(audit.load_verified().expect("audit should verify").len(), 1);
    fs::remove_dir_all(dir).expect("temp directory should remove");
}

#[test]
fn direct_allow_runtime_request_invokes_operation_after_audit() {
    let dir = temp_dir("allow-execute");
    let audit = AuditLog::new(dir.join("audit.jsonl"));
    let (policy, request) = tool_policy(ApprovalRequirement::None);
    let runtime = GovernanceRuntime::new(&policy, &audit);
    let called = Cell::new(false);

    let result = runtime
        .execute(request.clone(), None, |permit| {
            assert_eq!(permit.request, request);
            called.set(true);
            Ok(42_u8)
        })
        .expect("allowed operation should run");
    assert_eq!(result, 42);
    assert!(called.get());
    assert_eq!(audit.load_verified().expect("audit should verify").len(), 1);
    fs::remove_dir_all(dir).expect("temp directory should remove");
}

#[test]
fn human_approval_is_required_before_runtime_operation() {
    let dir = temp_dir("approval-execute");
    let audit = AuditLog::new(dir.join("audit.jsonl"));
    let (policy, request) = tool_policy(ApprovalRequirement::Human);
    let runtime = GovernanceRuntime::new(&policy, &audit);
    let called = Cell::new(false);

    let missing = runtime
        .execute(request.clone(), None, |_| {
            called.set(true);
            Ok(())
        })
        .expect_err("missing human approval must block execution");
    assert!(matches!(missing, RuntimeError::MissingApproval));
    assert!(!called.get());

    let signer = ApprovalSigner::new(KEY).expect("test key should be valid");
    let verifier = ApprovalVerifier::new(KEY).expect("test key should be valid");
    let proof = signer
        .issue(&request, "human-owner", "runtime-nonce", 100, 200)
        .expect("proof should issue");
    let context = ApprovalContext {
        proof: &proof,
        verifier: &verifier,
        now_unix: 150,
    };

    runtime
        .execute(request, Some(context), |_| {
            called.set(true);
            Ok(())
        })
        .expect("approved operation should run");
    assert!(called.get());
    assert_eq!(audit.load_verified().expect("audit should verify").len(), 1);
    fs::remove_dir_all(dir).expect("temp directory should remove");
}
