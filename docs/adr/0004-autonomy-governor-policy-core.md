# ADR 0004 — Autonomy Governor policy core

## Status
Accepted for Phase 03 implementation.

## Decision
Phase 03 begins with a provider-neutral, engine-neutral Rust policy core that is deny-by-default. Authorization matching is exact on principal, permission, and scope. Wildcards are intentionally excluded from the first increment. Protected capabilities always resolve to `RequireHumanApproval`, even when a grant is otherwise permissive.

The governor returns decisions and audit-ready decision records but performs no tool, filesystem, network, credential, repository, merge, release, or destructive operation itself.

## Rationale
Starting Phase 03 with a pure deterministic decision boundary makes later agent/tool integration testable without accidentally granting execution authority. Exact scopes minimize privilege ambiguity and preserve least privilege.

## Deferred
Transactional writes, rollback snapshots, durable audit storage, approval-token verification, policy persistence, scope hierarchies, and runtime integration remain Phase 03 work.
