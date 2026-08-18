# ADR 0008 — Transactional governed local file writes

Status: Proposed for B000010 verification

## Decision

Phase 03 introduces a narrow local-file transaction primitive inside `autonomy-governor`. The primitive accepts an immutable `Policy` and `Request`, reevaluates authorization itself, and proceeds only when the request is an exact `Permission::Write` grant on the exact lexical `Filesystem` scope matching the target path and the resulting decision is `Allow`.

Before mutation it writes a durable same-directory rollback snapshot containing the target path, existence flag, original bytes, and a SHA-256 integrity digest. New bytes are written to a same-directory staging file and durably flushed before rename into place. The authorization decision is then appended to the verified audit log. If audit persistence fails after replacement, the transaction immediately restores the pre-write snapshot. A successful commit retains its snapshot and returns a receipt that can be used for explicit rollback.

## Boundaries

- This increment supports ordinary local files only.
- Symlink targets are rejected.
- Transaction IDs are restricted to a bounded ASCII identifier grammar and are used only for sibling staging/snapshot names.
- The transaction primitive does not bypass `RequireHumanApproval`; requests that still require approval fail closed.
- Snapshot SHA-256 detects accidental/post-write snapshot corruption but is not an authenticity guarantee against an attacker with unrestricted rewrite access.
- Scope matching is lexical and exact in this increment; canonical path containment, parent-directory symlink/race hardening, multi-file transactions, crash-recovery scanning, and approved-proof execution remain future work.
- Runtime tool/network/repository/release execution remains outside this increment.

## Rationale

The verified policy, approval, audit, and writer-lock foundations now provide enough prerequisites to introduce the first reversible mutation boundary without granting general runtime execution authority. Keeping the first transaction primitive to one exact local path makes its security properties testable and preserves a narrow rollback surface.
