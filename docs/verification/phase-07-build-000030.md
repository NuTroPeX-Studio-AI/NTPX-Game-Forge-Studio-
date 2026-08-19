# Phase 07 — Build B000030 verification evidence

## Candidate

- Build: `NTPX-GFΩ-G0-v0.7.0-dev.2-M07-B000030`
- Branch: `agent/phase07-agent-zero-contracts`
- Classification: **UNKNOWN**

## Intended acceptance checks

1. Rust formatting: `cargo fmt --check`
2. Strict static analysis: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
3. Rust unit/integration tests: `cargo test --workspace`
4. Desktop typecheck/build
5. Windows Autonomy Governor regression checks/tests
6. Carried-forward NTPX Fortress gate when available on the Phase 07 topology

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

## Residual limits

B000030 does not claim OS/container isolation, process-group/tree termination, network isolation, filesystem isolation, syscall filtering, CPU/memory quotas, or credential-file isolation. `env_clear` only addresses ambient environment-variable inheritance. These remain Phase 07 acceptance work.

## Outcome

**UNKNOWN** — implementation candidate recorded, but executable verification did not start. Phase 07 remains IN PROGRESS and no milestone advancement is justified.
