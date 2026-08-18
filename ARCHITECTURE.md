# Architecture

## Current boundary

Phase 01 established the compilable, testable repository foundation. Phase 02 adds the engine-neutral GameSpec canonical model and direct validation layer. It intentionally does not implement autonomous execution, model providers, Unreal automation, Hermes, OpenClaw, Agent Zero, or self-improvement.

## Layers

1. **Studio UI** — Tauri + React desktop shell.
2. **Core services** — Rust workspace crates with explicit interfaces.
3. **Specifications** — versioned GameSpec types and schema; compiler work continues in Phase 03.
4. **Governance** — permissions, policy, audit, transactions, rollback (Phase 03 in the master roadmap after GameSpec verification).
5. **Intelligence** — model routing, orchestration, project world model (Phases 04–13).
6. **Execution** — Agent Zero, engine/tool adapters, build/test/repair (Phases 07–11).
7. **Production** — assets, narrative/audio/NPCs, controlled improvement, game build, release (Phases 14–18).

## GameSpec boundary

- `GameSpec` is engine-neutral and AI-provider-neutral.
- Top-level objects use explicit stable IDs and explicit references.
- Reproducible project facts belong in GameSpec or other versioned project data, not only in model memory.
- Asset records include licensing and provenance metadata.
- Phase 02 validates direct references; deeper dependency compilation and cycle analysis are deferred.

## Non-negotiable boundaries

- Shared core crates must not depend on a specific game engine.
- Shared core crates must not depend on a specific AI provider.
- Project state that must be reproducible cannot live only in model memory.
- Destructive, credentialed, financial, production, and public-release actions require human authorization.
- Every automated write must ultimately be auditable and reversible.

## Repository layout

```text
apps/                  User-facing applications
crates/                Shared Rust core services
schemas/               Versioned GameSpec JSON Schema contracts
examples/              Reference GameSpec projects
docs/                  ADRs and verification evidence
.github/workflows/      CI validation
.gameforge/             Build/version metadata
```
