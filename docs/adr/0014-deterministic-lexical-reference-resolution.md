# ADR 0014 — Deterministic lexical reference resolution

Status: Accepted for Phase 06 incremental build B000016

## Context

B000015 established deterministic lexical indexing but intentionally left repository-level reference resolution unfinished. Phase 06 requires reference intelligence before later call/dependency graphs, diagnostics, semantic search, and refactor planning can be trusted.

## Decision

Add an advisory repository-level reference graph that resolves each indexed lexical reference token against exact indexed symbol names.

- A unique exact symbol-name match is recorded as `Resolved` with stable symbol ID, path, and line evidence.
- Multiple exact matches are recorded as `Ambiguous`; candidates are deterministically ordered and no arbitrary winner is selected.
- Missing matches are preserved as `Unresolved` evidence rather than silently discarded.
- Equivalent repository indexes produce equivalent ordered graph records and SHA-256 evidence.
- The resolver consumes only the deterministic `RepositoryIndex`; it performs no source mutation, model invocation, process/tool execution, network access, credential use, repository write, or engine action.

## Deliberate limits

This is lexical correlation, not AST-, compiler-, linker-, or LSP-grade semantic resolution. Exact token/name matching does not model namespaces, imports, overloads, scopes, aliases, templates/generics, type inference, macros, or engine reflection semantics. Ambiguity remains explicit until a later language-service increment can prove stronger resolution.

## Consequences

Phase 06 gains a deterministic resolved/ambiguous/unresolved evidence layer suitable as an input to later dependency/call graph work without overstating semantic precision. Writable refactor/application paths remain gated behind the Agent Orchestrator and Autonomy Governor.
