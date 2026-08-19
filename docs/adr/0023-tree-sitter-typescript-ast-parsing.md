# ADR 0023 — Tree-sitter TypeScript AST Parsing

Status: Accepted
Build: B000025
Phase: M06 Code Intelligence

## Context

B000024 established a deterministic TypeScript-specific parser adapter using lexical declaration recognition and structural shielding. That foundation was insufficient for multiline grammar, regular-expression literals, template expressions, and other syntax whose structure cannot be represented reliably with brace counting and line-prefix recognition.

Phase 06 needs parser-derived TypeScript syntax evidence before targeted refactor planning and later semantic resolution can depend on source structure.

## Decision

Use the upstream Tree-sitter TypeScript grammar behind the existing provider-neutral `TypeScriptParserAdapter` boundary.

The adapter parses the complete TypeScript source file and derives supported top-level declaration evidence from syntax-tree nodes. Evidence remains deterministic and records declaration kind, name, parser-derived line, ordinal, and SHA-256 correlation output.

Malformed syntax fails closed when the parse tree contains error or missing nodes. Nested declarations are not promoted to repository-level top-level evidence. Multiple variable declarators are represented explicitly.

`tree-sitter` and `tree-sitter-typescript` are implementation dependencies of the syntax adapter only; they do not alter the shared provider/engine authority model.

## Consequences

- TypeScript syntax evidence is parser-derived instead of brace/line-prefix derived.
- Multiline declarations, generics, regex literals, and template expressions are handled by the grammar rather than ad-hoc shielding.
- The adapter remains deterministic and read-only.
- This does not provide TypeScript compiler binding, module resolution, type checking, control/data-flow analysis, LSP services, semantic call graphs, or source-mutation authority.
- Future C++ and Blueprint adapters remain separate language boundaries.

## Authority boundary

Parsing is analysis only. It grants no filesystem, repository, process, network, credential, engine, merge, publish, signing, or release authority. Any future source mutation must pass through the verified Autonomy Governor transaction boundary.
