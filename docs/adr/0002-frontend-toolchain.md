# ADR-0002: Frontend Toolchain Baseline

## Status
Accepted

## Decision
Pin the Phase 01 frontend baseline to React 19.2.8, Vite 8.2.0, `@vitejs/plugin-react` 6.0.5, and TypeScript 7.0.2. Keep Tauri on the compatible `2.x` line.

## Rationale
The versions were checked against current package releases during Phase 01. Vite 8 requires Node.js 20.19+ or 22.12+; CI uses the current Node 22 line.

## Consequences
- Dependency updates after Phase 01 must be explicit and validated.
- A lockfile should be generated and committed as soon as a trusted package-install environment is available.
