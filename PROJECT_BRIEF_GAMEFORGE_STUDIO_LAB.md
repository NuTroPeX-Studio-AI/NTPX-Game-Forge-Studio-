# NTPX GameForge Studio Lab — Project Brief

## Purpose

NTPX GameForge Studio Lab is an AI-native game development control plane. It coordinates planning, code intelligence, governed execution, game-engine adapters, creative tooling, build/test/repair loops, and release pipelines without collapsing all responsibilities into one monolithic engine.

## Human role

The human remains the top-level creative, security, and release authority. Autonomous agents may plan, generate, test, diagnose, and perform governed work, but protected actions remain policy- and approval-bound.

## Architectural position

GameForge is the developer above the developers:

- **GameSpec** expresses engine-neutral intent and project state.
- **Genesis/Director logic** plans and decomposes work.
- **Model Router** selects model candidates without granting authority.
- **Agent Orchestrator** schedules validated tasks without granting authority.
- **Code Intelligence** provides repository evidence and evidence-bound edit planning.
- **Agent Zero Execution** provides isolated governed process/tool execution.
- **Engine adapters** translate supported GameSpec/GameForge capabilities into native engine actions.
- **Tool adapters** connect Blender, audio systems, repositories, build systems, and other production software.
- **Autonomy Governor / GovernanceRuntime** remains the execution and mutation authorization boundary.
- **NTPX Fortress** remains a repository protection gate.

## Studio Lab domains

The long-term Studio Lab includes:

- Game design
- Gameplay engineering
- World building
- Character systems
- NPC intelligence
- Narrative and dialogue
- 2D/3D asset production
- Rigging and animation
- VFX
- Audio/music/voice
- Physics
- Networking and multiplayer
- XR
- Build/test/repair
- Performance profiling
- Cross-platform packaging
- Release operations
- Security, provenance, and IP protection

## Engine strategy

GameForge must not attempt to merge every engine into one runtime. Engines remain execution backends behind adapters.

Priority families include:

1. Unreal Engine
2. Unity
3. Godot
4. Web/WebGPU engines
5. Roblox Studio
6. GameMaker / GDevelop / Defold and additional engines where useful

Each adapter must advertise real capabilities. Unsupported mappings remain explicit rather than silently degraded.

## Capability states

For engine/tool translation use explicit capability states:

- REQUIRED
- SUPPORTED
- PARTIALLY_SUPPORTED
- EMULATED
- UNSUPPORTED

For runtime/tool availability distinguish:

- KNOWN
- INSTALLED
- LICENSED
- AUTHENTICATED
- CALLABLE
- GOVERNED

Registry presence alone proves only `KNOWN`.

## Hands-free creation objective

A future user interaction may look like:

> Create a playable abandoned-city third-person prototype with dynamic weather, hostile AI, vehicles, and conversational NPCs.

GameForge should be able to:

1. formalize requirements;
2. generate/update GameSpec;
3. create a dependency/task graph;
4. select compatible models/tools/engines;
5. obtain required governance decisions;
6. execute specialist agents/tools in controlled workers;
7. generate or integrate assets;
8. build the project;
9. run automated QA/playtests;
10. diagnose failures;
11. perform governed repairs;
12. rebuild and verify;
13. package a playable artifact;
14. hold protected publishing/release actions behind human approval.

## Game developer ecosystem registry

The repository includes a curated ecosystem catalog and machine-readable registry describing engines, frameworks, platform SDK ecosystems, XR stacks, asset tools, physics/audio/network middleware, IDEs, version-control systems, and specialized runtimes.

These files are planning/discovery metadata, not redistribution rights or automatic SDK access:

- `docs/reference/GAME_DEVELOPER_ECOSYSTEM.md`
- `data/registry/game-development-tools.json`

## Current implementation state

The repository roadmap is authoritative. At this handoff, Phases 01–06 are recorded VERIFIED and Phase 07 — Agent Zero Execution — is the next implementation target.

Do not replace the verified repository with an earlier conceptual scaffold.
