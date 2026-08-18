# GameSpec 0.1.0

GameSpec is the canonical machine-readable project model for NTPX GameForge Ω∞.

## Phase 02 scope

Phase 02 defines typed Rust structures, the checked-in JSON Schema contract, stable identifiers, serialization/deserialization, direct-reference validation, provenance metadata, and a Signal Born reference document.

The following are intentionally deferred to later phases:

- dependency graph compilation and cycle analysis;
- deterministic generated IDs;
- migrations between schema versions;
- engine-specific code generation;
- autonomous writes or tool execution.

## Canonical domains

- Project
- Architecture and modules
- Features
- Components and entities
- Inputs and abilities
- NPCs and dialogue
- Quests and worlds
- Assets, licensing, and provenance
- Tests
- Build targets

## Identifier grammar

Canonical IDs match:

```text
^[a-z0-9][a-z0-9._-]*$
```

The Rust `SpecId` type can validate IDs at construction time. `GameSpec::validate` also validates IDs after deserialization so untrusted JSON cannot bypass the rule.

## Validation boundary

`GameSpec::validate` currently checks:

1. schema version support;
2. canonical identifier grammar;
3. global ID uniqueness for top-level objects;
4. local uniqueness for dialogue nodes, quest steps, and world regions;
5. direct module, component, NPC, test, dialogue-node, and generic references.

Phase 03 will compile the complete dependency graph, add cycle detection, deterministic compilation output, and migration interfaces.
