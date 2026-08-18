# ADR 0003 — Phase 02 GameSpec completion contract

**Status:** Accepted  
**Date:** 2026-08-18

## Decision

GameSpec 0.1.0 is the canonical engine-neutral project contract. Phase 02 is complete only when the repository can decode, validate, normalize, migrate, graph, and deterministically compile a reference specification without an AI model or network dependency.

## Required properties

- explicit versioning;
- canonical stable identifiers;
- deterministic generated IDs using a fixed algorithm;
- full Phase 02 domain coverage;
- explicit direct references;
- cycle detection for dependency-bearing edges;
- asset license/provenance metadata;
- checked-in JSON Schema and CLI schema emission;
- legacy `0.1` migration;
- deterministic compilation manifest;
- executable CLI and tests.

## Deferred

Engine adapters, autonomous writes, permissions, model routing, agent orchestration, and production release authority remain outside Phase 02.
