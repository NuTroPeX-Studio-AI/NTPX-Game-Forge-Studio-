# NTPX GameForge Studio Lab

AI-native universal game-development studio and orchestration layer.

## Mission

GameForge sits above game engines, creative tools, repositories, AI agents, QA systems, and release targets. The system should let a human director describe a game in natural language and have governed AI agents plan, create, test, repair, build, and package a playable project.

## Current build target

Generation 0 → Generation 1:

1. Establish the monorepo and development environment.
2. Build the Command Center shell.
3. Implement Genesis Director interfaces.
4. Define GFIR (GameForge Intermediate Representation).
5. Build the agent runtime and task graph.
6. Create the Engine Adapter SDK.
7. Implement Godot and Web/Babylon.js reference adapters.
8. Add autonomous validation and QA.
9. Produce the first playable generated prototype.

## Non-goals for the first build

- Do not integrate every game engine at once.
- Do not claim unsupported engine translation.
- Do not allow unreviewed AI agents to mutate production/release state.
- Do not make cloud dependencies mandatory when a local-first option is practical.

See `AGENTS.md`, `ARCHITECTURE.md`, `ROADMAP.md`, and `CODEX_START.md`.
