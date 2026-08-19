# Architecture

## Current boundary

Phases 01–06 are verified foundations. Phase 07 Agent Zero Execution is the next construction target. GameSpec remains the canonical engine-neutral project contract; the Model Router selects candidates; the Agent Orchestrator schedules work; Code Intelligence supplies deterministic repository/syntax/reference/dependency/search/diagnostic/edit-plan evidence; and the Autonomy Governor remains the sole verified authorization and reversible-mutation boundary. A plan or analysis result is never permission.

NTPX Fortress bootstrap protection on `main` is an additional repository gate and is carried forward by Phase 06.

## Layers

1. **Studio UI** — Tauri + React desktop shell.
2. **Core services** — Rust workspace crates with explicit provider/engine-neutral interfaces.
3. **Specifications** — GameSpec schema, validation, dependency graph, migrations, compiler metadata, and CLI.
4. **Governance** — deny-by-default identities/permissions/scopes, approval proofs, durable audit, canonical runtime enforcement, reversible transactions, rollback, and recovery.
5. **Intelligence** — verified Model Router, Agent Orchestrator, and Code Intelligence; later Project World Model and Evaluation Council.
6. **Execution** — Agent Zero, engine/tool adapters, and build/test/repair.
7. **Production** — asset/narrative/audio/NPC production, controlled improvement, game build, and release.

## Authority chain

```text
GameSpec / validated project state
        ↓
Code Intelligence evidence + edit plan
        ↓
Agent Orchestrator scheduling context
        ↓
Model Router selection evidence (when needed)
        ↓
Autonomy Governor authorization
        ↓
Governed reversible mutation / later execution adapter
```

Selection, analysis, scheduling, planning, authorization, and execution are distinct operations. No upstream layer may infer a downstream permission.

## GameSpec boundary

- `GameSpec` is engine-neutral and AI-provider-neutral.
- Stable IDs/references and deterministic generated IDs make project intent reproducible.
- Dependency-bearing references validate and cycles fail closed.
- Asset records carry licensing/provenance metadata.
- Engine adapters consume validated/compiled GameSpec state rather than invent untracked canonical state.

## Autonomy Governor boundary

- Policy evaluation is deny-by-default with exact principal/permission/resource scopes.
- Protected credential, destructive, merge, publish, signing, and release capabilities preserve explicit human approval.
- Request-bound HMAC-SHA256 approval proofs expire and use replay-safe one-time nonces.
- Authorization evidence is durable append-only hash-chained JSONL with cooperative cross-process writer exclusion.
- `GovernanceRuntime` is the in-process enforcement boundary; denied operations do not execute.
- Governed filesystem paths are canonicalized/hardened, symlink ancestors/targets are rejected, and parent identity is revalidated immediately before mutation.
- Governed file transactions use durable pre-write snapshots, same-directory staging, platform-aware replacement, durable authorization audit, explicit rollback receipts, and automatic restoration for ordinary post-replacement audit/replay failures.
- Multi-file batches are all-or-reverted for ordinary failures, not kernel-atomic across multiple paths.
- Recovery is conservative and explicit.

### Governor residual limits

- Advisory locks coordinate cooperating processes only.
- Canonical path/parent-identity revalidation reduces path races but does not pin a directory handle for the entire mutation and does not mathematically eliminate hostile TOCTOU.
- Audit/snapshot hashes are tamper-evident, not authenticated against an actor able to rewrite all relevant files and recompute hashes.
- Windows fallback replacement is reversible/failure-safe but not kernel-atomic across both renames.

## Model Router boundary

- Provider adapters expose validated catalog metadata only; Phase 04 added no live provider invocation API.
- Hard capability/privacy/cost/latency/reliability constraints fail closed.
- Equivalent inputs produce deterministic primary/fallback or abstention evidence.
- Model selection never grants network, credential, tool, filesystem, repository, engine, publishing, signing, or release authority.

## Agent Orchestrator boundary

- Immutable agent/task contracts validate identity, skills, dependencies, routing requests/policies, retry budgets, and exact predeclared governed requests.
- Task DAG scheduling and ready ordering are deterministic.
- Per-agent concurrency and bounded retry/escalation are explicit.
- Routing abstention escalates without silently relaxing constraints.
- A task can submit only an exact governed request already declared in its contract and only while running.
- Governor denials and human approval requirements remain authoritative.
- Phase 05 implements no live model inference, remote executor, distributed queue, or crash-safe scheduler journal.

## Phase 06 Code Intelligence boundary

### Repository/index evidence

- `code-intelligence` provides deterministic source indexing with stable source SHA-256 evidence, stable symbol identifiers, lexical references, generated-file metadata, repository correlation evidence, and advisory change-impact output.
- Duplicate source paths, driver mismatch, missing drivers, and corrupted prior incremental evidence fail closed.
- Incremental reuse requires exact path/language/generated-state/content-digest matching against verified prior repository evidence.

### Reference/dependency/search/diagnostic evidence

- Exact lexical reference resolution preserves explicit `Resolved`, `Ambiguous`, and `Unresolved` outcomes; ambiguity never chooses an arbitrary winner.
- Advisory dependency edges are created only from uniquely resolved reference evidence.
- Diagnostics and symbol search are deterministic and evidence-bound.
- These lexical products are not represented as compiler/LSP semantic analysis or true semantic call graphs.

### Language-specific syntax

- Rust syntax parsing uses `syn` for full-file AST syntax evidence.
- TypeScript and C++ syntax parsing use Tree-sitter behind provider-neutral adapters.
- Textual Blueprint metadata parsing is deterministic and fail closed for malformed/unknown records.
- Phase 06 does not claim `rustc`, Clang, or TypeScript-compiler type semantics, module/type inference, control-flow semantics, or Blueprint binary `.uasset` understanding.

### Exact edit planning

- `code-intelligence-edits` remains read-only.
- An `ExactEditPlan` binds repository evidence, source path/language/current content digest, replacement content digest, and deterministic plan evidence.
- Generated-source, no-op, stale repository, and stale source plans fail closed.

### Governed exact edit application

- `code-intelligence-apply` is the only Phase 06 bridge from a fresh plan to source mutation.
- Before Governor entry it recomputes replacement-content and full-plan evidence, requires an existing canonical repository root, rejects non-normal repository-relative paths, resolves the target through the Governor filesystem boundary, re-reads the on-disk source, and reruns B000027 freshness validation.
- Direct mutation calls `GovernanceRuntime::write_file`; human-gated mutation calls `write_file_with_approval`.
- A Code Intelligence plan cannot override policy denial or manufacture/satisfy a human approval.
- Successful application returns the existing Autonomy Governor `TransactionReceipt`; Phase 06 creates no parallel transaction, audit, rollback, or permission system.
- The freshness recheck immediately precedes Governor entry, but Phase 06 does not claim hostile compare-and-swap semantics against a non-cooperating process racing the file between those steps. Phase 03 path/race/locking limits remain authoritative.

### Deferred Code Intelligence expansions

The revised M06→M09 completion threshold intentionally defers until a concrete later workflow proves a need:

- compiler/type-system semantic resolution;
- true semantic call graphs;
- LSP semantic models;
- embedding/vector semantic search;
- broader persistent semantic caches;
- Blueprint binary `.uasset` parsing/mutation, which belongs behind the Unreal bridge.

## NTPX Fortress boundary

- Fortress bootstrap monitoring is a required CI gate for repository changes.
- Phase 06 does not commit production secrets, private signing keys, or higher-classification crown-jewel material to satisfy tests.
- Fortress does not replace the Autonomy Governor; it is a repository protection/gating layer, while the Governor remains the runtime authorization boundary.

## Non-negotiable boundaries

- Shared core crates remain engine-neutral and AI-provider-neutral.
- Reproducible project state cannot live only in model memory.
- Destructive, credentialed, financial, production, signing, and public-release actions require human authorization.
- Automated writes must ultimately be auditable and reversible.
- GameSpec validity is not permission.
- Model selection is not execution authority.
- Orchestration/scheduling is not authorization authority.
- Code analysis/edit planning is not mutation authority.
- Future model, agent, tool, network, repository, engine, and release adapters must consume the Governor rather than create parallel permission paths.

## Repository layout

```text
apps/                  User-facing applications
crates/                Core services, GameSpec, governance, routing, orchestration, Code Intelligence
schemas/               Versioned GameSpec JSON Schema contracts
examples/              Reference and migration fixtures
docs/                  ADRs and verification evidence
.github/workflows/      CI and Fortress gates
.gameforge/             Build/version metadata and history
```
