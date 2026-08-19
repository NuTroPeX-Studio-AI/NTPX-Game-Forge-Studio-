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

B000017 adds a VERIFIED deterministic advisory lexical dependency graph derived from B000016 evidence. Only uniquely resolved references become file-to-symbol dependency edges; ambiguous and unresolved references never become guessed edges and remain explicitly counted. Canonical edge ordering and SHA-256 correlation evidence preserve reproducibility. The exact stamped B000017 candidate head passed the complete repository gate in CI `32218018982`.

B000018 adds VERIFIED deterministic incremental repository indexing. Unchanged file evidence is reused only when path, language, generated-file state, and current source-content SHA-256 match exactly. Prior repository evidence is recomputed before reuse and fails closed on mismatch; removed files are excluded and reuse/reindex/removal counts remain explicit. Repaired implementation candidate CI `32221974551` passed the complete repository gate.

B000019 adds VERIFIED deterministic advisory diagnostics over the existing lexical reference graph. Unresolved and ambiguous lexical references emit stable warning records with canonical ordering, normalized candidate symbol IDs, explicit counts, and SHA-256 report evidence. Repaired implementation candidate CI `32226278375` passed the complete repository gate.

B000020 adds VERIFIED deterministic advisory lexical symbol search over the verified repository index. Exact, prefix, and substring symbol-name matches use stable ranking and tie-breaking; query normalization and result limiting are deterministic; SHA-256 search evidence is bound to the repository evidence digest. Repaired implementation candidate CI `32230716674` passed the complete repository gate, and exact synchronized B000020 head CI `32230940838` also completed successfully.

B000021 adds a VERIFIED implementation candidate for the provider-neutral syntax-service boundary. The new `code-intelligence-syntax` crate introduces `SyntaxService`, deterministic declaration/block structural evidence, stable line/depth records, SHA-256 syntax evidence, and fail-closed language-mismatch/unbalanced-block handling. Repaired implementation head `c21f51098c077375b832979bf6267ff24ab6b855` passed the complete repository gate in CI `32235947084`. This is a structural syntax/language-service substrate for later native parsers, not a compiler-native AST or LSP semantic model.

Remaining Phase 06 work includes native Rust/TypeScript/C++/Blueprint parser or language-service adapters with AST-grade evidence, compiler/LSP-grade semantic resolution and true call graphs, embedding/semantic search beyond the lexical query substrate, refactor planning/application, persistent cache policy, and governance-bound write execution.

## Boundaries carried forward

- Model Router selection does not invoke a model or grant execution authority.
- Agent Orchestrator scheduling does not grant tool, network, credential, repository, engine, publishing, or release authority.
- A task may request only exact governed capabilities declared in its validated task contract; undeclared capability expansion fails before Governor submission.
- `GovernanceRuntime` remains the sole verified authorization boundary for governed requests and preserves human approval for protected actions.
- Code Intelligence analysis/index/reference/dependency/incremental-index/diagnostic/search/syntax evidence does not authorize source mutation, repository writes, tool execution, model invocation, or engine actions.
- The Phase 06 lexical drivers, B000016 resolver, B000017 dependency graph, B000018 incremental reuse, B000019 diagnostics, B000020 symbol search, and B000021 structural syntax service are not represented as compiler/LSP-grade semantic analysis; B000017 is not a call graph, B000018 is not a semantic dependency invalidator or persistent cache, B000019 is not a compiler/LSP diagnostic engine, B000020 is not embedding-based semantic search, and B000021 is not a compiler-native AST or type-resolving language server.
- Incremental reuse requires verified prior repository evidence plus exact path/language/generated-state/content-digest matching; corrupted prior evidence fails closed.
- Ambiguous lexical matches remain explicit and are never resolved or promoted into dependency edges by input or enumeration order.
- Phase 05 workflow state/evidence is serializable in-memory state, not yet a crash-safe durable distributed scheduler journal.
- Phase 03 path/locking/replacement/recovery residual limits remain as documented in `ARCHITECTURE.md` and `SECURITY.md`.
