//! Governed local process worker for NTPX `GameForge` Ω∞ Agent Zero.
//!
//! B000030 is the first real process boundary. It consumes a previously validated execution,
//! obtains the exact Autonomy Governor permit immediately before spawn, executes only the
//! registered program path, clears the child environment, reapplies only request-allowlisted
//! variables, enforces wall-time/output/cancellation bounds, and emits deterministic result
//! evidence. It does not claim container/process-tree/network/filesystem isolation.

use std::{
    error::Error,
    fmt,
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    process::{Child, ChildStderr, ChildStdout, Command, ExitStatus, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use agent_zero_execution::{ExecutionState, ValidatedExecution};
use autonomy_governor::{AuthorizationPermit, GovernanceRuntime, RuntimeError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Cooperative cancellation signal supplied by the caller.
#[derive(Debug, Clone, Default)]
pub struct CancellationToken(Arc<AtomicBool>);

impl CancellationToken {
    /// Request cancellation. The worker observes this flag while the direct child is running.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }

    /// Return whether cancellation has been requested.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

/// Why execution stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminationReason {
    Exited,
    Cancelled,
    TimedOut,
}

/// Bounded captured stream evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapturedOutput {
    pub bytes: Vec<u8>,
    pub truncated: bool,
    pub sha256: String,
}

/// Successful worker completion evidence, including Governor authorization evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionReceipt {
    pub execution_id: String,
    pub task_id: String,
    pub state: ExecutionState,
    pub termination: TerminationReason,
    pub exit_code: Option<i32>,
    pub stdout: CapturedOutput,
    pub stderr: CapturedOutput,
    pub workspace: PathBuf,
    pub authorization: AuthorizationPermit,
    pub evidence_sha256: String,
}

/// Failures at the real worker boundary.
#[derive(Debug)]
pub enum WorkerError {
    Governor(RuntimeError),
    ProgramMustBeAbsolute(PathBuf),
    ProgramMissing(PathBuf),
    ProgramSymlink(PathBuf),
    WorkspaceMustBeDirectory(PathBuf),
    WorkspaceSymlink(PathBuf),
    Io(io::Error),
    Spawn(io::Error),
    OutputReaderPanicked(&'static str),
    Encode(serde_json::Error),
}

impl fmt::Display for WorkerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Governor(error) => write!(formatter, "Governor authorization failed: {error}"),
            Self::ProgramMustBeAbsolute(path) => write!(
                formatter,
                "registered executable path must be absolute: '{}'",
                path.display()
            ),
            Self::ProgramMissing(path) => {
                write!(formatter, "registered executable does not exist: '{}'", path.display())
            }
            Self::ProgramSymlink(path) => write!(
                formatter,
                "registered executable must not be a symlink: '{}'",
                path.display()
            ),
            Self::WorkspaceMustBeDirectory(path) => write!(
                formatter,
                "worker workspace must be an existing directory: '{}'",
                path.display()
            ),
            Self::WorkspaceSymlink(path) => write!(
                formatter,
                "worker workspace must not be a symlink: '{}'",
                path.display()
            ),
            Self::Io(error) => write!(formatter, "worker I/O failed: {error}"),
            Self::Spawn(error) => write!(formatter, "worker process spawn failed: {error}"),
            Self::OutputReaderPanicked(stream) => {
                write!(formatter, "{stream} output reader thread panicked")
            }
            Self::Encode(error) => write!(formatter, "worker evidence encoding failed: {error}"),
        }
    }
}

impl Error for WorkerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Governor(error) => Some(error),
            Self::Io(error) | Self::Spawn(error) => Some(error),
            Self::Encode(error) => Some(error),
            Self::ProgramMustBeAbsolute(_)
            | Self::ProgramMissing(_)
            | Self::ProgramSymlink(_)
            | Self::WorkspaceMustBeDirectory(_)
            | Self::WorkspaceSymlink(_)
            | Self::OutputReaderPanicked(_) => None,
        }
    }
}

impl From<RuntimeError> for WorkerError {
    fn from(error: RuntimeError) -> Self {
        Self::Governor(error)
    }
}

impl From<io::Error> for WorkerError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// Execute one validated Agent Zero request through the verified Governor.
///
/// Authorization occurs immediately before process creation. A denied or human-gated request
/// returns before spawn. Child environment inheritance is disabled with `env_clear`; only the
/// already validated request environment is added. The direct child is killed on wall timeout or
/// after the cancellation grace period. stdout/stderr are always drained concurrently while only
/// the declared byte limits are retained.
///
/// # Errors
/// Returns [`WorkerError`] for Governor denial/approval requirements, invalid executable/workspace
/// boundaries, process I/O/spawn failures, output-reader failure, or evidence encoding failure.
pub fn execute_local(
    validated: &ValidatedExecution,
    runtime: &GovernanceRuntime<'_>,
    workspace: impl AsRef<Path>,
    cancellation: &CancellationToken,
) -> Result<ExecutionReceipt, WorkerError> {
    let program = validate_program(Path::new(&validated.contract.program))?;
    let workspace = validate_workspace(workspace.as_ref())?;

    // This is intentionally the final authority check before Command::spawn.
    let authorization = runtime.authorize(validated.request.governed_request.clone(), None)?;

    let mut command = Command::new(&program);
    command
        .args(&validated.contract.fixed_args)
        .args(&validated.request.dynamic_args)
        .current_dir(&workspace)
        .env_clear()
        .envs(&validated.request.environment)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn().map_err(WorkerError::Spawn)?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| WorkerError::Io(io::Error::other("stdout pipe unavailable")))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| WorkerError::Io(io::Error::other("stderr pipe unavailable")))?;

    let stdout_limit = validated.contract.limits.max_stdout_bytes;
    let stderr_limit = validated.contract.limits.max_stderr_bytes;
    let stdout_reader = thread::spawn(move || read_bounded_stdout(stdout, stdout_limit));
    let stderr_reader = thread::spawn(move || read_bounded_stderr(stderr, stderr_limit));

    let started = Instant::now();
    let wall_time = Duration::from_millis(validated.contract.limits.wall_time_ms);
    let cancellation_grace = Duration::from_millis(validated.contract.limits.cancel_grace_ms);
    let (status, termination) = wait_bounded(
        &mut child,
        started,
        wall_time,
        cancellation_grace,
        cancellation,
    )?;

    let stdout = stdout_reader
        .join()
        .map_err(|_| WorkerError::OutputReaderPanicked("stdout"))??;
    let stderr = stderr_reader
        .join()
        .map_err(|_| WorkerError::OutputReaderPanicked("stderr"))??;

    let state = match termination {
        TerminationReason::TimedOut => ExecutionState::TimedOut,
        TerminationReason::Cancelled => ExecutionState::Cancelled,
        TerminationReason::Exited if status.success() => ExecutionState::Succeeded,
        TerminationReason::Exited => ExecutionState::Failed,
    };
    let exit_code = status.code();
    let evidence_sha256 = receipt_digest(
        validated,
        state,
        termination,
        exit_code,
        &stdout,
        &stderr,
        &workspace,
    )?;

    Ok(ExecutionReceipt {
        execution_id: validated.request.execution_id.clone(),
        task_id: validated.request.task_id.clone(),
        state,
        termination,
        exit_code,
        stdout,
        stderr,
        workspace,
        authorization,
        evidence_sha256,
    })
}

fn validate_program(program: &Path) -> Result<PathBuf, WorkerError> {
    if !program.is_absolute() {
        return Err(WorkerError::ProgramMustBeAbsolute(program.to_owned()));
    }
    let metadata = fs::symlink_metadata(program).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            WorkerError::ProgramMissing(program.to_owned())
        } else {
            WorkerError::Io(error)
        }
    })?;
    if metadata.file_type().is_symlink() {
        return Err(WorkerError::ProgramSymlink(program.to_owned()));
    }
    if !metadata.is_file() {
        return Err(WorkerError::ProgramMissing(program.to_owned()));
    }
    fs::canonicalize(program).map_err(WorkerError::Io)
}

fn validate_workspace(workspace: &Path) -> Result<PathBuf, WorkerError> {
    let metadata = fs::symlink_metadata(workspace).map_err(WorkerError::Io)?;
    if metadata.file_type().is_symlink() {
        return Err(WorkerError::WorkspaceSymlink(workspace.to_owned()));
    }
    if !metadata.is_dir() {
        return Err(WorkerError::WorkspaceMustBeDirectory(workspace.to_owned()));
    }
    fs::canonicalize(workspace).map_err(WorkerError::Io)
}

fn wait_bounded(
    child: &mut Child,
    started: Instant,
    wall_time: Duration,
    cancellation_grace: Duration,
    cancellation: &CancellationToken,
) -> Result<(ExitStatus, TerminationReason), WorkerError> {
    let mut cancellation_started = None;
    loop {
        if let Some(status) = child.try_wait()? {
            let reason = if cancellation_started.is_some() {
                TerminationReason::Cancelled
            } else {
                TerminationReason::Exited
            };
            return Ok((status, reason));
        }

        if started.elapsed() >= wall_time {
            child.kill()?;
            return Ok((child.wait()?, TerminationReason::TimedOut));
        }

        if cancellation.is_cancelled() {
            let cancel_started = cancellation_started.get_or_insert_with(Instant::now);
            if cancel_started.elapsed() >= cancellation_grace {
                child.kill()?;
                return Ok((child.wait()?, TerminationReason::Cancelled));
            }
        }

        thread::sleep(Duration::from_millis(5));
    }
}

fn read_bounded_stdout(stream: ChildStdout, limit: u64) -> io::Result<CapturedOutput> {
    read_bounded(stream, limit)
}

fn read_bounded_stderr(stream: ChildStderr, limit: u64) -> io::Result<CapturedOutput> {
    read_bounded(stream, limit)
}

fn read_bounded(mut reader: impl Read, limit: u64) -> io::Result<CapturedOutput> {
    let mut retained = Vec::new();
    let mut digest = Sha256::new();
    let mut truncated = false;
    let mut buffer = [0_u8; 8192];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        let chunk = &buffer[..count];
        digest.update(chunk);
        let available = usize::try_from(limit.saturating_sub(retained.len() as u64))
            .unwrap_or(usize::MAX);
        let keep = available.min(chunk.len());
        retained.extend_from_slice(&chunk[..keep]);
        truncated |= keep < chunk.len();
    }
    Ok(CapturedOutput {
        bytes: retained,
        truncated,
        sha256: format!("{:x}", digest.finalize()),
    })
}

#[derive(Serialize)]
struct ReceiptEvidence<'a> {
    contract_evidence_sha256: &'a str,
    request_evidence_sha256: &'a str,
    state: ExecutionState,
    termination: TerminationReason,
    exit_code: Option<i32>,
    stdout_sha256: &'a str,
    stdout_truncated: bool,
    stderr_sha256: &'a str,
    stderr_truncated: bool,
    workspace: &'a Path,
}

fn receipt_digest(
    validated: &ValidatedExecution,
    state: ExecutionState,
    termination: TerminationReason,
    exit_code: Option<i32>,
    stdout: &CapturedOutput,
    stderr: &CapturedOutput,
    workspace: &Path,
) -> Result<String, WorkerError> {
    let evidence = ReceiptEvidence {
        contract_evidence_sha256: &validated.contract_evidence_sha256,
        request_evidence_sha256: &validated.request_evidence_sha256,
        state,
        termination,
        exit_code,
        stdout_sha256: &stdout.sha256,
        stdout_truncated: stdout.truncated,
        stderr_sha256: &stderr.sha256,
        stderr_truncated: stderr.truncated,
        workspace,
    };
    let encoded = serde_json::to_vec(&evidence).map_err(WorkerError::Encode)?;
    Ok(format!("{:x}", Sha256::digest(encoded)))
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, BTreeSet},
        env,
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use agent_zero_execution::{
        validate_execution, ExecutionLimits, ExecutionRequest, ToolContract,
    };
    use autonomy_governor::{
        ApprovalRequirement, AuditLog, Grant, Permission, Policy, Principal, PrincipalId, Request,
        RiskLevel, Scope, ScopeKind,
    };

    use super::*;

    fn temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        let path = env::temp_dir().join(format!(
            "gameforge-agent-zero-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("temp workspace should be created");
        path
    }

    fn setup(
        label: &str,
        child_mode: &str,
        limits: ExecutionLimits,
        environment: BTreeMap<String, String>,
        allowed: bool,
    ) -> (ValidatedExecution, Policy, AuditLog, PathBuf) {
        let principal = PrincipalId("agent-zero".to_owned());
        let scope = Scope {
            kind: ScopeKind::Tool,
            resource: format!("tool:{label}"),
        };
        let request = Request {
            principal: principal.clone(),
            permission: Permission::Execute,
            scope: scope.clone(),
            risk: RiskLevel::Elevated,
        };
        let contract = ToolContract {
            id: label.to_owned(),
            program: env::current_exe()
                .expect("test executable path")
                .to_string_lossy()
                .into_owned(),
            governed_resource: scope.resource.clone(),
            fixed_args: vec![format!("worker_child_{child_mode}"), "--nocapture".to_owned()],
            environment_allowlist: environment.keys().cloned().collect::<BTreeSet<_>>(),
            limits,
        };
        let execution = ExecutionRequest {
            execution_id: format!("exec-{label}"),
            task_id: format!("task-{label}"),
            principal: principal.clone(),
            tool_id: label.to_owned(),
            dynamic_args: Vec::new(),
            environment,
            governed_request: request,
        };
        let validated = validate_execution(&contract, &execution).expect("execution should validate");
        let grants = if allowed {
            vec![Grant {
                principal: principal.clone(),
                permission: Permission::Execute,
                scope,
                approval: ApprovalRequirement::None,
            }]
        } else {
            Vec::new()
        };
        let policy = Policy {
            principals: vec![Principal {
                id: principal,
                role: "agent-zero-test".to_owned(),
            }],
            grants,
        };
        let workspace = temp_dir(label);
        let audit = AuditLog::new(workspace.join("audit.jsonl"));
        (validated, policy, audit, workspace)
    }

    fn normal_limits() -> ExecutionLimits {
        ExecutionLimits {
            wall_time_ms: 5_000,
            cancel_grace_ms: 20,
            max_stdout_bytes: 4096,
            max_stderr_bytes: 4096,
            max_dynamic_args: 0,
        }
    }

    #[test]
    fn denied_request_never_reaches_spawn() {
        let (validated, policy, audit, workspace) = setup(
            "denied",
            "success",
            normal_limits(),
            BTreeMap::new(),
            false,
        );
        let runtime = GovernanceRuntime::new(&policy, &audit);
        let error = execute_local(
            &validated,
            &runtime,
            &workspace,
            &CancellationToken::default(),
        )
        .expect_err("denied execution must fail before spawn");
        assert!(matches!(error, WorkerError::Governor(RuntimeError::Denied { .. })));
        assert_eq!(audit.load_verified().expect("audit should verify").len(), 1);
        fs::remove_dir_all(workspace).expect("workspace should remove");
    }

    #[test]
    fn successful_execution_is_authorized_and_captured() {
        if env::var("NTPX_WORKER_CHILD_MODE").as_deref() == Ok("success") {
            println!("agent-zero-success");
            return;
        }
        let environment = BTreeMap::from([(
            "NTPX_WORKER_CHILD_MODE".to_owned(),
            "success".to_owned(),
        )]);
        let (validated, policy, audit, workspace) = setup(
            "success",
            "success",
            normal_limits(),
            environment,
            true,
        );
        let runtime = GovernanceRuntime::new(&policy, &audit);
        let receipt = execute_local(
            &validated,
            &runtime,
            &workspace,
            &CancellationToken::default(),
        )
        .expect("authorized execution should succeed");
        assert_eq!(receipt.state, ExecutionState::Succeeded);
        assert_eq!(receipt.termination, TerminationReason::Exited);
        assert!(String::from_utf8_lossy(&receipt.stdout.bytes).contains("agent-zero-success"));
        assert_eq!(audit.load_verified().expect("audit should verify").len(), 1);
        fs::remove_dir_all(workspace).expect("workspace should remove");
    }

    #[test]
    fn worker_child_success() {
        if env::var("NTPX_WORKER_CHILD_MODE").as_deref() == Ok("success") {
            println!("agent-zero-success");
        }
    }

    #[test]
    fn environment_is_cleared_before_allowlisted_values_are_added() {
        if env::var("NTPX_WORKER_CHILD_MODE").as_deref() == Ok("env") {
            assert_eq!(env::var("NTPX_ALLOWED").as_deref(), Ok("visible"));
            assert!(env::var("PATH").is_err());
            println!("environment-scrubbed");
            return;
        }
        let environment = BTreeMap::from([
            ("NTPX_WORKER_CHILD_MODE".to_owned(), "env".to_owned()),
            ("NTPX_ALLOWED".to_owned(), "visible".to_owned()),
        ]);
        let (validated, policy, audit, workspace) = setup(
            "env",
            "env",
            normal_limits(),
            environment,
            true,
        );
        let runtime = GovernanceRuntime::new(&policy, &audit);
        let receipt = execute_local(
            &validated,
            &runtime,
            &workspace,
            &CancellationToken::default(),
        )
        .expect("environment test should run");
        assert_eq!(receipt.state, ExecutionState::Succeeded);
        assert!(String::from_utf8_lossy(&receipt.stdout.bytes).contains("environment-scrubbed"));
        fs::remove_dir_all(workspace).expect("workspace should remove");
    }

    #[test]
    fn worker_child_env() {
        if env::var("NTPX_WORKER_CHILD_MODE").as_deref() == Ok("env") {
            assert_eq!(env::var("NTPX_ALLOWED").as_deref(), Ok("visible"));
            assert!(env::var("PATH").is_err());
            println!("environment-scrubbed");
        }
    }

    #[test]
    fn stdout_limit_is_enforced_while_stream_is_drained() {
        if env::var("NTPX_WORKER_CHILD_MODE").as_deref() == Ok("output") {
            println!("{}", "x".repeat(8192));
            return;
        }
        let mut limits = normal_limits();
        limits.max_stdout_bytes = 64;
        let environment = BTreeMap::from([(
            "NTPX_WORKER_CHILD_MODE".to_owned(),
            "output".to_owned(),
        )]);
        let (validated, policy, audit, workspace) =
            setup("output", "output", limits, environment, true);
        let runtime = GovernanceRuntime::new(&policy, &audit);
        let receipt = execute_local(
            &validated,
            &runtime,
            &workspace,
            &CancellationToken::default(),
        )
        .expect("output test should run");
        assert_eq!(receipt.stdout.bytes.len(), 64);
        assert!(receipt.stdout.truncated);
        fs::remove_dir_all(workspace).expect("workspace should remove");
    }

    #[test]
    fn worker_child_output() {
        if env::var("NTPX_WORKER_CHILD_MODE").as_deref() == Ok("output") {
            println!("{}", "x".repeat(8192));
        }
    }

    #[test]
    fn wall_timeout_terminates_direct_child() {
        if env::var("NTPX_WORKER_CHILD_MODE").as_deref() == Ok("timeout") {
            thread::sleep(Duration::from_secs(2));
            return;
        }
        let mut limits = normal_limits();
        limits.wall_time_ms = 30;
        let environment = BTreeMap::from([(
            "NTPX_WORKER_CHILD_MODE".to_owned(),
            "timeout".to_owned(),
        )]);
        let (validated, policy, audit, workspace) =
            setup("timeout", "timeout", limits, environment, true);
        let runtime = GovernanceRuntime::new(&policy, &audit);
        let receipt = execute_local(
            &validated,
            &runtime,
            &workspace,
            &CancellationToken::default(),
        )
        .expect("timeout must produce receipt");
        assert_eq!(receipt.state, ExecutionState::TimedOut);
        assert_eq!(receipt.termination, TerminationReason::TimedOut);
        fs::remove_dir_all(workspace).expect("workspace should remove");
    }

    #[test]
    fn worker_child_timeout() {
        if env::var("NTPX_WORKER_CHILD_MODE").as_deref() == Ok("timeout") {
            thread::sleep(Duration::from_secs(2));
        }
    }

    #[test]
    fn cancellation_grace_then_kill_is_enforced() {
        if env::var("NTPX_WORKER_CHILD_MODE").as_deref() == Ok("cancel") {
            thread::sleep(Duration::from_secs(2));
            return;
        }
        let environment = BTreeMap::from([(
            "NTPX_WORKER_CHILD_MODE".to_owned(),
            "cancel".to_owned(),
        )]);
        let (validated, policy, audit, workspace) = setup(
            "cancel",
            "cancel",
            normal_limits(),
            environment,
            true,
        );
        let runtime = GovernanceRuntime::new(&policy, &audit);
        let token = CancellationToken::default();
        token.cancel();
        let receipt = execute_local(&validated, &runtime, &workspace, &token)
            .expect("cancel must produce receipt");
        assert_eq!(receipt.state, ExecutionState::Cancelled);
        assert_eq!(receipt.termination, TerminationReason::Cancelled);
        fs::remove_dir_all(workspace).expect("workspace should remove");
    }

    #[test]
    fn worker_child_cancel() {
        if env::var("NTPX_WORKER_CHILD_MODE").as_deref() == Ok("cancel") {
            thread::sleep(Duration::from_secs(2));
        }
    }
}
