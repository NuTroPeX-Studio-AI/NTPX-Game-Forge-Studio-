# ADR-0001: Phase 01 Foundation

## Status
Accepted

## Decision
Use a Rust workspace for shared core services and a Tauri + React + TypeScript shell for the desktop command center.

## Rationale
- Rust provides a strong basis for deterministic, memory-safe core services.
- Tauri supports a lightweight cross-platform desktop shell while keeping the UI web-native.
- Engine-specific and model-specific behavior remains outside the shared core.

## Consequences
- Phase 02 can add GameSpec without coupling it to Unreal or any model provider.
- Tauri native packaging will require platform-specific dependencies and is validated separately from the frontend build.
