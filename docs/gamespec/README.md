# GameSpec 0.1.0

GameSpec is the canonical machine-readable project model for NTPX GameForge Ω∞.

## Phase 02 completion scope

Phase 02 provides:

- engine-neutral typed Rust domain model;
- canonical IDs and deterministic generated IDs;
- stable JSON serialization/deserialization;
- versioned JSON Schema emission;
- direct reference validation;
- deterministic dependency graph extraction;
- module/feature cycle rejection;
- migration from the verified legacy Phase 02 `0.1` shape;
- compiler manifest generation;
- command-line validation, inspection, graphing, compilation, migration, normalization, and schema output;
- Signal Born reference and migration fixtures;
- asset license and provenance fields.

## CLI

```text
gamespec-compiler validate <file>
gamespec-compiler inspect <file>
gamespec-compiler graph <file>
gamespec-compiler compile <file>
gamespec-compiler migrate <file>
gamespec-compiler normalize <file>
gamespec-compiler schema
```

## Boundary

GameSpec remains independent of AI providers and game engines. Phase 03 may consume these validated specifications to implement governance, but Phase 02 does not grant autonomous write or tool authority.
