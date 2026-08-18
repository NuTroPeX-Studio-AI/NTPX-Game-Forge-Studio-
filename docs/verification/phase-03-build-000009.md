# Phase 03 — Build B000009 evidence

Status: VERIFIED

Build: `NTPX-GFΩ-G0-v0.3.0-dev.4-M03-B000009`

## Increment

Cross-process Autonomy Governor audit writer exclusion.

## Acceptance evidence

Implementation candidate commit `728556cf822f820d2d9c382441bf9ad9f82c4972` passed GitHub CI run `32180449557`:

- `cargo fmt --check` — PASS
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — PASS
- `cargo test --workspace` — PASS
- frontend TypeScript typecheck — PASS
- frontend production build — PASS

The workspace test gate includes the new multi-process audit integration test: four independent processes each append eight records to one shared audit path, after which the parent verifies one continuous 32-entry sequence and SHA-256 hash chain.

## Security properties verified by tests and code path

- every audit mutation acquires the same OS-backed exclusive advisory sibling-file lock;
- existing audit-chain verification and next-sequence allocation occur while that lock is held;
- approval nonce lookup and approved-record append occur atomically under that lock;
- existing corruption still fails closed before append;
- the audit append still calls `sync_data` before releasing the writer critical section.

## Boundaries

The lock coordinates cooperating GameForge processes using the Autonomy Governor audit API. It does not prevent a process with filesystem access from bypassing the API, ignoring advisory locking, or rewriting the complete audit log. The existing hash chain remains tamper-evident rather than authenticated.

This increment introduces no governed external execution, production credential use, destructive action, or protected release authority.

Phase 03 remains IN PROGRESS. Transactional governed writes, rollback snapshots, and runtime enforcement remain pending.
