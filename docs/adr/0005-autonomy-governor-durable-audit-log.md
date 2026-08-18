# ADR 0005 — Autonomy Governor durable audit log

Status: Accepted

## Context

Phase 03 already produces deterministic authorization decision records, but those records need durable evidence before approval proof verification, replay prevention, and transactional runtime enforcement can safely depend on them.

## Decision

Persist authorization decisions as append-only JSON Lines records. Each entry carries a monotonic sequence number, the previous entry hash, the original deterministic `DecisionRecord`, and a SHA-256 digest over the canonical serialized audit payload.

Before appending a new entry, the complete existing log is read and verified. Malformed JSON, sequence discontinuity, previous-hash mismatch, or digest mismatch fails closed and prevents extension. Successful appends are flushed with `sync_data`.

The audit store records authorization evidence only. It does not execute tools, use credentials, mutate repositories, merge code, publish artifacts, or release software.

## Security boundary

The hash chain is tamper-evident rather than cryptographically authenticated. A principal with unrestricted ability to rewrite the entire audit file can recompute the chain. A later security increment may anchor or sign audit heads when a suitable key-management and approval model is verified.

Concurrent multi-process writers are not yet supported. Transactional locking, write snapshots, rollback, approval-proof replay protection, and runtime enforcement remain Phase 03 work.

## Consequences

- Policy decisions now have durable integrity-checked evidence.
- Corrupt logs fail closed before append.
- No new autonomous execution authority is introduced.
- Later approval and transaction layers can reference a stable audit sequence/hash lineage.
