# NTPX GameForge Ω∞ — 18-Stage Roadmap

1. 🟦 Repository Foundation — VERIFIED
2. 🟪 GameSpec — VERIFIED
3. ⬛ Autonomy Governor — VERIFIED
4. 🧠 Model Router — NEXT
5. 🧠 Agent Orchestrator
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

## Current target

Phase 04: Model Router — introduce provider-neutral model capability descriptors, routing policies, local/cloud provider adapters, privacy/cost/latency constraints, deterministic routing evidence, fallback/abstention behavior, and tests. Model selection must not grant execution authority; later model adapters consume the verified Autonomy Governor boundary for any governed action.

## Phase 03 residual boundaries carried forward

- Path canonicalization and parent identity revalidation harden against aliases/symlinks/directory replacement but do not pin a directory handle across the complete mutation sequence; hostile TOCTOU elimination is not claimed.
- Windows replacement fallback is reversible/failure-safe but not kernel-atomic across its two renames.
- Multi-file batches are all-or-reverted for ordinary failures, not kernel-level atomic across crashes.
- Recovery remains explicit and conservative rather than automatic because retained rollback snapshots are intentionally ambiguous evidence of successful reversible or interrupted work.
- No production credentials, network, repository merge, engine execution, publishing, or release authority is granted by Phase 03.
