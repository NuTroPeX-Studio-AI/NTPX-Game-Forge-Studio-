# Phase 03 — Build B000006 evidence

Status: CANDIDATE

Build: `NTPX-GFΩ-G0-v0.3.0-dev.1-M03-B000006`

## Scope
First Autonomy Governor increment: typed principals, permissions, exact tool/filesystem/network/credential/repository/release scopes, risk levels, human-approval gates, deny-by-default evaluation, protected-action escalation, and deterministic audit-ready decision records.

## Acceptance commands
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- frontend typecheck and production build regression gates from repository CI

## Required behavior
- unknown principal is denied;
- missing grant is denied;
- low-risk exact grant can allow;
- grants do not escape exact scope;
- explicit human approval gates are preserved;
- credential/merge/publish/destructive/release capabilities cannot be auto-authorized;
- decision records serialize deterministically.

## Classification
UNKNOWN until the exact candidate commit passes CI.

## Phase status
Phase 03 remains IN PROGRESS. Durable audit storage, approval proof verification, transactional writes, rollback snapshots, policy persistence, and runtime enforcement remain pending.
