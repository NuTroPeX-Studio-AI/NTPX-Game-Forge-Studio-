# ADR 0002 — GameSpec is the canonical machine-readable project model

**Status:** Accepted  
**Date:** 2026-08-18

## Decision

NTPX GameForge Ω∞ will use a versioned, engine-neutral `GameSpec` document as the canonical machine-readable representation of game intent and project state that must be reproducible.

The first schema version is `0.1.0`. Shared GameSpec code remains independent of Unreal, Unity, Godot, AI providers, agent frameworks, and cloud services.

## Consequences

- Conversations and model memory cannot be the only source of truth.
- Engine adapters must consume compiled or validated GameSpec output rather than inventing untracked project logic.
- Asset records carry licensing and provenance data from the start.
- References are explicit IDs rather than implicit object names.
- Later migrations must preserve schema lineage.

## Deferred

Graph compilation, cycle detection, migration execution, and engine code generation are separate later milestones.
