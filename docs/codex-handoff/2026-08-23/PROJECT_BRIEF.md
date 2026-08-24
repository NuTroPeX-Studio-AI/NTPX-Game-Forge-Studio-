# Project Brief — NTPX GameForge Studio Lab

## Purpose

Build an AI-native universal game studio that can coordinate multiple game engines and development tools through a common orchestration layer.

## Human role

The human remains the top-level creative and approval authority.

## Desired interaction

Text, voice, touch, keyboard/mouse, controller, and eventually XR.

Example:
> Create a playable abandoned-city prototype with third-person movement, dynamic weather, hostile AI, vehicles, and conversational NPCs.

The system should plan the work, select an appropriate engine, generate a project, execute governed agents, build, test, diagnose failures, repair, and present a playable artifact.

## Architectural philosophy

GameForge is not another monolithic game engine.
It is the **developer above the developers**:
- engines are execution backends
- tools are adapters
- agents are specialist workers
- GFIR is the portable project representation
- policy/QA/review gates govern autonomous actions

## Initial reference engines

1. Godot
2. Web/Babylon.js/WebGPU

Use these to prove the abstraction before expanding to Unreal, Unity, Roblox, GameMaker, GDevelop, and others.

## Key risks

- false universality: engine features do not map 1:1
- uncontrolled agent writes
- asset licensing/provenance
- build reproducibility
- dependency sprawl
- console SDK access restrictions
- cross-platform packaging complexity
- silent capability loss

Design explicitly against these risks.
