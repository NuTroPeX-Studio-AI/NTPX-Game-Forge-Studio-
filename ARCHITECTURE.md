# Architecture

## Current boundary

Phases 01–03 are verified foundations. GameSpec 0.1.0 is the canonical, engine-neutral, AI-provider-neutral machine-readable contract for reproducible game intent and project state. The Autonomy Governor is now the verified authorization, evidence, approval, reversible-mutation, recovery, and in-process runtime-enforcement boundary. It still does not grant general model-provider, network, repository, credential, engine, publishing, or release authority. Phase 04 Model Router is the next construction target.

## Layers

1. **Studio UI** — Tauri + React desktop shell.
2. **Core services** — Rust workspace crates with explicit interfaces.
3. **Specifications** — verified GameSpec types, versioned schema, validation, dependency graph, migrations, deterministic compiler metadata, and CLI.
4. **Governance** — verified deny-by-default identities, permissions, exact scopes, risk classification, approval proofs/gates, durable audit evidence, canonical runtime permits, governed transactions, batch rollback, recovery, and rollback (Phase 03).
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

### Policy and approval

- Policy evaluation is deny-by-default.
- Principal, permission, and resource scope matching is exact.
- Protected credential, merge, publish, destructive, and release capabilities always require explicit human approval.
- Human approval proofs use HMAC-SHA256 and bind an authority identifier, the exact serialized request digest, a one-time nonce, issue time, and expiry time.
- Approval secrets are external inputs, require at least 32 bytes, and are not serialized or exposed through `Debug`.
- A proof can satisfy only an existing `RequireHumanApproval` result; it cannot override a denial or create a grant.
- Approval evidence is persisted and nonce consumption is durable/replay-safe across cooperating processes.

### Durable evidence

- Decision records are deterministic append-only JSONL audit entries.
- Audit entries are SHA-256 hash-chained by sequence and previous hash; existing entries are fully verified before append and corruption fails closed.
- Audit mutations acquire an exclusive advisory sibling lock. Full-chain verification, sequence allocation, hash construction, append, approval-nonce lookup/consumption, and durable flush occur under that writer lock.
- The hash chain is tamper-evident, not authenticated against an actor able to rewrite the complete log and recompute hashes.
- Locks are cooperative/advisory; a process that bypasses the governor API is outside the guarantee.

### Canonical runtime enforcement

- `GovernanceRuntime` is the in-process enforcement boundary: supplied operations execute only after policy evaluation and durable authorization evidence.
- Denied runtime requests are durably audited and do not invoke the supplied operation.
- Human-gated runtime operations require a valid request-bound one-time approval proof before invocation.
- Governed filesystem targets must be absolute.
- Existing parent ancestors are inspected without following symlinks; symlink ancestors and symlink targets are rejected.
- The target parent is canonicalized, the canonical resource string must exactly match policy scope, parent identity is captured, and that identity is revalidated immediately before mutation.
- On Unix, parent identity includes device/inode. On other platforms it includes the canonical parent path.

### Reversible mutation

- Governed local-file transactions re-evaluate policy internally and require exact `Filesystem` + `Write` scope matching.
- Direct transactions execute only on policy `Allow`. Human-approved transactions execute only on `RequireHumanApproval` plus a valid exact proof verified before mutation.
- Transactions use per-target advisory locks, durable SHA-256-protected pre-write snapshots, same-directory durable staging, platform-aware replacement, durable audit append, and automatic restoration when replay/audit persistence fails after replacement.
- Successful commits retain rollback snapshots and return receipts for explicit restoration.
- `GovernedFileBatch` rejects duplicate canonical targets before mutation and routes every member through `GovernanceRuntime`. If a later member fails, earlier members are rolled back in reverse order. This is all-or-reverted for ordinary failures, not kernel-level atomic multi-file replacement.

### Recovery and platform semantics

- `RecoveryScanner` discovers rollback artifacts read-only and never automatically restores them because a retained verified snapshot may represent either an intentionally reversible committed transaction or interrupted work.
- Explicit recovery re-reads and verifies snapshot SHA-256 integrity and target binding, rejects symlink/non-file snapshot substitution, validates canonical target identity, and removes the snapshot only after successful restoration.
- Existing snapshot identity is compared canonically so Windows short/long-path aliases do not create false mismatches.
- Transaction restoration and recovery share the same replacement primitive.
- Unix-like replacement uses same-filesystem rename semantics.
- Windows first attempts rename replacement and otherwise uses a reversible target-backup fallback. The fallback is failure-safe but is not claimed to be kernel-atomic across both renames.
- CI compiles/tests `autonomy-governor` on Windows in addition to the full Ubuntu workspace gate.

### Residual limits

- Canonicalization plus parent identity revalidation is path/race hardening, not a proof of hostile TOCTOU elimination: the implementation does not pin a directory handle across the complete mutation sequence.
- Snapshot and audit hashes are tamper-evident, not secret-key authenticity proofs.
- Phase 03 grants no production credential, network, repository merge, engine, publishing, or release authority. Later adapters must consume the governor boundary rather than bypass it.

## Non-negotiable boundaries

- Shared core crates must not depend on a specific game engine.
- Shared core crates must not depend on a specific AI provider.
- Project state that must be reproducible cannot live only in model memory.
- Destructive, credentialed, financial, production, and public-release actions require human authorization.
- Every automated write must ultimately be auditable and reversible.
- GameSpec validation does not grant agents permission to execute tools or change repositories.
- Later model, agent, tool, network, and engine adapters must obtain governor authorization rather than invent parallel permission paths.

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
