use std::{fs, path::PathBuf, process, time::SystemTime};

use autonomy_governor::{
    ApprovalError, ApprovalRequirement, ApprovalSigner, ApprovalVerifier, AuditLog, Decision,
    GovernanceError, Grant, Permission, Policy, Principal, PrincipalId, Request, RiskLevel, Scope,
    ScopeKind,
};

const KEY: &[u8] = b"gameforge-phase03-test-key-000001";

fn unique_path(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("system clock should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "gameforge-approval-{label}-{}-{nanos}.jsonl",
        process::id()
    ))
}

fn principal() -> Principal {
    Principal {
        id: PrincipalId("agent.builder".to_owned()),
        role: "builder".to_owned(),
    }
}

fn protected_request(resource: &str) -> Request {
    Request {
        principal: principal().id,
        permission: Permission::Publish,
        scope: Scope {
            kind: ScopeKind::Release,
            resource: resource.to_owned(),
        },
        risk: RiskLevel::Protected,
    }
}

fn policy_for(request: &Request) -> Policy {
    Policy {
        principals: vec![principal()],
        grants: vec![Grant {
            principal: request.principal.clone(),
            permission: request.permission,
            scope: request.scope.clone(),
            approval: ApprovalRequirement::None,
        }],
    }
}

#[test]
fn exact_proof_allows_human_gated_request_and_persists_evidence() {
    let path = unique_path("allow");
    let log = AuditLog::new(&path);
    let request = protected_request("gameforge-staging");
    let policy = policy_for(&request);
    let signer = ApprovalSigner::new(KEY).expect("test key is strong");
    let verifier = ApprovalVerifier::new(KEY).expect("test key is strong");
    let proof = signer
        .issue(&request, "human.nutropex", "nonce-001", 1_000, 1_100)
        .expect("proof is issued");

    let entry = policy
        .evaluate_and_audit_with_approval(request, Some(&proof), &verifier, &log, 1_050)
        .expect("valid proof authorizes the gated request");

    assert!(matches!(entry.record.decision, Decision::Allow { .. }));
    let evidence = entry
        .record
        .approval
        .as_ref()
        .expect("approval evidence is persisted");
    assert_eq!(evidence.authority, "human.nutropex");
    assert_eq!(evidence.nonce, "nonce-001");
    assert!(log
        .contains_approval_nonce("nonce-001")
        .expect("verified log can find nonce"));

    fs::remove_file(path).expect("temporary audit file removed");
}

#[test]
fn consumed_nonce_cannot_be_replayed_after_reload() {
    let path = unique_path("replay");
    let request = protected_request("gameforge-staging");
    let policy = policy_for(&request);
    let signer = ApprovalSigner::new(KEY).expect("test key is strong");
    let verifier = ApprovalVerifier::new(KEY).expect("test key is strong");
    let proof = signer
        .issue(&request, "human.nutropex", "nonce-replay", 2_000, 2_100)
        .expect("proof is issued");

    AuditLog::new(&path)
        .append(policy.evaluate(request.clone()))
        .expect("preapproval decision is audited");
    policy
        .evaluate_and_audit_with_approval(
            request.clone(),
            Some(&proof),
            &verifier,
            &AuditLog::new(&path),
            2_050,
        )
        .expect("first approval use succeeds");

    let result = policy.evaluate_and_audit_with_approval(
        request,
        Some(&proof),
        &verifier,
        &AuditLog::new(&path),
        2_060,
    );
    assert!(matches!(
        result,
        Err(GovernanceError::ReplayDetected { .. })
    ));

    fs::remove_file(path).expect("temporary audit file removed");
}

#[test]
fn proof_is_bound_to_the_exact_request() {
    let path = unique_path("binding");
    let approved_request = protected_request("gameforge-staging");
    let changed_request = protected_request("gameforge-production");
    let policy = policy_for(&changed_request);
    let signer = ApprovalSigner::new(KEY).expect("test key is strong");
    let verifier = ApprovalVerifier::new(KEY).expect("test key is strong");
    let proof = signer
        .issue(
            &approved_request,
            "human.nutropex",
            "nonce-binding",
            3_000,
            3_100,
        )
        .expect("proof is issued");

    let result = policy.evaluate_and_audit_with_approval(
        changed_request,
        Some(&proof),
        &verifier,
        &AuditLog::new(&path),
        3_050,
    );
    assert!(matches!(
        result,
        Err(GovernanceError::Approval(
            ApprovalError::RequestDigestMismatch
        ))
    ));
    assert!(
        !path.exists(),
        "invalid proof must not create an audit file"
    );
}

#[test]
fn expired_and_future_proofs_fail_closed() {
    let request = protected_request("gameforge-staging");
    let signer = ApprovalSigner::new(KEY).expect("test key is strong");
    let verifier = ApprovalVerifier::new(KEY).expect("test key is strong");
    let proof = signer
        .issue(&request, "human.nutropex", "nonce-time", 4_000, 4_100)
        .expect("proof is issued");

    assert!(matches!(
        verifier.verify(&request, &proof, 3_999),
        Err(ApprovalError::NotYetValid)
    ));
    assert!(matches!(
        verifier.verify(&request, &proof, 4_100),
        Err(ApprovalError::Expired)
    ));
}

#[test]
fn tampered_tag_is_rejected() {
    let request = protected_request("gameforge-staging");
    let signer = ApprovalSigner::new(KEY).expect("test key is strong");
    let verifier = ApprovalVerifier::new(KEY).expect("test key is strong");
    let mut proof = signer
        .issue(&request, "human.nutropex", "nonce-tag", 5_000, 5_100)
        .expect("proof is issued");
    proof.tag.replace_range(..2, "00");

    assert!(matches!(
        verifier.verify(&request, &proof, 5_050),
        Err(ApprovalError::InvalidTag)
    ));
}

#[test]
fn proof_cannot_override_policy_denial() {
    let path = unique_path("deny");
    let request = protected_request("gameforge-staging");
    let denied_policy = Policy {
        principals: vec![principal()],
        grants: Vec::new(),
    };
    let signer = ApprovalSigner::new(KEY).expect("test key is strong");
    let verifier = ApprovalVerifier::new(KEY).expect("test key is strong");
    let proof = signer
        .issue(&request, "human.nutropex", "nonce-deny", 6_000, 6_100)
        .expect("proof is issued");

    let entry = denied_policy
        .evaluate_and_audit_with_approval(
            request,
            Some(&proof),
            &verifier,
            &AuditLog::new(&path),
            6_050,
        )
        .expect("policy denial itself is auditable");
    assert!(matches!(entry.record.decision, Decision::Deny { .. }));
    assert!(entry.record.approval.is_none());
    assert!(!AuditLog::new(&path)
        .contains_approval_nonce("nonce-deny")
        .expect("audit log verifies"));

    fs::remove_file(path).expect("temporary audit file removed");
}

#[test]
fn missing_proof_and_weak_keys_are_rejected() {
    let path = unique_path("missing");
    let request = protected_request("gameforge-staging");
    let policy = policy_for(&request);
    let verifier = ApprovalVerifier::new(KEY).expect("test key is strong");

    assert!(matches!(
        policy.evaluate_and_audit_with_approval(
            request,
            None,
            &verifier,
            &AuditLog::new(&path),
            7_000,
        ),
        Err(GovernanceError::MissingApprovalProof)
    ));
    assert!(matches!(
        ApprovalSigner::new(b"short"),
        Err(ApprovalError::WeakKey { .. })
    ));
    assert!(!path.exists());
}
