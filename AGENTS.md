# NTPX GameForge Ω∞ — Agent Rules

1. Read `ARCHITECTURE.md`, `SECURITY.md`, and `ROADMAP.md` before architectural work.
2. Never claim a phase is complete without executable evidence.
3. Keep engine-specific behavior behind adapters.
4. Prefer offline-first and privacy-preserving designs.
5. Never expose credentials in code, logs, prompts, tests, or artifacts.
6. Do not publish releases or use production credentials without explicit human approval.
7. Preserve rollback capability for every autonomous change.
8. Add or update tests with every behavior change.
9. Never weaken tests, validation, or security controls merely to make a build pass.
10. Prefer the smallest coherent implementation that satisfies acceptance criteria.
11. Record architectural decisions in `docs/adr/`.
12. Keep deterministic state separate from renderer/editor state.
13. Do not hard-code a model provider or game engine into shared core modules.
14. Treat versioned GameSpec as the canonical machine-readable source of truth for reproducible game intent and project state.
15. GameSpec validity never grants tool, filesystem, network, credential, merge, or release authority.
16. Report status as VERIFIED, FAILED, INFERRED, BLOCKED, or UNKNOWN.
17. Signal Born: First Thought is the primary reference implementation.
18. Model, agent, tool, network, repository, engine, and release adapters must consume the verified Autonomy Governor boundary for governed actions; they must not create parallel permission paths.
19. Protected actions remain human-gated even when a later model or agent recommends execution.
20. Do not represent advisory locking, reversible Windows replacement, all-or-reverted batches, or canonical path revalidation as stronger guarantees than documented in `ARCHITECTURE.md` and `SECURITY.md`.
21. Treat Model Router outputs as selection evidence only. A selected provider/model never grants network, credential, tool, repository, engine, publishing, or release authority.
22. Preserve deterministic routing: hard constraints must not be silently relaxed, adapter enumeration order must not change equivalent decisions, and stable identity must remain the final tie-breaker unless an ADR explicitly changes the contract.
23. Keep prompt/project payload content out of routing requests unless a future reviewed architecture explicitly requires content-aware routing and documents its privacy/provenance implications.
24. Treat provider capability, cost, latency, and reliability declarations as versioned input metadata, not independently verified truth or permission evidence.
25. Treat validated Agent Orchestrator task contracts as immutable work authority inputs. An agent may not add dependencies, increase retry limits, change its assigned principal, or expand governed scopes during execution without producing a newly validated contract.
26. Scheduling is not authorization. A ready or dispatched task still requires the Autonomy Governor for every governed capability.
27. A task may submit only exact governed requests predeclared in its validated contract; never synthesize a broader permission request as a retry or fallback.
28. Retry exhaustion must escalate rather than silently increasing `max_attempts`; dependency failures must remain visible and block downstream work until explicitly replanned.
29. Model Router abstention must remain an explicit orchestration escalation; never bypass hard model constraints to keep a workflow moving.
30. Phase 05 workflow evidence is not a substitute for the durable Autonomy Governor audit log. Governed authorization evidence remains Governor-owned.
31. Do not claim Phase 05 provides distributed workers, crash-safe scheduler journaling, leases, heartbeats, or external execution; those capabilities belong to later phases unless separately implemented and verified.
