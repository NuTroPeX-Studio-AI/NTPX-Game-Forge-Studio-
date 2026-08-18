# ADR 0010 — Reject symlink ancestors for governed file transactions

Status: Accepted

## Context

Phase 03 governed file transactions already rejected a symlink target, but an exact lexical target could still traverse an existing symlink in an ancestor directory. That leaves a path-redirection gap even when policy scope matching itself is exact.

## Decision

Before commit, approved commit, or rollback, the transaction boundary inspects each existing ancestor with `symlink_metadata`. If any ancestor is a symlink, the operation fails closed with `TransactionError::SymlinkAncestor` before snapshot creation, staging, replacement, or audit append.

The check intentionally preserves lexical exact-scope matching and does not canonicalize the request into a different authority target.

## Consequences

- Governed writes cannot intentionally traverse symlinked parent directories.
- Direct-allow and human-approved transaction paths receive the same hardening through the shared target validator.
- Rollback is also blocked if the ancestor chain has been redirected through a symlink.
- This is not a complete TOCTOU defense: an attacker able to replace ancestor directories between validation and filesystem operations remains outside this increment.
- Canonical containment, descriptor-relative path traversal, crash recovery, multi-file transactions, stronger platform replacement semantics, and runtime enforcement remain pending Phase 03 work.
