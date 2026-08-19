# NTPX GameForge Ω∞ — 18-Stage Roadmap

1. 🟦 Repository Foundation — VERIFIED
2. 🟪 GameSpec — VERIFIED
3. ⬛ Autonomy Governor — VERIFIED
4. 🧠 Model Router — VERIFIED
5. 🧠 Agent Orchestrator — CANDIDATE
6. 🟥 Code Intelligence — NEXT AFTER PHASE 05 VERIFICATION
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
- Phase 05 candidate establishes the deterministic Agent Orchestrator: immutable typed agent/task contracts, dependency-DAG validation, stable scheduling, per-agent concurrency bounds, Model Router dispatch integration, bounded retries/escalation, dependency blocking, workflow evidence, and exact predeclared Autonomy Governor authorization handoff.

## Current target

Phase 05: Agent Orchestrator — exact stamped-head CI and final verification evidence are the remaining acceptance gates for M05/B000014. Until those pass, Phase 06 remains gated.

## Next target after verification

Phase 06: Code Intelligence — repository parsing/indexing, symbols, references, call/dependency graphs, change-impact analysis, and refactor intelligence behind the existing orchestration and governance boundaries.

## Boundaries carried forward

- Model Router selection does not invoke a model or grant execution authority.
- Agent Orchestrator scheduling does not grant tool, network, credential, repository, engine, publishing, or release authority.
- A task may request only exact governed capabilities declared in its validated task contract; undeclared capability expansion fails before Governor submission.
- `GovernanceRuntime` remains the sole verified authorization boundary for governed requests and preserves human approval for protected actions.
- Phase 05 workflow state/evidence is serializable in-memory state, not yet a crash-safe durable distributed scheduler journal.
- Phase 03 path/locking/replacement/recovery residual limits remain as documented in `ARCHITECTURE.md` and `SECURITY.md`.
