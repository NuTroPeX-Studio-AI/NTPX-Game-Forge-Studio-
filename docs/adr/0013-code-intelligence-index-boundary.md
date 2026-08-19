# ADR 0013 — Deterministic Code Intelligence Index Boundary

Status: Accepted for Phase 06 increment candidate

## Context

Phase 06 requires repository parsing/indexing, symbols, references, dependency/call graphs, change-impact analysis, and refactor intelligence. Phase 05 is verified and already separates scheduling from authorization and execution. Code Intelligence must therefore begin as an advisory read/index boundary rather than a writable code-modification path.

The first increment needs a deterministic representation that can be consumed by later agents without depending on a model provider, game engine, editor, or external language-service process.

## Decision

Introduce a provider-neutral `code-intelligence` Rust workspace crate with:

- explicit language-driver interfaces;
- initial lexical drivers for Rust, TypeScript, C++, and Blueprint metadata;
- stable source paths, content SHA-256 digests, symbol IDs, and reference records;
- deterministic repository ordering and a SHA-256 repository evidence digest;
- fail-closed duplicate-path, missing-driver, and driver/language mismatch validation;
- generated-file metadata preserved in the index;
- advisory change-impact output derived from indexed references.

Equivalent source sets and driver enumeration orders must produce equivalent repository evidence.

## Authority boundary

The Code Intelligence index is evidence and analysis only. It grants no permission to edit files, execute tools, invoke models, access networks or credentials, mutate repositories, execute engines, publish, sign, merge, or release.

Any later refactor application or source mutation must use the existing Agent Orchestrator and Autonomy Governor boundaries rather than introducing a parallel write path.

## Deliberate limitations

The initial lexical drivers are not represented as AST-complete or semantically resolved language services. They do not yet provide:

- AST-native parsing;
- compiler/LSP-grade symbol resolution;
- resolved call graphs;
- incremental cache invalidation;
- semantic embeddings/search;
- refactor transformation generation or application;
- executable repository mutation.

These remain Phase 06 work and require separate verification evidence.

## Consequences

The project gains a deterministic, testable Code Intelligence substrate that can be safely extended with richer parsers and language adapters while preserving reproducibility and authority separation. False semantic precision is avoided by explicitly labeling the first driver layer as lexical.
