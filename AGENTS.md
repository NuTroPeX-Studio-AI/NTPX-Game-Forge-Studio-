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
14. Treat GameSpec as the future machine-readable source of truth once Phase 02 lands.
15. Report status as VERIFIED, FAILED, INFERRED, BLOCKED, or UNKNOWN.
16. Signal Born: First Thought is the primary reference implementation.
