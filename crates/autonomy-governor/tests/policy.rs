use autonomy_governor::{
    ApprovalRequirement, Decision, Grant, Permission, Policy, Principal, PrincipalId, Request,
    RiskLevel, Scope, ScopeKind,
};

fn principal() -> Principal {
    Principal {
        id: PrincipalId("agent.builder".to_owned()),
        role: "builder".to_owned(),
    }
}

fn scope(kind: ScopeKind, resource: &str) -> Scope {
    Scope {
        kind,
        resource: resource.to_owned(),
    }
}

fn request(permission: Permission, request_scope: Scope, risk: RiskLevel) -> Request {
    Request {
        principal: principal().id,
        permission,
        scope: request_scope,
        risk,
    }
}

#[test]
fn default_policy_denies_unknown_principal() {
    let record = Policy::default().evaluate(request(
        Permission::Read,
        scope(ScopeKind::Filesystem, "workspace/src"),
        RiskLevel::Low,
    ));

    assert!(matches!(record.decision, Decision::Deny { .. }));
}

#[test]
fn registered_principal_without_grant_is_denied() {
    let policy = Policy {
        principals: vec![principal()],
        grants: Vec::new(),
    };

    let record = policy.evaluate(request(
        Permission::Execute,
        scope(ScopeKind::Tool, "cargo-test"),
        RiskLevel::Low,
    ));

    assert!(matches!(record.decision, Decision::Deny { .. }));
}

#[test]
fn exact_low_risk_grant_is_allowed() {
    let allowed_scope = scope(ScopeKind::Tool, "cargo-test");
    let policy = Policy {
        principals: vec![principal()],
        grants: vec![Grant {
            principal: principal().id,
            permission: Permission::Execute,
            scope: allowed_scope.clone(),
            approval: ApprovalRequirement::None,
        }],
    };

    let record = policy.evaluate(request(Permission::Execute, allowed_scope, RiskLevel::Low));

    assert!(matches!(record.decision, Decision::Allow { .. }));
}

#[test]
fn grant_does_not_escape_its_exact_scope() {
    let policy = Policy {
        principals: vec![principal()],
        grants: vec![Grant {
            principal: principal().id,
            permission: Permission::Read,
            scope: scope(ScopeKind::Filesystem, "workspace/src"),
            approval: ApprovalRequirement::None,
        }],
    };

    let record = policy.evaluate(request(
        Permission::Read,
        scope(ScopeKind::Filesystem, "workspace/.env"),
        RiskLevel::Low,
    ));

    assert!(matches!(record.decision, Decision::Deny { .. }));
}

#[test]
fn explicit_human_gate_is_preserved() {
    let repository_scope = scope(ScopeKind::Repository, "NuTroPeX/NTPX-Game-Forge-Studio-");
    let policy = Policy {
        principals: vec![principal()],
        grants: vec![Grant {
            principal: principal().id,
            permission: Permission::Write,
            scope: repository_scope.clone(),
            approval: ApprovalRequirement::Human,
        }],
    };

    let record = policy.evaluate(request(
        Permission::Write,
        repository_scope,
        RiskLevel::Elevated,
    ));

    assert!(matches!(
        record.decision,
        Decision::RequireHumanApproval { .. }
    ));
}

#[test]
fn protected_capability_requires_human_even_when_grant_says_none() {
    let release_scope = scope(ScopeKind::Release, "gameforge-production");
    let policy = Policy {
        principals: vec![principal()],
        grants: vec![Grant {
            principal: principal().id,
            permission: Permission::Publish,
            scope: release_scope.clone(),
            approval: ApprovalRequirement::None,
        }],
    };

    let record = policy.evaluate(request(Permission::Publish, release_scope, RiskLevel::Low));

    assert!(matches!(
        record.decision,
        Decision::RequireHumanApproval { .. }
    ));
}

#[test]
fn decision_record_serialization_is_deterministic() {
    let policy = Policy {
        principals: vec![principal()],
        grants: Vec::new(),
    };
    let requested = request(
        Permission::NetworkConnect,
        scope(ScopeKind::Network, "api.example.invalid:443"),
        RiskLevel::Elevated,
    );

    let first = serde_json::to_string(&policy.evaluate(requested.clone())).expect("record encodes");
    let second = serde_json::to_string(&policy.evaluate(requested)).expect("record encodes");

    assert_eq!(first, second);
}
