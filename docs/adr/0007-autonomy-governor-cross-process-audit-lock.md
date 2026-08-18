# ADR 0007 — Cross-process Autonomy Governor audit writer lock

## Status
Accepted for Phase 03 build B000009 candidate.

## Context
The verified B000008 approval path rejects replayed approval nonces by reading the durable audit chain before append. That is safe for serialized writers but leaves a time-of-check/time-of-use race when multiple GameForge processes share one audit log: two processes could both observe an unused nonce and then append independently.

Audit sequence allocation has the same race because each writer derives the next sequence from the verified current log.

## Decision
All audit mutations acquire an exclusive advisory lock on a deterministic sibling file `<audit-path>.lock` using OS-backed file locking. The lock covers full-chain verification, next-sequence allocation, hash construction, append, and durable flush.

Approval nonce consumption uses a dedicated atomic audit operation that performs nonce lookup and append under the same writer lock. A previously consumed nonce produces no append.

Read-only verification remains lock-free so readers do not unnecessarily block writers.

## Security boundary
The lock coordinates cooperating processes that use the Autonomy Governor audit API. It does not protect against an attacker who bypasses the API, ignores advisory locking, or has unrestricted filesystem rewrite privileges. Existing hash-chain integrity checks remain fail-closed.

The lock adds no execution authority and performs no governed external action.

## Consequences
- concurrent cooperating writers cannot allocate duplicate audit sequence numbers;
- approval replay prevention becomes atomic across cooperating processes;
- a separate lock file remains beside the audit log after use;
- transactional governed writes, rollback snapshots, and runtime enforcement remain pending Phase 03 work.
