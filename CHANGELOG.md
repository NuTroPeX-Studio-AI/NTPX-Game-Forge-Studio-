# Changelog

All notable changes to NTPX GameForge Ω∞ are recorded here.

## [0.1.0-dev.2] - 2026-08-18

### Phase 02 candidate — Build B000003
- Added the engine-neutral `gamespec` Rust crate.
- Added versioned GameSpec 0.1.0 domain types.
- Added canonical `SpecId` validation.
- Added serialization/deserialization and direct-reference validation.
- Added JSON Schema contract for GameSpec 0.1.0.
- Added Signal Born: First Thought reference GameSpec.
- Added licensing and provenance fields for assets.
- Added round-trip and invalid-document tests.
- Added ADR 0002 and Phase 02 verification checklist.

### Scope
- GameSpec compiler graph construction, cycle detection, migrations, and engine code generation remain deferred.

## [0.1.0-dev.1] - 2026-08-18

### Verified foundation — Build B000002
- Rust formatting passed.
- Strict Clippy passed with warnings denied.
- Rust tests passed.
- Frontend dependency installation passed.
- TypeScript typecheck passed.
- Vite production build passed.
- Phase 01 acceptance criteria satisfied.

### Added
- Rust workspace foundation.
- `gameforge-core` crate with build identity and phase metadata.
- Tauri + React + TypeScript desktop shell.
- pnpm workspace configuration.
- GitHub Actions CI for Rust formatting, linting, tests, and frontend type/build checks.
- Architecture, roadmap, security, agent-policy, ADR, and verification documentation.
- Continuous build lineage.

### Scope
- Phase 01 only. No GameSpec, autonomous execution, engine integration, or AI-provider implementation yet.
