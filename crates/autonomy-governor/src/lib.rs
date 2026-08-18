//! Deny-by-default authorization policy core for NTPX GameForge Ω∞.
//!
//! This crate decides whether a requested capability is allowed, denied, or requires
//! explicit human approval. It deliberately performs no tool, filesystem, network,
//! credential, repository, or release action itself.

use serde::{Deserialize, Serialize};

/// Stable identity for an agent or human principal.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PrincipalId(pub String);

/// Principal registered with the governor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Principal {
    pub id: PrincipalId,
    pub role: String,
}

/// Resource boundary governed by a permission grant.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopeKind {
    Tool,
    Filesystem,
    Network,
    Credential,
    Repository,
    Release,
}

/// Exact resource scope. Wildcard matching is intentionally not supported in this increment.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Scope {
    pub kind: ScopeKind,
    pub resource: String,
}

/// Capability requested by a principal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    Read,
    Write,
    Execute,
    NetworkConnect,
    UseCredential,
    Merge,
    Publish,
    Destructive,
}

/// Risk assigned to a request before policy evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Elevated,
    High,
    Protected,
}

/// Approval requirement attached to a matching grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalRequirement {
    None,
    Human,
}

/// Explicit permission grant. Matching is exact on principal, permission, and scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grant {
    pub principal: PrincipalId,
    pub permission: Permission,
    pub scope: Scope,
    pub approval: ApprovalRequirement,
}

/// Requested capability submitted to the governor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request {
    pub principal: PrincipalId,
    pub permission: Permission,
    pub scope: Scope,
    pub risk: RiskLevel,
}

/// Deterministic authorization result. This is a decision only; it never executes the action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub enum Decision {
    Allow { reason: String },
    RequireHumanApproval { reason: String },
    Deny { reason: String },
}

/// Audit-ready record produced for every evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub request: Request,
    pub decision: Decision,
}

/// Immutable policy input used by the evaluator.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Policy {
    pub principals: Vec<Principal>,
    pub grants: Vec<Grant>,
}

impl Policy {
    /// Evaluate one request with least privilege and deny-by-default semantics.
    #[must_use]
    pub fn evaluate(&self, request: Request) -> DecisionRecord {
        let decision = if !self
            .principals
            .iter()
            .any(|principal| principal.id == request.principal)
        {
            Decision::Deny {
                reason: "unknown principal".to_owned(),
            }
        } else if let Some(grant) = self.grants.iter().find(|grant| {
            grant.principal == request.principal
                && grant.permission == request.permission
                && grant.scope == request.scope
        }) {
            evaluate_matching_grant(grant, &request)
        } else {
            Decision::Deny {
                reason: "no matching explicit grant".to_owned(),
            }
        };

        DecisionRecord { request, decision }
    }
}

fn evaluate_matching_grant(grant: &Grant, request: &Request) -> Decision {
    if is_protected(request) {
        return Decision::RequireHumanApproval {
            reason: "protected capability requires explicit human approval".to_owned(),
        };
    }

    match grant.approval {
        ApprovalRequirement::None => Decision::Allow {
            reason: "matching explicit grant".to_owned(),
        },
        ApprovalRequirement::Human => Decision::RequireHumanApproval {
            reason: "matching grant requires human approval".to_owned(),
        },
    }
}

fn is_protected(request: &Request) -> bool {
    request.risk == RiskLevel::Protected
        || matches!(
            request.permission,
            Permission::UseCredential
                | Permission::Merge
                | Permission::Publish
                | Permission::Destructive
        )
        || matches!(
            request.scope.kind,
            ScopeKind::Credential | ScopeKind::Release
        )
}
