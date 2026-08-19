# ADR 0018 — Deterministic Advisory Symbol Search

## Status

Accepted for Phase 06 Build B000020.

## Context

Phase 06 already provides deterministic lexical indexing, reference resolution, dependency evidence, incremental index reuse, and diagnostics. A query surface is needed before later semantic-search work, but the repository does not yet have AST/LSP-grade semantic models or embedding infrastructure.

## Decision

Add a separate provider-neutral `code-intelligence-search` crate that consumes `RepositoryIndex` evidence and performs deterministic local lexical symbol-name ranking. Queries are whitespace-normalized and lowercased. Exact matches outrank prefixes, which outrank substrings. Equal scores are resolved by stable path, name, line, and symbol ID ordering. Result evidence includes the verified repository digest, normalized query, ordered hits, and SHA-256 correlation digest.

## Authority boundary

Search is advisory and read-only. It grants no source mutation, model, tool, network, credential, repository, engine, publishing, signing, or release authority.

## Consequences

The system gains a reproducible symbol query primitive that can later be composed with AST/LSP or embedding-backed search. It must not be described as semantic search, compiler resolution, or language-service proof. Phase 06 remains in progress.
