# NTPX GameForge Ω∞ — 18-Stage Roadmap

1. 🟦 Repository Foundation — VERIFIED
2. 🟪 GameSpec — VERIFIED
3. ⬛ Autonomy Governor — VERIFIED
4. 🧠 Model Router — VERIFIED
5. 🧠 Agent Orchestrator — IN PROGRESS
6. 🟥 Code Intelligence
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

## Current target

Phase 05: Agent Orchestrator — IN PROGRESS. Build B000014 verified the typed agent/task/state contracts, fail-closed dependency graph, deterministic topological ordering, and capacity/capability-aware advisory scheduler; its exact stamped head passed CI run `32200062490`. Build B000015 adds bounded retry/state-transition semantics with retained attempt counts, exact retry-budget exhaustion, fail-closed malformed attempt state, retry-aware scheduling, and explicit non-empty escalation state. Remaining Phase 05 work includes explicit delegation, deterministic orchestration evidence collection, Model Router integration for selection, Autonomy Governor integration for governed actions, and broader escalation routing/policy layered on the explicit escalation state.

## Boundaries carried forward

- A Model Router decision selects a candidate model; it does not authorize network access, credential use, model invocation, tool execution, repository mutation, engine execution, publishing, or release.
- Agent scheduling, retry, delegation, and escalation decisions are advisory and must not become a parallel permission path.
- Catalog capability, cost, latency, and reliability values are routing inputs and require explicit provenance/versioning before future telemetry can update them.
- Model routing evidence is deterministic SHA-256 correlation evidence, not an authorization token or secret-key authenticity proof.
- Protected actions remain human-gated regardless of which model or future agent recommends them.
- Phase 03 path/locking/replacement/recovery residual limits remain as documented in `ARCHITECTURE.md` and `SECURITY.md`.
