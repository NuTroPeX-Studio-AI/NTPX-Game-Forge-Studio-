# Changelog

All notable verified changes to NTPX GameForge Ω∞ are recorded here.

## Unreleased — Phase 03 — Build B000006 candidate

### Autonomy Governor policy core
- Added a provider-neutral, engine-neutral `autonomy-governor` Rust crate.
- Added typed principals, permissions, exact resource scopes, risk levels, and approval requirements.
- Added deny-by-default deterministic policy evaluation.
- Added protected-action escalation so credential, merge, publish, destructive, and release capabilities always require human approval.
- Added deterministic audit-ready decision records.
- Added tests for default denial, exact-scope enforcement, grants, human gates, protected actions, and deterministic records.
- Phase 03 remains in progress; no tool execution or external mutation authority is introduced by this increment.

## 0.2.0-dev.2 — Phase 02 — Build B000005

### Verified GameSpec foundation
- Completed the engine-neutral GameSpec 0.1.0 typed domain contract.
- Added canonical `SpecId` validation and fixed-algorithm deterministic generated IDs.
- Added stable JSON serialization/deserialization and semantic validation.
- Added deterministic direct-reference dependency graph extraction and dependency cycle rejection.
- Added the checked-in versioned JSON Schema and CLI schema emission.
- Added migration from the verified legacy `0.1` shape to `0.1.0`.
- Added deterministic compilation manifests.
- Added `gamespec-compiler` CLI commands for validate, inspect, graph, compile, migrate, normalize, and schema.
- Added Signal Born full reference and legacy migration fixtures.
- Added asset licensing and provenance metadata.
- Added regression, migration, graph, compiler, deterministic-ID, and CLI tests.
- Verified Rust formatting, strict Clippy, workspace tests, frontend typecheck, and frontend production build.

### Boundary
- Phase 03 Autonomy Governor is the next milestone.
- No autonomous tool authority, engine adapter, model router, or release authority is introduced by Phase 02.

## 0.2.0-dev.1 — Phase 02 — Build B000003
- Added the first verified engine-neutral GameSpec core increment.
- Preserved Phase 02 as in progress pending schema, graph, migration, compiler, and full domain completion.

## 0.1.0-dev.1 — Phase 01 — Build B000002
- Established the Rust workspace and `gameforge-core`.
- Added Tauri + React + TypeScript desktop shell.
- Added CI, governance docs, ADRs, and build metadata.
- Verified the repository foundation.
