# NTPX GameForge Ω∞

NTPX GameForge Ω∞ is an autonomous AI-native game development platform that plans, codes, tests, repairs, and evolves complete games. It unifies multi-agent AI, GameSpec, Unreal automation, asset pipelines, project memory, remote control, and governed self-improvement into one scalable development system.

## Current status

- Phase 01 — Repository Foundation: **VERIFIED**
- Phase 02 — GameSpec: next
- Current foundation build: `B000002`

## Foundation stack

- Rust workspace for core services
- Tauri + React + TypeScript desktop shell
- pnpm workspace
- GitHub Actions CI
- Offline-first architecture
- Explicit governance and security boundaries

## Quick start

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
corepack enable
pnpm install
pnpm --filter @ntpx/studio-desktop build
```

See `ARCHITECTURE.md`, `ROADMAP.md`, `SECURITY.md`, and `AGENTS.md` before making architectural changes.
