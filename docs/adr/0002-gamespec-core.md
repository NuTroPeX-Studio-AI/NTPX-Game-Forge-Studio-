# ADR 0002 — GameSpec core model

## Status
Accepted for Phase 02 increment 1.

## Decision
Introduce an engine-neutral `gamespec` Rust crate as the machine-readable source-of-truth layer. The first increment owns a minimal project/module/feature/entity hierarchy, deterministic JSON serialization, semantic validation, and a Signal Born reference fixture.

## Constraints
- No AI-provider dependency.
- No game-engine dependency.
- Validation errors are structured with paths.
- Invalid specifications fail before downstream compilation.
- Later Phase 02 increments may extend the model without bypassing version checks or validation.

## Deferred
JSON Schema generation, references/dependency graphs, migrations, full domain types, compiler CLI, and deterministic generated IDs remain Phase 02 work.
