# Security Policy

## Default posture

NTPX GameForge Ω∞ is offline-first, least-privilege, and deny-by-default.

## Protected actions

The following always require explicit human authorization:

- production publishing
- signing-key or protected credential use
- credential changes
- destructive migrations or data deletion
- protected-branch merges when policy requires review
- security-policy or permission-root changes
- paid infrastructure purchases
- public release promotion

## Verified Autonomy Governor guarantees

- Unknown principals and unmatched permissions/scopes fail closed.
- Protected capability classes preserve a human-approval gate even if a grant would otherwise allow the action.
- Human approval proofs are HMAC-SHA256 authenticated, bound to the exact request, expire, and carry one-time nonces.
- Approval nonces are durably consumed through the cross-process locked audit append and cannot be replayed by cooperating processes.
- Authorization records are append-only SHA-256 hash-chained JSONL and are fully verified before append.
- In-process runtime operations supplied to `GovernanceRuntime` execute only after durable authorization evidence; denied operations are audited and not invoked.
- Governed file mutation requires an absolute canonical scope, rejects symlink ancestors and symlink targets, and revalidates parent identity before entering the transaction boundary.
- Governed writes persist durable pre-write snapshots, stage bytes durably, replace through a platform-aware primitive, record authorization evidence, and automatically restore the pre-write state when audit/replay failure occurs after replacement.
- Human-approved file writes verify the exact proof before mutation and cannot use a proof to override denial.
- Multi-file batches reject duplicate canonical targets before mutation and roll earlier members back if a later member fails.
- Crash-recovery discovery is read-only. Explicit recovery verifies snapshot integrity/target binding and rejects corrupt or symlink-substituted artifacts.
- The Autonomy Governor is compiled and tested on Windows in addition to the full Ubuntu workspace CI gate.

## Residual security boundaries

- Advisory audit/transaction locks coordinate cooperating processes only. An actor bypassing governor APIs is outside the locking guarantee.
- Audit and rollback SHA-256 chains/digests are tamper-evident, not authenticated against an actor able to rewrite all relevant files and recompute hashes.
- Canonical path resolution plus parent identity revalidation reduces path-alias, symlink, and directory-replacement risk but does not pin a directory handle for the entire mutation. Hostile TOCTOU races are therefore not claimed to be mathematically eliminated.
- The Windows replacement fallback is reversible/failure-safe but not kernel-atomic across the target-to-backup and stage-to-target renames.
- Multi-file batches are all-or-reverted for ordinary failures, not kernel-level atomic across multiple files or crash points.
- Verified rollback snapshots are intentionally not auto-restored because a retained snapshot can represent a successful reversible transaction as well as interrupted work.
- Phase 03 does not grant production credentials, network connections, repository merge rights, engine execution, publishing, or release authority. Future adapters must consume the governor rather than bypass it.

## Agent execution principles

- least privilege
- scoped filesystem access
- scoped network access
- ephemeral credentials
- complete audit trails
- authorization before execution
- transactional edits
- snapshot/rollback before high-risk changes
- fail closed on ambiguity or corrupted evidence

## Secret handling

Never commit secrets, tokens, private keys, certificates, `.env` files, or production credentials. Approval-signing secrets must remain external inputs and must not be serialized or logged. Use environment variables or an approved secret store.

## Reporting

Security issues should be documented privately with reproducible evidence, impact, affected components, and remediation status. Do not publish exploitable details before remediation.
