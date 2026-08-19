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

B000022 adds the first VERIFIED language-specific parser adapter foundation. `RustParserAdapter` emits deterministic top-level Rust item evidence for functions, structs, enums, traits, modules, impls, type aliases, constants, and statics, with stable source line/ordinal metadata and SHA-256 evidence. It handles common visibility prefixes, keeps nested methods out of top-level item evidence, validates extracted identifiers, and fails closed on malformed recognized items, language mismatch, empty paths, and unbalanced blocks. Repaired implementation head `8242add6cf64bc51a4804e631ebdb5669ece37c1` passed the complete repository gate in CI `32240851940`. This remains a parser-adapter foundation, not a `rustc` AST, macro/type resolver, control-flow graph, true call graph, or LSP semantic model.

B000023 upgrades the Rust parser foundation to AST-grade full-file Rust syntax parsing backed by `syn`. Supported top-level item evidence is derived from the parsed syntax tree rather than line-prefix and brace-counting heuristics; parser spans provide source-line evidence, comments/string braces no longer corrupt structural interpretation, multiline/generic/attributed Rust items parse naturally, nested methods remain outside top-level evidence, and malformed Rust fails closed with parser evidence. Repaired implementation head `748c3045e76f681e752d7d5e152a7ef5e4364ada` passed the complete repository gate in CI `32245983720`. B000023 remains a syntax parser rather than `rustc` compilation or semantic/type resolution.

B000024 adds the first VERIFIED TypeScript-specific parser-adapter foundation. `TypeScriptParserAdapter` emits deterministic top-level evidence for supported functions, classes, interfaces, type aliases, enums, namespaces, and `const`/`let`/`var` declarations with stable line/ordinal metadata and SHA-256 evidence. Its lexical structure scanner shields comments, quoted strings, and multiline template strings from brace-depth accounting so nested declarations are not promoted. Empty paths, language mismatch, malformed recognized declarations, and unbalanced lexical structure fail closed. Repaired implementation head `aabae131b2a06a9f5a365c91e3974043b8406c22` passed CI `32250932142`. This remains a TypeScript parser-adapter foundation rather than a compiler AST or language service.

B000025 upgrades TypeScript analysis to VERIFIED AST-grade full-file syntax parsing using the Tree-sitter TypeScript grammar behind the existing provider-neutral adapter. Supported top-level declarations and variable declarators are derived from syntax-tree nodes with parser-derived line evidence and deterministic SHA-256 correlation output. Multiline/generic declarations, regular-expression literals, template expressions, and multiple declarators are grammar-parsed rather than handled by ad-hoc brace shielding; malformed syntax fails closed. After formatter and cursor-lifetime repairs, implementation head `807778bf0fcf9c756f7cf0c58c5eac5f571c152a` passed complete CI `32252319138`. B000025 remains syntax analysis rather than TypeScript compiler binding, module resolution, type checking, LSP semantics, or refactor execution.

B000026 completes the language-specific parser-adapter coverage required by the revised safe-edit threshold. A Tree-sitter-backed C++ adapter emits deterministic top-level function/class/struct/enum/namespace/alias evidence and supports template-wrapped top-level declarations without promoting class methods. A textual Blueprint metadata adapter emits deterministic class/function/variable plus parent-class/asset reference evidence and fails closed on malformed or unknown records. Repaired implementation head `a1af5cbd71db318f98f40f5d0bfb49b763f2ddd8` passed complete CI `32253543411`. The C++ adapter is not Clang semantic analysis, and Blueprint binary `.uasset` inspection remains a Phase 08 Unreal adapter responsibility.

### Revised Phase 06 completion threshold

To validate the first real GameForge vertical loop sooner, Phase 06 completion is narrowed to the minimum safe-edit substrate approved for the M06→M09 integration path: deterministic language-specific parsing, exact evidence-bound edit planning, persistent/cache evidence sufficient to reject stale plans, and Autonomy-Governor-bound reversible source mutation. Advanced compiler/type semantics, true semantic call graphs, and embedding-based semantic search are deferred until a concrete M07–M09 workflow proves they are required; they remain planned Code Intelligence expansions and are not represented as completed functionality.

Remaining Phase 06 work is therefore deterministic exact-edit planning/application evidence, stale-plan/persistent evidence policy, Autonomy-Governor-bound reversible source mutation, and full Phase 06 acceptance verification.

## Boundaries carried forward

- Model Router selection does not invoke a model or grant execution authority.
- Agent Orchestrator scheduling does not grant tool, network, credential, repository, engine, publishing, or release authority.
- A task may request only exact governed capabilities declared in its validated task contract; undeclared capability expansion fails before Governor submission.
- `GovernanceRuntime` remains the sole verified authorization boundary for governed requests and preserves human approval for protected actions.
- Code Intelligence analysis/index/reference/dependency/incremental-index/diagnostic/search/syntax/parser evidence does not authorize source mutation, repository writes, tool execution, model invocation, or engine actions.
- The Phase 06 lexical drivers, B000016 resolver, B000017 dependency graph, B000018 incremental reuse, B000019 diagnostics, B000020 symbol search, B000021 structural syntax service, B000022 Rust parser adapter foundation, B000023 `syn`-backed Rust syntax parser, B000024 TypeScript parser-adapter foundation, B000025 Tree-sitter TypeScript syntax parser, and B000026 C++/Blueprint parser adapters are not represented as compiler/LSP semantic analysis; B000017 is not a call graph, B000018 is not a semantic dependency invalidator or persistent cache, B000019 is not a compiler/LSP diagnostic engine, B000020 is not embedding-based semantic search, B000023 does not perform macro expansion/cfg/module/type resolution, B000025 does not perform TypeScript binding/type checking/LSP services, and B000026 does not perform Clang semantics or Unreal binary asset parsing.
- Incremental reuse requires verified prior repository evidence plus exact path/language/generated-state/content-digest matching; corrupted prior evidence fails closed.
- Ambiguous lexical matches remain explicit and are never resolved or promoted into dependency edges by input or enumeration order.
- Phase 05 workflow state/evidence is serializable in-memory state, not yet a crash-safe durable distributed scheduler journal.
- Phase 03 path/locking/replacement/recovery residual limits remain as documented in `ARCHITECTURE.md` and `SECURITY.md`.
