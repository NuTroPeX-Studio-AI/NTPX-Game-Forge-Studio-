# Changelog

All notable verified changes to NTPX GameForge Ω∞ are recorded here.

## Unreleased

### Phase 02 — GameSpec
- VERIFIED B000003: initial engine-neutral `gamespec` Rust crate.
- B000004 candidate expands GameSpec to the complete Phase 02 domain contract.
- Added canonical IDs and fixed-algorithm deterministic generated IDs.
- Added full project/architecture/features/components/entities/inputs/abilities/NPC/dialogue/quest/world/asset/test/build domains.
- Added direct-reference graph extraction and dependency cycle detection.
- Added versioned JSON Schema and schema CLI emission.
- Added migration from the verified legacy `0.1` GameSpec shape to `0.1.0`.
- Added deterministic compiler manifest and `gamespec-compiler` CLI.
- Added Signal Born full reference specification, legacy fixture, and regression tests.

## 0.1.0-dev.1 — Phase 01
- Established Rust workspace and `gameforge-core`.
- Added Tauri + React + TypeScript desktop shell.
- Added CI, governance docs, ADRs, and build metadata.
- Verified foundation as build B000002.
