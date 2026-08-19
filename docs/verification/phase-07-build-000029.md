# Phase 07 — Build B000029 verification

Status: UNKNOWN (implementation CI verified; required Fortress gate unavailable on the stacked branch topology)

Build: `NTPX-GFΩ-G0-v0.7.0-dev.1-M07-B000029`

## Acceptance evidence

- [x] provider-neutral Agent Zero execution-contract crate added;
- [x] explicit tool/program/resource identity;
- [x] bounded wall-time, cancellation grace, stdout/stderr size, and dynamic-argument declarations;
- [x] environment request keys fail closed unless allowlisted;
- [x] exact execution principal matches Governor request principal;
- [x] exact `Permission::Execute` required;
- [x] exact `ScopeKind::Tool` required;
- [x] exact governed tool resource required;
- [x] deterministic contract/request SHA-256 evidence;
- [x] explicit lifecycle and cancellation state machine;
- [x] invalid lifecycle transitions fail closed;
- [x] tests added for all new behavior;
- [x] repaired implementation Rust formatting passes;
- [x] repaired implementation strict Clippy passes with warnings denied;
- [x] repaired implementation full workspace tests pass;
- [x] repaired implementation desktop TypeScript typecheck/build passes;
- [x] repaired implementation Windows Autonomy Governor checks/tests pass;
- [ ] NTPX Fortress gate executes on this exact candidate head;
- [ ] runtime process isolation/spawn is implemented;
- [ ] declared time/output/cancellation limits are enforced against a real child process;
- [ ] governed authorization is consumed immediately before process creation;
- [ ] Phase 07 acceptance criteria are complete.

## Executable evidence

Initial CI `32262392263` found rustfmt-only layout drift in `crates/agent-zero-execution/src/lib.rs`. The exact formatter-equivalent layout was applied without suppressing linting or weakening behavior.

Repaired head `0738494d547c2332b5db5a7d0c1288df44b1bcc9` passed CI `32262553710`: formatting, strict Clippy, complete Rust workspace tests, desktop frontend typecheck/build, and Windows Autonomy Governor regression checks/tests.

## Fortress limitation

The repository's Fortress workflow is present on `main` and listens to `pull_request`, but PR #20 is intentionally stacked on the unmerged Phase 06 branch, which does not contain that workflow file. GitHub therefore produced no Fortress workflow run for this candidate. This is treated as missing verification, not as a pass.

## Claim boundary

B000029 verifies deterministic execution contracts and lifecycle validation only. It does not verify process spawning, OS/container sandboxing, runtime timeout/output enforcement, signal/kill semantics, environment scrubbing beyond request-key validation, filesystem mount isolation, network isolation, credential handling, engine execution, or release behavior.
