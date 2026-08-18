# ADR 0006 — Request-bound human approval proofs

Status: Accepted for Phase 03 increment B000008

## Context

The verified Autonomy Governor can deny, allow, or require explicit human approval and can persist tamper-evident authorization evidence. It still needs a way to prove that a specific human-gated request was approved without turning one approval into reusable authority.

## Decision

- Human approval proofs are HMAC-SHA256 authenticated using a secret supplied by an external approved human-approval boundary.
- Approval keys must be at least 32 bytes and are never serialized or exposed through `Debug`.
- Each proof binds the authority identifier, exact serialized request digest, one-time nonce, issue time, and expiry time.
- A proof can satisfy only an existing `RequireHumanApproval` decision. It cannot override denial and cannot create a grant.
- Successful approval evidence is persisted in the existing verified audit chain.
- Replay detection scans verified prior audit entries and rejects a previously consumed nonce.
- Missing, malformed, forged, future, expired, mismatched, or replayed proofs fail closed before an approved decision is appended.

## Security boundary

This increment provides durable replay prevention for serialized writers and HMAC authenticity relative to the configured approval secret. It does not provide cross-process locking or distributed single-use guarantees. Concurrent-writer exclusion, transactional governed writes, rollback snapshots, and runtime execution enforcement remain Phase 03 work.

The signer helper does not establish that a human actually approved an action. Production callers must invoke signing only behind an explicit human approval UI/service and approved secret-management boundary. No production secret is committed to the repository.

## Consequences

The governor can now convert a human-gated policy result into an auditable allow decision only when an exact, valid, unexpired, unused approval proof is supplied. The crate still performs no governed external action itself.
