# ADR 0027 — Agent Zero execution contract boundary

Status: Accepted for B000029 candidate

## Context

Phase 06 established governed, reversible source mutation. Phase 07 must introduce external process/tool execution without allowing scheduling, model selection, or execution planning to become authorization.

## Decision

Introduce `agent-zero-execution` as a provider-neutral and engine-neutral execution-contract layer. A `ToolContract` declares a fixed program identity, fixed arguments, an exact governed tool resource, environment-variable allowlist, and bounded wall-time/cancellation/output/dynamic-argument limits. An `ExecutionRequest` is accepted only when its principal, `Permission::Execute`, `ScopeKind::Tool`, and exact resource match the declared contract and the predeclared Governor request.

Validated contracts produce deterministic SHA-256 correlation evidence and an explicit lifecycle state machine (`planned`, `authorized`, `running`, cancellation, terminal outcomes). Validation does not produce an authorization permit.

## Authority boundary

B000029 does not create processes, invoke a shell, inherit ambient credentials, open network connections, mutate repositories, execute an engine, publish, sign, merge, or release. A later worker implementation must obtain exact Autonomy Governor authorization immediately before process creation and must enforce the declared runtime limits rather than merely recording them.

Environment expansion fails closed: request-supplied keys must be present in the contract allowlist. No ambient secret inheritance is implied by an allowlist entry.

## Consequences

The next Phase 07 increment can implement a narrow isolated worker against a stable contract without redesigning authorization semantics. Until runtime enforcement is implemented, wall-time, output, cancellation-grace, and isolation limits are declarations rather than verified process-level guarantees.

## Verification boundary

The repaired B000029 implementation head `0738494d547c2332b5db5a7d0c1288df44b1bcc9` passed CI `32262553710` for Rust formatting, strict Clippy, full workspace tests, desktop typecheck/build, and Windows Autonomy Governor regressions. The carried-forward NTPX Fortress workflow did not execute on the stacked Phase 07 PR because its Phase 06 base branch predates the workflow file present on `main`; therefore B000029 remains a candidate and is not classified VERIFIED.
