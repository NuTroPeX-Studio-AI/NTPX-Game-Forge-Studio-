# Architecture

## Phase 01 boundary

Phase 01 establishes a compilable, testable, cross-platform repository foundation. It intentionally does not implement GameSpec, autonomous agents, model providers, Unreal automation, Hermes, OpenClaw, Agent Zero, or self-improvement.

## Layers

1. **Studio UI** — Tauri + React desktop shell.
2. **Core services** — Rust workspace crates with explicit interfaces.
3. **Specifications** — GameSpec compiler and schemas (Phase 02).
4. **Governance** — permissions, policy, audit, transactions, rollback (Phase 03).
5. **Intelligence** — model routing, orchestration, project world model (Phases 04–13).
6. **Execution** — Agent Zero, engine/tool adapters, build/test/repair (Phases 07–11).
7. **Production** — assets, narrative/audio/NPCs, controlled improvement, game build, release (Phases 14–18).

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
docs/                  ADRs and verification evidence
.github/workflows/      CI validation
.gameforge/             Build/version metadata
```
