# Architecture

## Current boundary

Phase 01 and Phase 02 are verified foundations. Phase 03 is in progress. GameSpec 0.1.0 remains the canonical, engine-neutral, AI-provider-neutral machine-readable contract for reproducible game intent and project state. The Autonomy Governor is a policy, evidence, and narrowly governed local-mutation boundary: it classifies requests, verifies exact request-bound human approval proofs where required, durably records authorization evidence, rejects replayed approval nonces atomically across cooperating processes, and supports exact-path local file transactions with durable pre-write rollback snapshots. Human-approved file transactions bridge those verified approval and mutation primitives without bypassing policy: proofs are verified before mutation, approval evidence and nonce consumption are bound to the durable audit append, replay/audit failures after replacement restore the pre-write state, and existing symlink ancestors are rejected before governed mutation or rollback. It still does not provide general tool, network, repository, credential, engine, or release execution authority.

## Layers

1. **Studio UI** — Tauri + React desktop shell.
2. **Core services** — Rust workspace crates with explicit interfaces.
3. **Specifications** — verified GameSpec types, versioned schema, validation, dependency graph, migrations, deterministic compiler metadata, and CLI.
4. **Governance** — deny-by-default identities, permissions, exact scopes, risk classification, approval proofs/gates, durable audit evidence, governed file transactions, and rollback (Phase 03).
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
- Principal, permission, and resource scope matching is exact in the initial increments.
- Protected credential, merge, publish, destructive, and release capabilities always require explicit human approval.
- Human approval proofs use HMAC-SHA256 and bind an authority identifier, the exact serialized request digest, a one-time nonce, issue time, and expiry time.
- Approval secrets are external inputs, require at least 32 bytes, and are not serialized or exposed through `Debug`.
- A proof can satisfy only an existing `RequireHumanApproval` policy result; it cannot override a denial or create a grant.
- Successful approval evidence is persisted in the authorization audit record.
- Audit mutations acquire an exclusive advisory lock on a deterministic sibling `<audit-path>.lock` file. Full-chain verification, next-sequence allocation, hash construction, append, and durable flush occur while the writer lock is held.
- Approval nonce lookup and append execute as one locked operation, providing durable replay prevention across cooperating processes sharing the same audit log.
- Decision records are deterministic and can be persisted as append-only JSONL audit entries.
- Audit entries are SHA-256 hash-chained by sequence and previous hash; existing entries are fully verified before append and corruption fails closed.
- The hash chain is tamper-evident, not authenticated: unrestricted rewrite access could recreate the full log and recompute hashes.
- The audit writer lock is cooperative/advisory. A process that bypasses the Autonomy Governor API or ignores advisory locking is outside this guarantee.
- Exact-path governed local file transactions re-evaluate policy internally and require `Permission::Write` on an exact lexical `Filesystem` scope matching the target.
- Direct transactions execute only on a policy `Allow`. Human-approved transactions execute only when the same policy evaluation yields `RequireHumanApproval` and an exact request-bound proof verifies before mutation.
- Human-approved transactions persist approval evidence and consume the proof nonce through the same atomic audit operation. A replay detected after replacement triggers restoration from the durable pre-write snapshot before returning failure.
- Governed transactions use a deterministic sibling per-target advisory lock to serialize cooperating processes across snapshot capture, staging, replacement, audit/nonce consumption, failure restoration, and explicit rollback.
- Governed file transactions reject symlink targets and any existing symlink ancestor, persist a same-directory SHA-256-protected pre-write snapshot, durably flush staged bytes before rename, append the authorization record after replacement, and automatically restore the pre-write state if audit persistence fails.
- Ancestor symlink rejection closes lexical path traversal through an already-symlinked parent but is not a complete TOCTOU defense; descriptor-relative containment and parent-directory race hardening remain future Phase 03 work.
- Successful file commits retain their rollback snapshot and return a receipt for explicit restoration. Consumed snapshots are removed only after successful rollback.
- Snapshot integrity is tamper-evident, not authenticated. Canonical containment, full parent-directory race hardening, crash recovery, multi-file atomicity, and platform-specific replacement semantics remain future Phase 03 work.
- Audit/approval/transaction persistence does not grant general tool, network, repository, credential, engine, or release authority.
- Runtime enforcement remains Phase 03 work.

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
