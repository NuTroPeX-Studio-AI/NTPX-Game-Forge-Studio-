# NTPX GameForge — Codex Instructions

## Role

Act as the implementation team for NTPX GameForge Studio Lab. Prefer working software, tests, and reproducible builds over speculative architecture.

## Source of truth

Read these before major work:
- `README.md`
- `ARCHITECTURE.md`
- `ROADMAP.md`
- `CODEX_START.md`
- relevant files under `docs/`

Keep this file concise. Put detailed design material under `docs/`.

## Core constraints

1. Preserve the project name: **NTPX GameForge Studio Lab**.
2. Keep Genesis Director as the top-level orchestration layer.
3. Keep GFIR engine-independent.
4. Use adapters rather than embedding or copying third-party engines.
5. First reference engines are **Godot** and **Web/Babylon.js/WebGPU**.
6. Unreal, Unity, Roblox, GameMaker, GDevelop, and others are later adapters.
7. Prefer local-first, privacy-conscious architecture.
8. External cloud services must be optional adapters where practical.
9. Never silently downgrade unsupported engine capabilities.
10. Capability negotiation must distinguish:
   - REQUIRED
   - SUPPORTED
   - PARTIALLY_SUPPORTED
   - EMULATED
   - UNSUPPORTED

## Engineering priorities

Order of importance:
1. Correctness
2. Testability
3. Security
4. Determinism/reproducibility
5. Maintainability
6. Performance
7. Developer ergonomics

## Agent governance

AI agents may plan, propose, generate, test, and repair within governed workspaces.
Production/release mutations must pass policy and validation gates.

Use:
- isolated worktrees/branches
- explicit task ownership
- structured diffs
- automated tests
- reviewer agent
- audit logs
- rollback paths

## Repository behavior

- Do not rewrite unrelated files.
- Keep commits/task patches narrowly scoped.
- Run relevant tests after edits.
- Add tests with new behavior.
- Surface blockers instead of faking completion.
- Record architectural decisions under `docs/architecture/`.
- Track executable plans under `docs/exec-plans/active/`.

## Initial technology direction

- UI: React + TypeScript
- Desktop: Tauri
- Mobile: Flutter (later phase)
- Core runtime: Rust
- Agent services: Python where advantageous
- Local data: SQLite
- Server data: PostgreSQL when required
- Messaging: NATS when a broker is justified
- Containers: Docker
- Observability: OpenTelemetry
- Game engine reference adapters: Godot + Babylon.js/WebGPU
- Asset pipeline reference: Blender
- SCM: Git; GitHub optional remote

Do not add infrastructure merely because it appears in this list. Introduce dependencies only when the current milestone needs them.

## Definition of done

A task is done only when:
- code/config/docs are implemented,
- relevant tests pass,
- failure modes are handled,
- commands are documented,
- no known critical regression is hidden,
- implementation matches the active milestone.

## Immediate objective

Execute `CODEX_START.md` and establish G000 → G001 without jumping ahead into dozens of engine integrations.
