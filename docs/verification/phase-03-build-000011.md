# Phase 03 — Build B000011 evidence

Status: VERIFIED

Build: `NTPX-GFΩ-G0-v0.3.0-dev.6-M03-B000011`

## Scope

Request-bound human-approved exact-path local file transactions that preserve deny-by-default policy, verify approval proofs before mutation, persist approval evidence, atomically consume one-time approval nonces through the durable audit append, serialize cooperating same-target writers, and automatically restore pre-write state when replay or audit failure is discovered after replacement.

## Acceptance evidence

Implementation candidate CI run `32191076927` passed:

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- frontend TypeScript typecheck
- frontend production build

Behavioral coverage includes successful approved commit with persisted evidence, explicit rollback, replay rejection with restoration of the prior committed state, mismatched/expired proof rejection before mutation, and proof inability to override denial or manufacture authority for an already-direct-allow path.

## Safe repair evidence

Earlier candidate runs exposed only formatter and strict-Clippy findings. Repairs applied formatter-prescribed changes, removed an unused import, and replaced an over-wide public function signature with a typed `ApprovalContext`. No lint, test, approval, replay, audit, or rollback control was weakened or suppressed.

## Security and rollback classification

- Approval proof verification occurs before mutation.
- The approved transaction path requires an existing `RequireHumanApproval` policy decision.
- One-time nonce consumption and approval evidence persistence occur in the verified cross-process locked audit append.
- A replay discovered after replacement restores the pre-write snapshot before returning failure.
- A deterministic sibling advisory transaction lock serializes cooperating processes operating on the same target.
- Direct-allow transactions remain a separate path and approval cannot override denial.
- Canonical path containment, parent-directory race hardening, crash recovery, multi-file atomicity, stronger platform-specific replacement semantics, and general runtime enforcement remain outside this increment.

## Finalization rule

B000011 is merged only if the exact stamped branch head passes the same complete CI gate. Phase 03 remains IN PROGRESS.
