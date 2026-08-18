# ADR 0010 — Autonomy Governor runtime completion boundary

## Status

Accepted for Phase 03 completion.

## Context

The verified Phase 03 policy, audit, approval-proof, replay-prevention, transaction, and rollback primitives required a final runtime boundary before later agent/model/engine layers can safely request execution. The remaining work also included canonical filesystem scope resolution, multi-file consistency, crash-recovery discovery, and cross-platform replacement behavior.

## Decision

1. `GovernanceRuntime` is the in-process enforcement boundary. A supplied operation runs only after policy evaluation and durable authorization evidence. Denials are audited; human-gated requests require a valid one-time request-bound approval proof.
2. Governed filesystem writes require an absolute target, reject symlink ancestors and symlink targets, canonicalize the parent, bind policy scope to the canonical resource string, capture parent identity, and revalidate that identity immediately before entering the transaction primitive.
3. `GovernedFileBatch` coordinates multiple governed writes through `GovernanceRuntime`. Canonical duplicate targets fail before mutation. A later member failure rolls earlier successful members back in reverse order. This is all-or-reverted for ordinary failures, not kernel-level multi-file atomicity.
4. `RecoveryScanner` is conservative and read-only during discovery. It never guesses whether a retained verified snapshot represents a committed transaction or an interrupted transaction. Recovery is explicit, re-verifies SHA-256 integrity and target binding, rejects symlink/non-file artifact substitution, and compares existing snapshot identity canonically so Windows 8.3/long-path aliases do not cause false mismatches.
5. Normal transaction restoration and recovery share one platform-aware replacement primitive. Unix-like systems use same-filesystem rename replacement. Windows first attempts rename and uses a reversible target-backup fallback when an existing target blocks direct replacement. The Windows fallback is failure-safe but is not claimed to be kernel-atomic across the two renames.
6. CI must compile and test `autonomy-governor` on Windows in addition to the full Ubuntu workspace gate.

## Security boundary

- Canonical path resolution and parent identity revalidation materially reduce lexical-alias, symlink, and parent-replacement risk, but they do not pin a directory handle across the complete mutation sequence. A non-cooperating attacker with concurrent directory-replacement capability could still race after the final identity check. This residual TOCTOU limitation is documented rather than hidden.
- Audit/transaction locks are cooperative advisory locks. Processes bypassing the governor APIs are outside the guarantee.
- Recovery snapshots and audit hash chains are tamper-evident, not authenticated against an attacker who can fully rewrite all relevant state and recompute digests.
- Phase 03 introduces no production credential use, network access, repository merge authority, engine execution, public publishing, or release authority. Later adapters must consume the governor boundary rather than bypass it.

## Consequences

Phase 03 can be considered complete once the exact stamped build passes Ubuntu formatting/strict Clippy/workspace tests, frontend typecheck/build, and Windows governor check/tests. Phase 04 may build model routing on top of this boundary without inheriting execution authority automatically.
