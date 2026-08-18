//! Deny-by-default authorization policy core for NTPX `GameForge` Ω∞.
//!
//! This crate decides whether a requested capability is allowed, denied, or requires
//! explicit human approval. It deliberately performs no tool, filesystem, network,
//! credential, repository, or release action itself. The audit module may persist
//! authorization decision records, but it never executes the requested capability.

mod approval;
mod audit;

use std::{error::Error, fmt};

use serde::{Deserialize, Serialize};

pub use approval::{
    ApprovalError, ApprovalEvidence, ApprovalProof, ApprovalSigner, ApprovalVerifier,
};
pub use audit::{AuditEntry, AuditError, AuditLog};

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval: Option<ApprovalEvidence>,
}

/// Failure produced by the audited human-approval path.
#[derive(Debug)]
pub enum GovernanceError {
    MissingApprovalProof,
    Approval(ApprovalError),
    Audit(AuditError),
    ReplayDetected { nonce: String },
}

impl fmt::Display for GovernanceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingApprovalProof => write!(formatter, "human approval proof is required"),
            Self::Approval(error) => write!(formatter, "approval verification failed: {error}"),
            Self::Audit(error) => write!(formatter, "audit operation failed: {error}"),
            Self::ReplayDetected { nonce } => {
                write!(formatter, "approval nonce '{nonce}' was already consumed")
            }
        }
    }
}

impl Error for GovernanceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Approval(error) => Some(error),
            Self::Audit(error) => Some(error),
            Self::MissingApprovalProof | Self::ReplayDetected { .. } => None,
        }
    }
}

impl From<ApprovalError> for GovernanceError {
    fn from(error: ApprovalError) -> Self {
        Self::Approval(error)
    }
}

impl From<AuditError> for GovernanceError {
    fn from(error: AuditError) -> Self {
        Self::Audit(error)
    }
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

        DecisionRecord {
            request,
            decision,
            approval: None,
        }
    }

    /// Evaluate a request, verify any required human approval proof, reject durable nonce replay,
    /// and append the final decision to the verified audit log.
    ///
    /// A proof can only satisfy an existing `RequireHumanApproval` result. It can never override a
    /// denial or create a grant. This function records authorization evidence but performs no
    /// governed external capability itself.
    ///
    /// Replay prevention is durable for serialized writers because consumed nonces are recovered
    /// from the verified audit log. Cross-process concurrent-writer exclusion remains separate
    /// Phase 03 work.
    ///
    /// # Errors
    /// Returns [`GovernanceError`] when approval is missing, invalid, expired, replayed, or when
    /// audit persistence/integrity verification fails.
    pub fn evaluate_and_audit_with_approval(
        &self,
        request: Request,
        proof: Option<&ApprovalProof>,
        verifier: &ApprovalVerifier,
        audit_log: &AuditLog,
        now_unix: u64,
    ) -> Result<AuditEntry, GovernanceError> {
        let base = self.evaluate(request.clone());
        if !matches!(base.decision, Decision::RequireHumanApproval { .. }) {
            return audit_log.append(base).map_err(GovernanceError::Audit);
        }

        let proof = proof.ok_or(GovernanceError::MissingApprovalProof)?;
        let evidence = verifier.verify(&request, proof, now_unix)?;
        if audit_log.contains_approval_nonce(&evidence.nonce)? {
            return Err(GovernanceError::ReplayDetected {
                nonce: evidence.nonce,
            });
        }

        let approved = DecisionRecord {
            request,
            decision: Decision::Allow {
                reason: "matching grant with valid one-time human approval proof".to_owned(),
            },
            approval: Some(evidence),
        };
        audit_log.append(approved).map_err(GovernanceError::Audit)
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
