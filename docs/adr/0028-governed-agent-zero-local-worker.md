# ADR 0028: Governed Agent Zero local worker

## Status

Candidate — B000030, Phase 07.

## Context

B000029 validates deterministic execution/tool contracts but deliberately creates no process. Phase 07 next requires the smallest real execution boundary without weakening the verified Autonomy Governor or introducing engine/provider-specific behavior.

## Decision

Add a provider-neutral `agent-zero-worker` crate that consumes `ValidatedExecution` evidence and performs the exact `GovernanceRuntime::authorize` call immediately before process creation.

The worker:

- requires an absolute, existing, non-symlink registered executable path before spawn;
- requires an existing, non-symlink workspace directory and records its canonical path in result evidence;
- executes the registered program directly through `std::process::Command`, never through an implicit shell;
- clears the child environment and adds only environment entries already admitted by the validated execution contract;
- nulls stdin and captures stdout/stderr concurrently;
- retains no more than each declared stdout/stderr byte limit while continuing to drain the stream and hashing the complete observed stream;
- enforces wall time and a cooperative cancellation grace interval, then kills and reaps the direct child when the bound is exceeded;
- returns the Governor `AuthorizationPermit` together with deterministic execution result evidence.

Governor denial or an unsatisfied human-approval gate returns before `Command::spawn`.

## Security and authority boundary

This increment does **not** claim container, namespace, process-tree, network, filesystem, syscall, memory, CPU, or credential-file isolation. `env_clear` prevents ambient environment-variable inheritance, but a launched executable still runs with the operating-system identity and filesystem/network capabilities of the GameForge process unless a later sandbox adapter constrains them.

For that reason B000030 is suitable only as the governed direct-child execution primitive. Phase 07 remains IN PROGRESS until sandbox/workspace-mount policy, process-tree termination, supported-OS isolation verification, and the carried-forward Fortress gate are separately satisfied.

The worker creates no new permission system. Tool registration and validation are evidence; only the Autonomy Governor authorizes execution.

## Consequences

- Phase 08 Unreal tooling can later consume a single governed process primitive rather than spawning independently.
- Bare executable names are rejected at the real worker boundary, avoiding ambient PATH-dependent program selection.
- Output truncation is explicit while full observed stream hashes remain available for correlation.
- Direct-child timeout/cancellation becomes executable behavior, while descendant-process cleanup remains a documented unresolved acceptance item.
