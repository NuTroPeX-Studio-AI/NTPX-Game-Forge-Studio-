# Phase 03 — Build B000010 evidence

Status: VERIFIED

Build: `NTPX-GFΩ-G0-v0.3.0-dev.5-M03-B000010`

## Scope

Governed transactional local-file writes with durable rollback snapshots, exact-path authorization binding, symlink-target refusal, same-directory staging, verified audit-on-commit, automatic restoration when audit persistence fails, and explicit rollback receipts.

## Acceptance evidence

Implementation candidate CI run `32186220917` passed:

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- frontend TypeScript typecheck
- frontend production build

Behavioral tests cover existing-file commit/rollback, new-file rollback, deny-before-mutation, exact scope mismatch, automatic restoration after corrupt-audit failure, and preservation of human-approval gates.

## Security and rollback classification

- Exact lexical filesystem scope is required.
- Only a direct policy `Allow` can execute this initial transaction path.
- Human-approval-required requests are rejected rather than bypassed.
- Symlink targets are rejected.
- Rollback snapshots include SHA-256 integrity evidence and persist until consumed by explicit rollback.
- Audit failure after replacement triggers immediate rollback.
- Snapshot/hash protections are tamper-evident, not authenticated against unrestricted rewrite access.

## Remaining Phase 03 work

Runtime enforcement, approved-proof transaction execution, canonical path containment and parent-directory race hardening, crash-recovery scanning, multi-file transactions, and stronger cross-platform replace semantics remain pending. Phase 03 therefore remains IN PROGRESS.
