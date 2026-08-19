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
32. Treat Code Intelligence indexes, references, dependency graphs, diagnostics, search results, syntax evidence, and edit plans as evidence only; none is an authorization token.
33. Ambiguous or unresolved lexical references must stay explicit. Never choose a mutation target merely to keep a workflow moving.
34. Do not describe Phase 06 lexical dependency evidence as a true semantic call graph or compiler/LSP semantic model.
35. Preserve language-specific syntax boundaries: Rust `syn`, TypeScript/C++ Tree-sitter, and textual Blueprint metadata. Do not claim compiler/type semantics or Blueprint binary `.uasset` understanding without separate implementation and verification.
36. Exact edit plans must remain bound to repository evidence, source digest, replacement digest, and plan evidence; generated/no-op/stale plans fail closed.
37. Before applying a Phase 06 edit plan, recheck replacement/plan integrity, enforce a normal repository-relative path, re-read current on-disk source, and rerun freshness validation.
38. Phase 06 source mutation must enter through `GovernanceRuntime` and the existing rollback-capable transaction path. Never add a Code Intelligence-specific bypass, raw write, permission grant, or parallel audit system.
39. A Code Intelligence plan cannot override denial or human approval. Human-gated mutation must use the exact existing request-bound approval path.
40. Do not claim B000028 eliminates hostile TOCTOU/content races; the Phase 03 cooperative-locking and path-race residual limits remain authoritative.
41. NTPX Fortress bootstrap monitoring is a carried-forward repository gate. Do not weaken, bypass, or seed secrets/high-classification material to make that gate pass.
42. Treat a validated Agent Zero execution contract as evidence only. It is never an authorization permit and cannot be used to infer permission to create a process.
43. Agent Zero execution must bind to the exact predeclared Governor principal, `Permission::Execute`, `ScopeKind::Tool`, and tool resource; any mismatch fails closed.
44. Request-provided environment variables must be explicitly allowlisted. Never inherit or expose ambient secrets merely because a process contract exists.
45. Runtime workers must obtain Governor authorization before process creation and must enforce declared time, output, argument, cancellation, and isolation limits rather than treating declarations as best-effort hints.
46. Do not claim B000029 provides process spawning, OS/container isolation, runtime resource enforcement, network isolation, credential isolation, or engine execution; those require separate implementation and executable verification.
