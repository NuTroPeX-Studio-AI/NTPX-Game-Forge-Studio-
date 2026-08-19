//! Deterministic execution-contract foundation for NTPX `GameForge` Ω∞ Agent Zero.
//!
//! This crate validates explicit tool/process requests, resource bounds, cancellation/lifecycle
//! transitions, and the exact Autonomy Governor request that a later worker must authorize before
//! process creation. B000029 intentionally does not spawn processes, invoke shells, use credentials,
//! open networks, mutate repositories, or bypass the verified Governor/Orchestrator boundaries.

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

use autonomy_governor::{Permission, PrincipalId, Request, ScopeKind};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Immutable resource limits attached to one registered tool contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionLimits {
    pub wall_time_ms: u64,
    pub cancel_grace_ms: u64,
    pub max_stdout_bytes: u64,
    pub max_stderr_bytes: u64,
    pub max_dynamic_args: u16,
}

/// Immutable executable/tool declaration. Registration is evidence only; it grants no authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolContract {
    pub id: String,
    pub program: String,
    pub governed_resource: String,
    pub fixed_args: Vec<String>,
    pub environment_allowlist: BTreeSet<String>,
    pub limits: ExecutionLimits,
}

/// One requested Agent Zero process execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub execution_id: String,
    pub task_id: String,
    pub principal: PrincipalId,
    pub tool_id: String,
    pub dynamic_args: Vec<String>,
    pub environment: BTreeMap<String, String>,
    pub governed_request: Request,
}

/// Validated deterministic execution evidence. This is still not an authorization permit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatedExecution {
    pub contract: ToolContract,
    pub request: ExecutionRequest,
    pub contract_evidence_sha256: String,
    pub request_evidence_sha256: String,
}

/// Explicit Agent Zero execution lifecycle states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionState {
    Planned,
    Authorized,
    Running,
    CancellationRequested,
    Succeeded,
    Failed,
    Cancelled,
    TimedOut,
}

impl ExecutionState {
    const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::Failed | Self::Cancelled | Self::TimedOut
        )
    }
}

/// Deterministic lifecycle event for one execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionEvent {
    pub sequence: u64,
    pub from: ExecutionState,
    pub to: ExecutionState,
    pub reason: String,
}

/// In-memory lifecycle state machine for one validated execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionLifecycle {
    execution_id: String,
    state: ExecutionState,
    events: Vec<ExecutionEvent>,
}

impl ExecutionLifecycle {
    /// Construct a planned lifecycle for one validated execution.
    #[must_use]
    pub fn new(validated: &ValidatedExecution) -> Self {
        Self {
            execution_id: validated.request.execution_id.clone(),
            state: ExecutionState::Planned,
            events: Vec::new(),
        }
    }

    /// Return the current lifecycle state.
    #[must_use]
    pub const fn state(&self) -> ExecutionState {
        self.state
    }

    /// Return immutable transition evidence.
    #[must_use]
    pub fn events(&self) -> &[ExecutionEvent] {
        &self.events
    }

    /// Apply one allowed lifecycle transition.
    ///
    /// # Errors
    /// Returns [`ExecutionError::InvalidTransition`] when the requested transition is not legal.
    pub fn transition(
        &mut self,
        to: ExecutionState,
        reason: impl Into<String>,
    ) -> Result<(), ExecutionError> {
        let from = self.state;
        if !transition_allowed(from, to) {
            return Err(ExecutionError::InvalidTransition { from, to });
        }
        let reason = reason.into();
        if reason.trim().is_empty() {
            return Err(ExecutionError::InvalidLifecycleReason);
        }
        let sequence = u64::try_from(self.events.len() + 1).unwrap_or(u64::MAX);
        self.events.push(ExecutionEvent {
            sequence,
            from,
            to,
            reason,
        });
        self.state = to;
        Ok(())
    }

    /// Request cooperative cancellation before or during process execution.
    ///
    /// # Errors
    /// Returns [`ExecutionError::InvalidTransition`] after a terminal state or from any state that
    /// cannot legally accept cancellation.
    pub fn request_cancellation(&mut self, reason: impl Into<String>) -> Result<(), ExecutionError> {
        self.transition(ExecutionState::CancellationRequested, reason)
    }
}

fn transition_allowed(from: ExecutionState, to: ExecutionState) -> bool {
    match from {
        ExecutionState::Planned => matches!(
            to,
            ExecutionState::Authorized | ExecutionState::CancellationRequested
        ),
        ExecutionState::Authorized => matches!(
            to,
            ExecutionState::Running | ExecutionState::CancellationRequested
        ),
        ExecutionState::Running => matches!(
            to,
            ExecutionState::CancellationRequested
                | ExecutionState::Succeeded
                | ExecutionState::Failed
                | ExecutionState::TimedOut
        ),
        ExecutionState::CancellationRequested => matches!(
            to,
            ExecutionState::Cancelled | ExecutionState::Failed | ExecutionState::TimedOut
        ),
        state if state.is_terminal() => false,
        _ => false,
    }
}

/// Validation and lifecycle failures for Agent Zero execution contracts.
#[derive(Debug)]
pub enum ExecutionError {
    InvalidTool { reason: String },
    InvalidRequest { reason: String },
    GovernedPrincipalMismatch,
    GovernedPermissionMismatch,
    GovernedScopeKindMismatch,
    GovernedResourceMismatch { expected: String, actual: String },
    DynamicArgumentLimit { allowed: u16, actual: usize },
    EnvironmentNotAllowed(String),
    Encode(serde_json::Error),
    InvalidTransition { from: ExecutionState, to: ExecutionState },
    InvalidLifecycleReason,
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTool { reason } => write!(formatter, "invalid tool contract: {reason}"),
            Self::InvalidRequest { reason } => write!(formatter, "invalid execution request: {reason}"),
            Self::GovernedPrincipalMismatch => {
                formatter.write_str("execution principal does not match governed request principal")
            }
            Self::GovernedPermissionMismatch => {
                formatter.write_str("execution requires exact Governor Execute permission")
            }
            Self::GovernedScopeKindMismatch => {
                formatter.write_str("execution requires exact Governor Tool scope")
            }
            Self::GovernedResourceMismatch { expected, actual } => write!(
                formatter,
                "governed tool resource mismatch: expected '{expected}', got '{actual}'"
            ),
            Self::DynamicArgumentLimit { allowed, actual } => write!(
                formatter,
                "execution dynamic argument count {actual} exceeds contract maximum {allowed}"
            ),
            Self::EnvironmentNotAllowed(key) => {
                write!(formatter, "execution environment key '{key}' is not allowlisted")
            }
            Self::Encode(error) => write!(formatter, "execution evidence encoding failed: {error}"),
            Self::InvalidTransition { from, to } => {
                write!(formatter, "invalid execution lifecycle transition {from:?} -> {to:?}")
            }
            Self::InvalidLifecycleReason => {
                formatter.write_str("execution lifecycle transition reason must not be blank")
            }
        }
    }
}

impl Error for ExecutionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Encode(error) => Some(error),
            Self::InvalidTool { .. }
            | Self::InvalidRequest { .. }
            | Self::GovernedPrincipalMismatch
            | Self::GovernedPermissionMismatch
            | Self::GovernedScopeKindMismatch
            | Self::GovernedResourceMismatch { .. }
            | Self::DynamicArgumentLimit { .. }
            | Self::EnvironmentNotAllowed(_)
            | Self::InvalidTransition { .. }
            | Self::InvalidLifecycleReason => None,
        }
    }
}

/// Validate one tool contract and execution request into deterministic evidence.
///
/// The exact governed request must already be predeclared by the Agent Orchestrator contract. This
/// function only verifies that it is correctly bound to this execution; it does not authorize it.
///
/// # Errors
/// Returns [`ExecutionError`] for malformed contracts/requests, limit violations, environment
/// expansion, governed-principal/scope/permission mismatch, or evidence serialization failure.
pub fn validate_execution(
    contract: &ToolContract,
    request: &ExecutionRequest,
) -> Result<ValidatedExecution, ExecutionError> {
    validate_contract(contract)?;
    validate_request(contract, request)?;
    let contract_evidence_sha256 = evidence_digest(contract)?;
    let request_evidence_sha256 = evidence_digest(request)?;
    Ok(ValidatedExecution {
        contract: contract.clone(),
        request: request.clone(),
        contract_evidence_sha256,
        request_evidence_sha256,
    })
}

fn validate_contract(contract: &ToolContract) -> Result<(), ExecutionError> {
    if contract.id.trim().is_empty() {
        return Err(ExecutionError::InvalidTool {
            reason: "tool id must not be blank".to_owned(),
        });
    }
    if contract.program.trim().is_empty() {
        return Err(ExecutionError::InvalidTool {
            reason: "program must not be blank".to_owned(),
        });
    }
    if contract.governed_resource.trim().is_empty() {
        return Err(ExecutionError::InvalidTool {
            reason: "governed resource must not be blank".to_owned(),
        });
    }
    if contract.limits.wall_time_ms == 0
        || contract.limits.cancel_grace_ms == 0
        || contract.limits.max_stdout_bytes == 0
        || contract.limits.max_stderr_bytes == 0
    {
        return Err(ExecutionError::InvalidTool {
            reason: "time and output limits must be greater than zero".to_owned(),
        });
    }
    if contract
        .environment_allowlist
        .iter()
        .any(|key| key.trim().is_empty())
    {
        return Err(ExecutionError::InvalidTool {
            reason: "environment allowlist keys must not be blank".to_owned(),
        });
    }
    Ok(())
}

fn validate_request(
    contract: &ToolContract,
    request: &ExecutionRequest,
) -> Result<(), ExecutionError> {
    if request.execution_id.trim().is_empty() || request.task_id.trim().is_empty() {
        return Err(ExecutionError::InvalidRequest {
            reason: "execution id and task id must not be blank".to_owned(),
        });
    }
    if request.tool_id != contract.id {
        return Err(ExecutionError::InvalidRequest {
            reason: "request tool id does not match tool contract".to_owned(),
        });
    }
    if request.dynamic_args.len() > usize::from(contract.limits.max_dynamic_args) {
        return Err(ExecutionError::DynamicArgumentLimit {
            allowed: contract.limits.max_dynamic_args,
            actual: request.dynamic_args.len(),
        });
    }
    for key in request.environment.keys() {
        if !contract.environment_allowlist.contains(key) {
            return Err(ExecutionError::EnvironmentNotAllowed(key.clone()));
        }
    }
    if request.principal != request.governed_request.principal {
        return Err(ExecutionError::GovernedPrincipalMismatch);
    }
    if request.governed_request.permission != Permission::Execute {
        return Err(ExecutionError::GovernedPermissionMismatch);
    }
    if request.governed_request.scope.kind != ScopeKind::Tool {
        return Err(ExecutionError::GovernedScopeKindMismatch);
    }
    if request.governed_request.scope.resource != contract.governed_resource {
        return Err(ExecutionError::GovernedResourceMismatch {
            expected: contract.governed_resource.clone(),
            actual: request.governed_request.scope.resource.clone(),
        });
    }
    Ok(())
}

fn evidence_digest(value: &impl Serialize) -> Result<String, ExecutionError> {
    let encoded = serde_json::to_vec(value).map_err(ExecutionError::Encode)?;
    Ok(format!("{:x}", Sha256::digest(encoded)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use autonomy_governor::{RiskLevel, Scope};

    fn contract() -> ToolContract {
        ToolContract {
            id: "cargo-test".to_owned(),
            program: "cargo".to_owned(),
            governed_resource: "tool:cargo-test".to_owned(),
            fixed_args: vec!["test".to_owned()],
            environment_allowlist: BTreeSet::from(["RUST_BACKTRACE".to_owned()]),
            limits: ExecutionLimits {
                wall_time_ms: 120_000,
                cancel_grace_ms: 2_000,
                max_stdout_bytes: 1_048_576,
                max_stderr_bytes: 1_048_576,
                max_dynamic_args: 4,
            },
        }
    }

    fn request() -> ExecutionRequest {
        let principal = PrincipalId("agent-zero".to_owned());
        ExecutionRequest {
            execution_id: "exec-0001".to_owned(),
            task_id: "task-build".to_owned(),
            principal: principal.clone(),
            tool_id: "cargo-test".to_owned(),
            dynamic_args: vec!["--workspace".to_owned()],
            environment: BTreeMap::from([("RUST_BACKTRACE".to_owned(), "1".to_owned())]),
            governed_request: Request {
                principal,
                permission: Permission::Execute,
                scope: Scope {
                    kind: ScopeKind::Tool,
                    resource: "tool:cargo-test".to_owned(),
                },
                risk: RiskLevel::Elevated,
            },
        }
    }

    #[test]
    fn validation_is_deterministic_and_bound_to_governor_request() {
        let first = validate_execution(&contract(), &request()).expect("valid execution");
        let second = validate_execution(&contract(), &request()).expect("valid execution");
        assert_eq!(first, second);
        assert_eq!(first.request.governed_request.permission, Permission::Execute);
        assert_eq!(first.request.governed_request.scope.kind, ScopeKind::Tool);
    }

    #[test]
    fn environment_expansion_fails_closed() {
        let mut requested = request();
        requested
            .environment
            .insert("SECRET_TOKEN".to_owned(), "never".to_owned());
        assert!(matches!(
            validate_execution(&contract(), &requested),
            Err(ExecutionError::EnvironmentNotAllowed(key)) if key == "SECRET_TOKEN"
        ));
    }

    #[test]
    fn governed_principal_scope_and_permission_are_exact() {
        let mut requested = request();
        requested.governed_request.principal = PrincipalId("other-agent".to_owned());
        assert!(matches!(
            validate_execution(&contract(), &requested),
            Err(ExecutionError::GovernedPrincipalMismatch)
        ));

        let mut requested = request();
        requested.governed_request.permission = Permission::Write;
        assert!(matches!(
            validate_execution(&contract(), &requested),
            Err(ExecutionError::GovernedPermissionMismatch)
        ));

        let mut requested = request();
        requested.governed_request.scope.resource = "tool:other".to_owned();
        assert!(matches!(
            validate_execution(&contract(), &requested),
            Err(ExecutionError::GovernedResourceMismatch { .. })
        ));
    }

    #[test]
    fn dynamic_argument_limit_fails_closed() {
        let mut tool = contract();
        tool.limits.max_dynamic_args = 0;
        let error = validate_execution(&tool, &request()).expect_err("argument must be rejected");
        assert!(matches!(error, ExecutionError::DynamicArgumentLimit { .. }));
    }

    #[test]
    fn lifecycle_enforces_authorize_run_cancel_terminal_order() {
        let validated = validate_execution(&contract(), &request()).expect("valid execution");
        let mut lifecycle = ExecutionLifecycle::new(&validated);
        lifecycle
            .transition(ExecutionState::Authorized, "Governor permit recorded")
            .expect("authorization transition");
        lifecycle
            .transition(ExecutionState::Running, "worker started")
            .expect("running transition");
        lifecycle
            .request_cancellation("task cancelled")
            .expect("cancellation request");
        lifecycle
            .transition(ExecutionState::Cancelled, "worker terminated")
            .expect("cancelled transition");
        assert_eq!(lifecycle.state(), ExecutionState::Cancelled);
        assert_eq!(lifecycle.events().len(), 4);
        assert!(matches!(
            lifecycle.transition(ExecutionState::Running, "restart"),
            Err(ExecutionError::InvalidTransition { .. })
        ));
    }

    #[test]
    fn terminal_success_cannot_be_cancelled_or_restarted() {
        let validated = validate_execution(&contract(), &request()).expect("valid execution");
        let mut lifecycle = ExecutionLifecycle::new(&validated);
        lifecycle
            .transition(ExecutionState::Authorized, "authorized")
            .expect("authorization transition");
        lifecycle
            .transition(ExecutionState::Running, "started")
            .expect("running transition");
        lifecycle
            .transition(ExecutionState::Succeeded, "exit code zero")
            .expect("success transition");
        assert!(matches!(
            lifecycle.request_cancellation("too late"),
            Err(ExecutionError::InvalidTransition { .. })
        ));
    }
}
