# Phase 03 — Build B000012 evidence

Status: VERIFIED

Build: `NTPX-GFΩ-G0-v0.3.0-dev.7-M03-B000012`

## Scope

Fail-closed ancestor-path hardening for governed local file transactions. Direct commits, human-approved commits, and rollback now reject any existing symlink in the target's ancestor chain before governed mutation or restoration proceeds.

## Acceptance evidence

Implementation candidate CI run `32195793642` passed:

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- frontend TypeScript typecheck
- frontend production build

Behavioral coverage includes a Unix integration test that creates a real parent directory plus a symlinked alias, targets a file through the symlinked parent, and verifies the transaction returns `TransactionError::SymlinkAncestor`, does not create the real target, and does not append an audit record.

## Security and rollback classification

- Existing symlink ancestors are detected with `symlink_metadata` and rejected before snapshot creation, staging, replacement, or audit append.
- The check is shared by direct commits, approved commits, and rollback.
- Exact lexical `Filesystem` scope matching remains unchanged.
- Existing symlink-target rejection remains in place.
- No credential, merge, publish, release, network, engine, or general tool authority is introduced.
- This increment does not claim complete TOCTOU protection; canonical containment, descriptor-relative traversal, full parent-directory race hardening, crash recovery, multi-file atomicity, stronger platform-specific replacement semantics, and runtime enforcement remain outside this build.

## Build progression

B000012 advances the continuous build lineage only after the implementation candidate passed the full repository CI gate. Phase 03 remains IN PROGRESS and no protected release is published.

## Finalization rule

The stamped branch head must pass the same complete CI gate before merge. Phase 03 advances only when all remaining Phase 03 acceptance criteria are verified.
