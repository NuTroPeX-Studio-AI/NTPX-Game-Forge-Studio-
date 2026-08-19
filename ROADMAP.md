# NTPX GameForge Ω∞ — 18-Stage Roadmap

1. 🟦 Repository Foundation — VERIFIED
2. 🟪 GameSpec — VERIFIED
3. ⬛ Autonomy Governor — VERIFIED
4. 🧠 Model Router — VERIFIED
5. 🧠 Agent Orchestrator — IN PROGRESS
6. 🟥 Code Intelligence — SUBSTRATE CANDIDATE
7. 0️⃣ Agent Zero Execution — SUBSTRATE CANDIDATE
8. 🟧 Unreal Engine Bridge — SUBSTRATE CANDIDATE
9. ⬜ Build / Test / Repair — SUBSTRATE CANDIDATE
10. ☤ Hermes Memory + Skills — SUBSTRATE CANDIDATE
11. 🦞 OpenClaw Remote Gateway — CONTRACT CANDIDATE
12. 🟣 AGI Project World Model — SUBSTRATE CANDIDATE
13. 🔺 ASI-Like Evaluation Council — SUBSTRATE CANDIDATE
14. 🟩 Asset Production Agents — SUBSTRATE CANDIDATE
15. 🟨 Narrative / Audio / NPC Agents — SUBSTRATE CANDIDATE
16. 🔷 Controlled Self-Improvement — SUBSTRATE CANDIDATE
17. 🎮 Signal Born Vertical Slice — CONTRACT CANDIDATE
18. 🚀 Release Pipeline — CONTRACT CANDIDATE

## Phase progression rule

A phase advances only after its acceptance criteria are implemented, compiled where applicable, tested, and recorded as verified evidence. A `SUBSTRATE CANDIDATE` or `CONTRACT CANDIDATE` is intentionally not a completed phase.

## Verified foundation

- Phase 01 established the repository, Rust workspace, desktop shell, CI, documentation, and build lineage.
- Phase 02 established the canonical engine-neutral GameSpec 0.1.0 contract, deterministic IDs, schema, validation, dependency graph/cycle detection, migration, compiler manifest, CLI, and Signal Born reference fixtures.
- Phase 03 established the verified Autonomy Governor: deny-by-default identities/permissions/scopes, protected-action human gates, request-bound HMAC approval proofs, durable replay-safe audit evidence, cross-process cooperative locking, reversible approved/direct file transactions, canonical path/runtime enforcement, all-or-reverted multi-file coordination for ordinary failures, conservative crash-recovery discovery/restoration, and tested Windows replacement fallback behavior.
- Phase 04 established the provider-neutral Model Router: typed capability catalogs, interchangeable local/cloud catalog adapters, privacy/cost/latency/reliability constraints, deterministic lexicographic ranking, ordered fallbacks, fail-closed abstention, rejection evidence, and SHA-256 routing evidence. Selection remains separate from execution authority.

## Current target

Phase 05: Agent Orchestrator — IN PROGRESS. Builds B000014–B000016 remain the verified base. The full-roadmap orchestration stream now contains implementation candidates for the three previously listed Phase 05 gaps: exact scheduler-assignment binding to Model Router selection, exact assignment binding to Autonomy Governor policy evaluation, and deterministic human/supervisor/safety escalation routing. These candidates do not invoke a model/tool or grant execution authority and are not considered verified until the exact combined head passes CI.

## Full-roadmap construction stream

`orchestration/roadmap.manifest.json` is the machine-checkable completeness inventory for all 18 phases. `roadmap-contract` validates phase continuity, component uniqueness, required safety rules, and the distinction between verified/in-progress/planned state. Later phase crates provide compile-targeted boundaries so native drivers can be added without leaking provider-, engine-, network-, signing-, or release authority into shared core modules.

Current candidate substrates cover Code Intelligence; governed Agent Zero execution envelopes; Windows/Linux Unreal command drivers; build/test/repair planning; local memory and skill metadata; a transport-neutral remote gateway contract; the project world model; independent evaluation council; asset-production jobs and provenance; narrative/audio/NPC graphs; controlled self-improvement promotion gates; the full Signal Born vertical-slice capability contract; and cross-platform release evidence/human-promotion gates.

Native/runtime work remains unfinished. This includes AST-native language services, real sandbox/process execution, real Unreal installation discovery and engine execution, build/test driver execution, durable memory persistence and retrieval, authenticated remote transport, live model/asset/audio provider adapters, runtime world-model synchronization, actual game-engine gameplay implementations, platform SDK/toolchains, real package/sign/notarize/store drivers, console devkit integrations, and real-device/platform validation.

## Boundaries carried forward

- A Model Router decision selects a candidate model; it does not authorize network access, credential use, model invocation, tool execution, repository mutation, engine execution, publishing, or release.
- Agent scheduling, retry, delegation, evidence, and escalation decisions are advisory and must not become a parallel permission path.
- A delegation directive is valid only when its delegate matches the deterministic scheduler assignment; delegation evidence cannot silently replace scheduling.
- Orchestration evidence SHA-256 digests are deterministic correlation evidence, not authorization tokens or secret-key authenticity proofs.
- Catalog capability, cost, latency, and reliability values are routing inputs and require explicit provenance/versioning before future telemetry can update them.
- Model routing evidence is deterministic SHA-256 correlation evidence, not an authorization token or secret-key authenticity proof.
- Protected actions remain human-gated regardless of which model or future agent recommends execution.
- Phase 03 path/locking/replacement/recovery residual limits remain as documented in `ARCHITECTURE.md` and `SECURITY.md`.
- Later phase substrate/contract candidates must not be represented as native/runtime completion until their concrete drivers, platform dependencies, and execution evidence exist.
