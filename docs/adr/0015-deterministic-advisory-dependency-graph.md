# ADR 0015 — Deterministic advisory dependency graph

Status: Accepted for Phase 06 incremental build B000017

## Context

B000015 established deterministic lexical indexing and B000016 established deterministic resolved/ambiguous/unresolved lexical reference evidence. Phase 06 still requires dependency-graph intelligence, but compiler/LSP-grade semantic resolution and AST-native language services are not yet implemented. Promoting ambiguous or unresolved lexical references into guessed graph edges would overstate the available evidence and violate the fail-closed architecture.

## Decision

Add a deterministic advisory dependency graph derived only from the verified B000016 lexical reference graph.

- Only uniquely `Resolved` references become dependency edges.
- Each edge preserves source path/reference line/target token and target symbol ID/path/line evidence.
- `Ambiguous` references produce no guessed edge and remain visible through an explicit deterministic count.
- `Unresolved` references produce no guessed edge and remain visible through an explicit deterministic count.
- Edges are canonically ordered and the graph carries SHA-256 correlation evidence over the ordered edges and unresolved/ambiguous counts.
- Equivalent repository input order produces equivalent dependency-graph evidence.
- The graph consumes existing `RepositoryIndex`/reference evidence and adds no provider SDK, game-engine dependency, process execution, network access, credential use, source mutation, repository mutation, or release authority.

## Deliberate limits

This increment is a lexical file-to-symbol dependency graph, not a compiler/LSP semantic dependency graph and not a call graph. It does not model scopes, aliases, namespaces, overload resolution, imports, macros, templates/generics, type inference, dynamic dispatch, linker behavior, or engine reflection. Those stronger claims remain gated on later AST-native/language-service work.

## Consequences

Phase 06 gains a deterministic dependency substrate that downstream diagnostics, impact analysis, semantic services, and refactor planning can consume without manufacturing certainty. Ambiguity remains fail-closed. Writable refactor/application and execution paths remain behind the verified Agent Orchestrator and Autonomy Governor boundaries.
