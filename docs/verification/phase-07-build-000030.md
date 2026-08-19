# Phase 07 — Build B000030 verification evidence

## Candidate

- Build: `NTPX-GFΩ-G0-v0.7.0-dev.2-M07-B000030`
- Branch: `agent/phase07-agent-zero-contracts`
- Classification: **BLOCKED**

## Intended acceptance checks

1. Rust formatting: `cargo fmt --check`
2. Strict static analysis: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
3. Rust unit/integration tests: `cargo test --workspace`
4. Desktop typecheck/build
5. Windows Autonomy Governor regression checks/tests
6. Carried-forward NTPX Fortress gate on the exact Phase 07 topology

## Candidate behavior under test

- Governor denial happens before `Command::spawn`.
- Registered executable must be absolute/existing/non-symlink.
- Workspace must be an existing non-symlink directory.
- Child environment is cleared before applying only request-allowlisted values.
- Direct child stdin is disabled.
- stdout/stderr are concurrently drained and retained to independent byte limits with explicit truncation and full-stream hashes.
- Wall timeout kills/reaps the direct child.
- Cancellation waits the declared grace period then kills/reaps the direct child when it remains alive.
- Result receipt carries the Governor permit and deterministic result evidence.

## CI evidence

### Run 32267727263

All configured jobs were marked failed, but GitHub returned no executed steps for Rust foundation, Desktop frontend, or Autonomy Governor Windows. Job logs were unavailable. This run therefore supplies no formatting, compilation, test, or behavior evidence.

### Run 32267871591

A second trigger produced the same runner-startup pattern: all three jobs were marked failed with zero executed steps. This is treated as unavailable execution evidence, not as a B000030 source/test failure.

### Run 32268086696

A third exact-head retry on commit `5711b89a6614c877749eed29e7789c51472aebb6` reproduced the same condition: all three CI jobs failed before step creation and no usable logs were produced.

### Phase 07 Fortress topology repair

The stacked Phase 07 branch did not contain the carried-forward Fortress workflow even though `main` did. Commit `bce0181a27a3c572f0b1d824ecdc0b0c1f802162` copied `.github/workflows/ntpx-fortress-bootstrap.yml` byte-for-byte in behavior from `main` onto the Phase 07 branch. No Fortress policy, action pin, permission, enforcement rule, or monitor command was weakened.

This repair successfully caused both CI and Fortress workflows to be created for the exact Phase 07 head.

### Run 32279260413 — CI after topology repair

All three CI jobs (`Rust foundation`, `Desktop frontend`, and `Autonomy Governor Windows`) again concluded failure with `steps: null`. The job-log endpoint for Rust returned `BlobNotFound`, so no formatter, compiler, Clippy, test, frontend, or Windows command executed or produced evidence.

### Run 32279260467 — Fortress after topology repair

The restored `Fortress monitor gate` job was created, but it also concluded failure with `steps: null`. This is significant because the Fortress job is independent of the B000030 Rust worker implementation. Its zero-step failure confirms that the current blocker occurs before workflow step execution rather than inside the Agent Zero worker tests or Fortress monitor script.

## Blocker diagnosis

Repository-side workflow definitions are present and syntactically accepted strongly enough for GitHub to create their jobs. Independent Linux CI, Windows CI, frontend, and Fortress jobs all terminate before step creation, and job logs are unavailable. No source-level failure can be extracted or safely repaired from these runs.

The next prerequisite is restoration of GitHub Actions runner execution for this repository/account. Until at least one job can actually start, changing B000030 source merely to chase a zero-step failure would violate the project rule against weakening or speculative build repairs.

## Residual limits

B000030 does not claim OS/container isolation, process-group/tree termination, network isolation, filesystem isolation, syscall filtering, CPU/memory quotas, or credential-file isolation. `env_clear` only addresses ambient environment-variable inheritance. These remain Phase 07 acceptance work.

## Outcome

**BLOCKED** — the candidate cannot be promoted to VERIFIED because the required executable gates cannot start. This is not classified as a source/test FAILED result: no relevant workflow step executed. Phase 07 remains IN PROGRESS, the continuous build stays B000030, and no milestone advancement is justified.
