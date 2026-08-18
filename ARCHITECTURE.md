# Architecture

## Current boundary

Phase 01 and Phase 02 are verified foundations. Phase 03 is in progress. GameSpec 0.1.0 remains the canonical, engine-neutral, AI-provider-neutral machine-readable contract for reproducible game intent and project state. The Autonomy Governor begins as a policy decision and evidence boundary: it can classify a request as allowed, denied, or requiring human approval and can durably persist that decision evidence, but it performs no requested external action itself.

## Layers

1. **Studio UI** — Tauri + React desktop shell.
2. **Core services** — Rust workspace crates with explicit interfaces.
3. **Specifications** — verified GameSpec types, versioned schema, validation, dependency graph, migrations, deterministic compiler metadata, and CLI.
4. **Governance** — deny-by-default identities, permissions, exact scopes, risk classification, approval gates, durable audit evidence, transactions, and rollback (Phase 03).
5. **Intelligence** — model routing, orchestration, code intelligence, project world model, and evaluation council (Phases 04–13).
6. **Execution** — Agent Zero, engine/tool adapters, build/test/repair (Phases 07–11).
7. **Production** — assets, narrative/audio/NPCs, controlled improvement, game build, and release (Phases 14–18).

## GameSpec boundary

- `GameSpec` is engine-neutral and AI-provider-neutral.
- Top-level objects use explicit stable IDs and explicit references.
- Deterministic generated IDs use a fixed algorithm and stable canonical serialization.
- Dependency-bearing references compile into a deterministic graph and dependency cycles are rejected.
- Legacy `0.1` documents can migrate to the current `0.1.0` contract.
- Asset records carry licensing and provenance metadata.
- Engine adapters must consume validated/compiled GameSpec data rather than create untracked canonical state.

## Autonomy Governor boundary

- Policy evaluation is deny-by-default.
- Principal, permission, and resource scope matching is exact in the initial increment.
- Protected credential, merge, publish, destructive, and release capabilities always require explicit human approval.
- Decision records are deterministic and can be persisted as append-only JSONL audit entries.
- Audit entries are SHA-256 hash-chained by sequence and previous hash; existing entries are fully verified before append and corruption fails closed.
- The hash chain is tamper-evident, not authenticated: unrestricted rewrite access could recreate the full log and recompute hashes.
- Audit persistence does not execute the governed request or grant any additional capability.
- Approval-proof verification and replay prevention, transactional writes, rollback snapshots, and runtime enforcement remain Phase 03 work.

## Non-negotiable boundaries

- Shared core crates must not depend on a specific game engine.
- Shared core crates must not depend on a specific AI provider.
- Project state that must be reproducible cannot live only in model memory.
- Destructive, credentialed, financial, production, and public-release actions require human authorization.
- Every automated write must ultimately be auditable and reversible.
- GameSpec validation does not grant agents permission to execute tools or change repositories.

## Repository layout

```text
apps/                  User-facing applications
crates/                Shared Rust core services, governance, and GameSpec compiler
schemas/               Versioned GameSpec JSON Schema contracts
examples/              Reference and migration fixtures
docs/                  ADRs and verification evidence
.github/workflows/      CI validation
.gameforge/             Build/version metadata
```
