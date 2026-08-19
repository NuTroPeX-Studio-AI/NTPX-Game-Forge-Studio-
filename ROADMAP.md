# NTPX GameForge Ω∞ — 18-Stage Roadmap

1. 🟦 Repository Foundation — VERIFIED
2. 🟪 GameSpec — VERIFIED
3. ⬛ Autonomy Governor — VERIFIED
4. 🧠 Model Router — VERIFIED
5. 🧠 Agent Orchestrator — VERIFIED
6. 🟥 Code Intelligence — VERIFIED
7. 0️⃣ Agent Zero Execution — NEXT
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

A phase advances only after its acceptance criteria are implemented, compiled where applicable, tested, and recorded as verified evidence. Phase 06/B000028 passed its candidate, VERIFIED-evidence, and final synchronization CI/Fortress gates before PR #17 was squash-merged to `main`.

## Verified foundation

- Phase 01: repository/workspace/desktop/CI foundation.
- Phase 02: canonical engine-neutral GameSpec 0.1.0, deterministic IDs/schema/validation/dependency graph/migration/compiler/CLI/reference fixtures.
- Phase 03: deny-by-default Autonomy Governor with exact scopes, human approval proofs, durable audit, runtime enforcement, reversible transactions, rollback/recovery, and Windows regression coverage.
- Phase 04: provider-neutral deterministic Model Router with hard constraints, ordered fallbacks/abstention, and routing evidence; selection is never execution authority.
- Phase 05: deterministic Agent Orchestrator with immutable agent/task contracts, dependency scheduling, concurrency bounds, retry/escalation, evidence, and exact Governor authorization handoff.
- Phase 06: deterministic multi-language repository/syntax/reference/dependency/search/diagnostic evidence, evidence-bound exact whole-file edit planning, stale-source/repository rejection, and Autonomy-Governor-bound reversible exact edit application with preserved human approval.

## Phase 06 lineage

- **B000015** — deterministic provider-neutral repository index foundation with stable source/symbol/repository SHA-256 evidence.
- **B000016** — deterministic lexical reference resolution with explicit resolved/ambiguous/unresolved states.
- **B000017** — deterministic advisory lexical dependency graph; only uniquely resolved references become edges.
- **B000018** — fail-closed incremental index reuse bound to verified prior repository evidence and exact file metadata/content digests.
- **B000019** — deterministic advisory diagnostics for unresolved/ambiguous reference evidence.
- **B000020** — deterministic advisory exact/prefix/substring symbol search.
- **B000021** — provider-neutral syntax-service boundary.
- **B000022/B000023** — Rust parser foundation upgraded to `syn`-backed full-file AST syntax parsing.
- **B000024/B000025** — TypeScript parser foundation upgraded to Tree-sitter full-file AST syntax parsing.
- **B000026** — Tree-sitter C++ top-level syntax parsing plus deterministic textual Blueprint metadata parsing.
- **B000027** — deterministic exact whole-file edit planning bound to repository/source/replacement evidence, with generated/no-op/stale-plan rejection.
- **B000028** — `code-intelligence-apply` rechecks plan/replacement integrity, canonical repository-relative targeting, and current on-disk source freshness, then routes mutation exclusively through the verified Autonomy Governor rollback-capable file transaction path. Direct policy and human-approval paths remain distinct and fail closed.

## B000028 verification evidence

Initial closure CI `32260168938` exposed rustfmt-only drift in the new governed application crate. Exact formatter-equivalent changes were applied without weakening behavior or tests.

Repaired implementation head `d63050c8184521e6408b6bb9923af4da327f0ebf` passed complete CI `32260654093` and Fortress `32260654100`.

The exact stamped B000028 candidate passed complete CI `32261483993` and NTPX Fortress `32261483663`.

The final VERIFIED evidence head passed complete CI `32262188186` and Fortress `32262188321`.

The exact final synchronization head `301980124823397d72592fa3f33186898bfb0844` passed complete CI `32262420120` and Fortress `32262420198`, then PR #17 squash-merged it to `main` as signed commit `094da1cc35efc596568fdc77bb7f3551093e8079`.

## Revised Phase 06 completion threshold

Phase 06 is completed against the minimum safe-edit substrate needed for the M06→M09 vertical loop:

- deterministic multi-language repository/syntax evidence;
- deterministic reference/dependency/impact/search/diagnostic evidence;
- exact evidence-bound edit planning;
- stale repository/source rejection;
- plan/replacement integrity verification before application;
- canonical repository-relative targeting;
- Autonomy-Governor-bound reversible source mutation with preserved human approval.

The following remain deferred and are **not** represented as completed Phase 06 functionality unless a later M07–M09 workflow proves them necessary:

- `rustc`, Clang, or TypeScript-compiler type semantics;
- true semantic call graphs or an LSP semantic model;
- embedding/vector semantic search;
- broad persistent semantic caches beyond the verified incremental evidence substrate;
- Blueprint binary `.uasset` parsing/mutation (Phase 08 Unreal bridge responsibility).

## Boundaries carried forward

- Model Router selection does not invoke a model or grant execution authority.
- Agent Orchestrator scheduling does not grant tool, network, credential, repository, engine, publishing, signing, or release authority.
- `GovernanceRuntime` remains the sole verified authorization boundary for governed source mutation.
- A Code Intelligence plan is evidence, never permission.
- B000028 whole-file application cannot override Governor denial or a human approval gate.
- Phase 03 cooperative locking, path-race, Windows replacement, rollback, and multi-file atomicity limitations remain authoritative; Phase 06 does not claim hostile compare-and-swap/TOCTOU elimination.
- NTPX Fortress bootstrap protection on `main` is a required carried-forward gate.

## Current target

Phase 07: Agent Zero Execution — introduce isolated execution workers, explicit process/tool contracts, lifecycle/cancellation/resource limits, evidence capture, and governed tool/process execution without bypassing the verified Autonomy Governor, Agent Orchestrator, Code Intelligence, or Fortress boundaries.
