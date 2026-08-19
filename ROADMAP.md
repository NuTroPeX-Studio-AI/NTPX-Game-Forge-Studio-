# NTPX GameForge Ω∞ — 18-Stage Roadmap

1. 🟦 Repository Foundation — VERIFIED
2. 🟪 GameSpec — VERIFIED
3. ⬛ Autonomy Governor — VERIFIED
4. 🧠 Model Router — VERIFIED
5. 🧠 Agent Orchestrator — VERIFIED
6. 🟥 Code Intelligence — IN PROGRESS
7. 0️⃣ Agent Zero Execution
8. 🟧 Unreal Engine Bridge
9. ⬜ Build / Test / Repair
10. ☤ Hermes Memory + Skills
11. 🦞 OpenClaw Remote Gateway
12. 🟣 AGI Project World Model
13. 🔺 ASI-Like Evaluation Council
14. 🟩 Asset Production Agents
15. 🟨 Narrative / Audio / NPC Agents
16. 🔷 Controlled Self-Improvement
17. 🎮 Signal Born Vertical Slice
18. 🚀 Release Pipeline

## Phase progression rule

A phase advances only after its acceptance criteria are implemented, compiled where applicable, tested, and recorded as verified evidence.

## Verified foundation

- Phase 01 established the repository, Rust workspace, desktop shell, CI, documentation, and build lineage.
- Phase 02 established the canonical engine-neutral GameSpec 0.1.0 contract, deterministic IDs, schema, validation, dependency graph/cycle detection, migration, compiler manifest, CLI, and Signal Born reference fixtures.
- Phase 03 established the verified Autonomy Governor: deny-by-default identities/permissions/scopes, protected-action human gates, request-bound HMAC approval proofs, durable replay-safe audit evidence, cross-process cooperative locking, reversible approved/direct file transactions, canonical path/runtime enforcement, all-or-reverted multi-file coordination for ordinary failures, conservative crash-recovery discovery/restoration, and tested Windows replacement fallback behavior.
- Phase 04 established the provider-neutral Model Router: typed capability catalogs, interchangeable local/cloud catalog adapters, privacy/cost/latency/reliability constraints, deterministic lexicographic ranking, ordered fallbacks, fail-closed abstention, rejection evidence, and SHA-256 routing evidence. Selection remains separate from execution authority.
- Phase 05 established the deterministic Agent Orchestrator: immutable typed agent/task contracts, dependency-DAG validation, stable scheduling, per-agent concurrency bounds, Model Router dispatch integration, bounded retries/escalation, dependency blocking, workflow evidence, and exact predeclared Autonomy Governor authorization handoff.

## Current target

Phase 06: Code Intelligence — IN PROGRESS. B000015 established the provider-neutral `code-intelligence` workspace crate, explicit Rust/TypeScript/C++/Blueprint-metadata lexical language-driver boundaries, deterministic source indexing, stable symbol IDs, lexical references, generated-file metadata, SHA-256 repository evidence, fail-closed input validation, and advisory change-impact output.

B000016 adds deterministic repository-level lexical reference resolution. Unique exact target/name matches resolve to stable symbol evidence; ambiguous matches remain explicit with deterministic candidate ordering and no arbitrary winner; missing matches remain explicit `Unresolved` evidence. The reference graph carries deterministic SHA-256 correlation evidence and remains advisory/read-only.

B000017 adds a deterministic advisory lexical dependency graph derived from B000016 evidence. Only uniquely resolved references become file-to-symbol dependency edges; ambiguous and unresolved references never become guessed edges and remain explicitly counted. Canonical edge ordering and SHA-256 correlation evidence preserve reproducibility. B000017 remains subject to exact stamped-head CI before final VERIFIED classification.

Remaining Phase 06 work includes AST-native parsing/language services, compiler/LSP-grade semantic resolution and call graphs, incremental caching, diagnostics, semantic search, refactor planning/application, and governance-bound write execution.

## Boundaries carried forward

- Model Router selection does not invoke a model or grant execution authority.
- Agent Orchestrator scheduling does not grant tool, network, credential, repository, engine, publishing, or release authority.
- A task may request only exact governed capabilities declared in its validated task contract; undeclared capability expansion fails before Governor submission.
- `GovernanceRuntime` remains the sole verified authorization boundary for governed requests and preserves human approval for protected actions.
- Code Intelligence analysis/index/reference/dependency-graph evidence does not authorize source mutation, repository writes, tool execution, model invocation, or engine actions.
- The Phase 06 lexical drivers, B000016 resolver, and B000017 dependency graph are not represented as AST-complete or compiler/LSP-grade semantic resolution; B000017 is not a call graph.
- Ambiguous lexical matches remain explicit and are never resolved or promoted into dependency edges by input or enumeration order.
- Phase 05 workflow state/evidence is serializable in-memory state, not yet a crash-safe durable distributed scheduler journal.
- Phase 03 path/locking/replacement/recovery residual limits remain as documented in `ARCHITECTURE.md` and `SECURITY.md`.
