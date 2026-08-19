//! Governed Agent Zero execution contracts for NTPX `GameForge` Ω∞.
//!
//! This phase boundary requires an Autonomy Governor `Allow` decision before an execution adapter
//! can receive a dispatch permit. The crate itself does not spawn processes or access credentials.

use std::{error::Error, fmt};

use autonomy_governor::{Decision, DecisionRecord, Permission, PrincipalId, Scope, ScopeKind};
use serde::{Deserialize, Serialize};

/// Stable execution request envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub request_id: String,
    pub principal: PrincipalId,
    pub tool_id: String,
    pub args: Vec<String>,
    pub working_directory: String,
    pub timeout_ms: u64,
    pub max_memory_bytes: u64,
    pub max_output_bytes: u64,
}

/// Dispatch permit produced only after exact governor evidence validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPermit {
    pub request: ExecutionRequest,
    pub authorization: DecisionRecord,
}

/// Normalized execution result contract returned by concrete drivers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub request_id: String,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
    pub cancelled: bool,
    pub output_truncated: bool,
}

/// Concrete runtime adapter contract. Implementations must receive a previously validated permit.
pub trait ExecutionAdapter: Send + Sync {
    fn adapter_id(&self) -> &str;

    /// Execute one previously authorized request.
    ///
    /// # Errors
    /// Returns [`ExecutionError`] when the concrete adapter cannot validate or execute the permit.
    fn execute(&self, permit: &ExecutionPermit) -> Result<ExecutionResult, ExecutionError>;
}

/// Deterministic no-side-effect adapter for CI and orchestration tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DryRunAdapter {
    id: String,
}

impl DryRunAdapter {
    /// Create a dry-run execution adapter.
    ///
    /// # Errors
    /// Returns [`ExecutionError`] when the adapter ID is empty.
    pub fn new(id: impl Into<String>) -> Result<Self, ExecutionError> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(ExecutionError::EmptyAdapterId);
        }
        Ok(Self { id })
    }
}

impl ExecutionAdapter for DryRunAdapter {
    fn adapter_id(&self) -> &str {
        &self.id
    }

    fn execute(&self, permit: &ExecutionPermit) -> Result<ExecutionResult, ExecutionError> {
        validate_execution_request(&permit.request)?;
        Ok(ExecutionResult {
            request_id: permit.request.request_id.clone(),
            exit_code: Some(0),
            stdout: format!("dry-run:{}", permit.request.tool_id),
            stderr: String::new(),
            timed_out: false,
            cancelled: false,
            output_truncated: false,
        })
    }
}

/// Execution boundary failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionError {
    EmptyAdapterId,
    EmptyRequestId,
    EmptyPrincipal,
    EmptyToolId,
    EmptyWorkingDirectory,
    ZeroTimeout,
    ZeroMemoryLimit,
    ZeroOutputLimit,
    AuthorizationNotAllowed,
    AuthorizationPrincipalMismatch,
    AuthorizationPermissionMismatch,
    AuthorizationScopeMismatch,
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyAdapterId => formatter.write_str("execution adapter id must not be empty"),
            Self::EmptyRequestId => formatter.write_str("execution request id must not be empty"),
            Self::EmptyPrincipal => formatter.write_str("execution principal must not be empty"),
            Self::EmptyToolId => formatter.write_str("execution tool id must not be empty"),
            Self::EmptyWorkingDirectory => {
                formatter.write_str("working directory must not be empty")
            }
            Self::ZeroTimeout => formatter.write_str("execution timeout must be positive"),
            Self::ZeroMemoryLimit => formatter.write_str("execution memory limit must be positive"),
            Self::ZeroOutputLimit => formatter.write_str("execution output limit must be positive"),
            Self::AuthorizationNotAllowed => formatter.write_str("governor decision is not allow"),
            Self::AuthorizationPrincipalMismatch => {
                formatter.write_str("authorization principal does not match execution principal")
            }
            Self::AuthorizationPermissionMismatch => {
                formatter.write_str("authorization permission must be execute")
            }
            Self::AuthorizationScopeMismatch => {
                formatter.write_str("authorization tool scope does not match execution tool")
            }
        }
    }
}

impl Error for ExecutionError {}

fn validate_execution_request(request: &ExecutionRequest) -> Result<(), ExecutionError> {
    if request.request_id.trim().is_empty() {
        return Err(ExecutionError::EmptyRequestId);
    }
    if request.principal.0.trim().is_empty() {
        return Err(ExecutionError::EmptyPrincipal);
    }
    if request.tool_id.trim().is_empty() {
        return Err(ExecutionError::EmptyToolId);
    }
    if request.working_directory.trim().is_empty() {
        return Err(ExecutionError::EmptyWorkingDirectory);
    }
    if request.timeout_ms == 0 {
        return Err(ExecutionError::ZeroTimeout);
    }
    if request.max_memory_bytes == 0 {
        return Err(ExecutionError::ZeroMemoryLimit);
    }
    if request.max_output_bytes == 0 {
        return Err(ExecutionError::ZeroOutputLimit);
    }
    Ok(())
}

/// Convert exact governor authorization evidence into a dispatch permit.
///
/// # Errors
/// Returns [`ExecutionError`] unless the decision is `Allow` for the same principal, exact tool
/// scope, and `Execute` permission declared by the execution request.
pub fn authorize_dispatch(
    request: ExecutionRequest,
    authorization: DecisionRecord,
) -> Result<ExecutionPermit, ExecutionError> {
    validate_execution_request(&request)?;
    if !matches!(authorization.decision, Decision::Allow { .. }) {
        return Err(ExecutionError::AuthorizationNotAllowed);
    }
    if authorization.request.principal != request.principal {
        return Err(ExecutionError::AuthorizationPrincipalMismatch);
    }
    if authorization.request.permission != Permission::Execute {
        return Err(ExecutionError::AuthorizationPermissionMismatch);
    }
    let expected_scope = Scope {
        kind: ScopeKind::Tool,
        resource: request.tool_id.clone(),
    };
    if authorization.request.scope != expected_scope {
        return Err(ExecutionError::AuthorizationScopeMismatch);
    }
    Ok(ExecutionPermit {
        request,
        authorization,
    })
}

#[cfg(test)]
mod tests {
    use autonomy_governor::{Decision, Request, RiskLevel};

    use super::*;

    fn request() -> ExecutionRequest {
        ExecutionRequest {
            request_id: "exec-1".to_owned(),
            principal: PrincipalId("builder".to_owned()),
            tool_id: "cargo.test".to_owned(),
            args: vec!["--workspace".to_owned()],
            working_directory: "/workspace".to_owned(),
            timeout_ms: 60_000,
            max_memory_bytes: 1_073_741_824,
            max_output_bytes: 1_048_576,
        }
    }

    fn authorization(decision: Decision) -> DecisionRecord {
        DecisionRecord {
            request: Request {
                principal: PrincipalId("builder".to_owned()),
                permission: Permission::Execute,
                scope: Scope {
                    kind: ScopeKind::Tool,
                    resource: "cargo.test".to_owned(),
                },
                risk: RiskLevel::Low,
            },
            decision,
            approval: None,
        }
    }

    #[test]
    fn exact_allow_decision_produces_dispatch_permit() {
        let permit = authorize_dispatch(
            request(),
            authorization(Decision::Allow {
                reason: "test grant".to_owned(),
            }),
        )
        .expect("exact allow should permit dispatch");
        let adapter = DryRunAdapter::new("dry-run").expect("valid adapter");
        let result = adapter.execute(&permit).expect("dry run succeeds");
        assert_eq!(result.exit_code, Some(0));
    }

    #[test]
    fn denial_never_becomes_dispatch_permit() {
        let error = authorize_dispatch(
            request(),
            authorization(Decision::Deny {
                reason: "no grant".to_owned(),
            }),
        )
        .expect_err("denial must fail closed");
        assert_eq!(error, ExecutionError::AuthorizationNotAllowed);
    }

    #[test]
    fn mismatched_scope_is_rejected() {
        let mut record = authorization(Decision::Allow {
            reason: "test grant".to_owned(),
        });
        record.request.scope.resource = "cargo.build".to_owned();
        let error = authorize_dispatch(request(), record).expect_err("scope mismatch must fail");
        assert_eq!(error, ExecutionError::AuthorizationScopeMismatch);
    }
}
