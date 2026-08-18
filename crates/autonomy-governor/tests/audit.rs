use std::{fs, path::PathBuf, process, time::SystemTime};

use autonomy_governor::{
    AuditError, AuditLog, Decision, Permission, Policy, Principal, PrincipalId, Request, RiskLevel,
    Scope, ScopeKind,
};

fn unique_path(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("system clock should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "gameforge-autonomy-audit-{label}-{}-{nanos}.jsonl",
        process::id()
    ))
}

fn record(resource: &str) -> autonomy_governor::DecisionRecord {
    let principal = Principal {
        id: PrincipalId("agent.builder".to_owned()),
        role: "builder".to_owned(),
    };
    let policy = Policy {
        principals: vec![principal.clone()],
        grants: Vec::new(),
    };

    policy.evaluate(Request {
        principal: principal.id,
        permission: Permission::Read,
        scope: Scope {
            kind: ScopeKind::Filesystem,
            resource: resource.to_owned(),
        },
        risk: RiskLevel::Low,
    })
}

#[test]
fn append_and_load_round_trip_preserves_chain() {
    let path = unique_path("roundtrip");
    let log = AuditLog::new(&path);

    let first = log
        .append(record("workspace/src"))
        .expect("first append succeeds");
    let second = log
        .append(record("workspace/tests"))
        .expect("second append succeeds");
    let loaded = log.load_verified().expect("audit log verifies");

    assert_eq!(first.sequence, 0);
    assert_eq!(first.previous_hash, None);
    assert_eq!(second.sequence, 1);
    assert_eq!(second.previous_hash.as_deref(), Some(first.hash.as_str()));
    assert_eq!(loaded, vec![first, second]);

    fs::remove_file(path).expect("temporary audit file removed");
}

#[test]
fn tampering_is_detected_and_fails_closed() {
    let path = unique_path("tamper");
    let log = AuditLog::new(&path);
    log.append(record("workspace/src"))
        .expect("audit append succeeds");

    let contents = fs::read_to_string(&path).expect("audit file readable");
    let tampered = contents.replace("unknown principal", "matching explicit grant");
    fs::write(&path, tampered).expect("test mutation succeeds");

    assert!(matches!(
        log.load_verified(),
        Err(AuditError::Integrity { .. })
    ));

    fs::remove_file(path).expect("temporary audit file removed");
}

#[test]
fn malformed_entry_is_rejected() {
    let path = unique_path("malformed");
    fs::write(&path, "{not-json}\n").expect("test fixture written");
    let log = AuditLog::new(&path);

    assert!(matches!(
        log.load_verified(),
        Err(AuditError::Decode { line: 1, .. })
    ));

    fs::remove_file(path).expect("temporary audit file removed");
}

#[test]
fn existing_corruption_prevents_append() {
    let path = unique_path("append-corrupt");
    let log = AuditLog::new(&path);
    let entry = log
        .append(record("workspace/src"))
        .expect("append succeeds");

    let contents = fs::read_to_string(&path).expect("audit file readable");
    let tampered = contents.replace(&entry.hash, &"0".repeat(64));
    fs::write(&path, tampered).expect("test mutation succeeds");

    let result = log.append(record("workspace/tests"));
    assert!(matches!(result, Err(AuditError::Integrity { .. })));

    fs::remove_file(path).expect("temporary audit file removed");
}

#[test]
fn missing_file_is_an_empty_verified_log() {
    let path = unique_path("missing");
    let log = AuditLog::new(&path);

    assert!(log
        .load_verified()
        .expect("missing log is empty")
        .is_empty());
    assert!(!path.exists());
}

#[test]
fn stored_decision_is_not_reinterpreted() {
    let path = unique_path("decision");
    let log = AuditLog::new(&path);
    let denied = record("workspace/src");
    assert!(matches!(denied.decision, Decision::Deny { .. }));

    log.append(denied.clone()).expect("append succeeds");
    let loaded = log.load_verified().expect("audit log verifies");

    assert_eq!(loaded[0].record, denied);

    fs::remove_file(path).expect("temporary audit file removed");
}
