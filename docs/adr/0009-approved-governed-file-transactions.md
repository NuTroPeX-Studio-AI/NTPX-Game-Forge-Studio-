# ADR 0009 — Approved governed file transactions

Status: Accepted

## Context

Phase 03 already had two independently verified primitives: exact request-bound HMAC human approval proofs with durable one-time nonce replay prevention, and exact-path governed local file transactions with durable rollback snapshots. The transaction engine deliberately rejected `RequireHumanApproval` decisions, leaving no safe bridge for a human-approved write to proceed without bypassing policy or duplicating approval logic outside the governor.

## Decision

Add a typed `ApprovalContext` and a separate `GovernedFileTransaction::commit_with_approval` path.

The approved transaction path:

1. requires exact `Filesystem` + `Write` request/target matching;
2. re-evaluates policy and proceeds only when the result is already `RequireHumanApproval`;
3. verifies the exact request-bound approval proof and lifetime before filesystem mutation;
4. persists verified approval evidence in the final decision record;
5. consumes the one-time approval nonce through the same cross-process locked audit append that records the successful transaction;
6. restores the durable pre-write snapshot if replay or audit failure is discovered after replacement; and
7. serializes cooperating processes targeting the same file with a deterministic sibling advisory transaction lock across snapshot, stage, replace, audit, failure restoration, and explicit rollback.

The existing direct-allow `commit` path remains separate. An approval proof cannot turn a denial into an allow and cannot be used to enter the approved transaction API when policy already grants direct authority.

## Security boundary

- Approval keys remain external inputs and are never persisted by the transaction engine.
- Proof verification happens before mutation, but nonce consumption is finalized by the durable audit append after replacement; rollback closes the fail-closed window if another cooperating process consumed the nonce first.
- Per-target and audit locks are cooperative advisory locks. Processes bypassing the governor are outside the guarantee.
- The target path is still matched lexically. Canonical path containment and parent-directory symlink/race hardening are not solved by this ADR.
- Rollback snapshots and audit hashes remain tamper-evident rather than authenticated against unrestricted filesystem rewrite access.
- This decision grants no general tool, network, repository, credential, engine, or release authority.

## Consequences

Human-approved local file writes can now use the same verified governance, audit, replay-prevention, and rollback boundaries as direct governed writes. Phase 03 still requires path hardening, crash recovery, multi-file transactions, stronger platform replacement semantics, and runtime enforcement before the Autonomy Governor milestone can be considered complete.
