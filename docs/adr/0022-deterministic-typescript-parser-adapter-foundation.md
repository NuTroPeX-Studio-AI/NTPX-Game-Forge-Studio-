# ADR 0022 — Deterministic TypeScript Parser Adapter Foundation

Status: Accepted
Build: B000024
Phase: 06 Code Intelligence

## Context

B000023 established AST-grade Rust syntax parsing behind the provider-neutral `code-intelligence-syntax` boundary. Phase 06 still lacked a TypeScript-specific parser adapter, leaving TypeScript dependent on the generic structural syntax service and lexical repository driver.

The next prerequisite-safe increment is a TypeScript-specific parsing boundary that can produce deterministic top-level declaration evidence without introducing a TypeScript compiler runtime, external language-server process, provider dependency, network authority, or source-mutation authority.

## Decision

Add `TypeScriptParserAdapter` inside `code-intelligence-syntax` and expose deterministic evidence for supported top-level TypeScript functions, classes, interfaces, type aliases, enums, namespaces, and `const`/`let`/`var` declarations.

The adapter records stable item kind, name, source line, ordinal, and SHA-256 correlation evidence. It tracks top-level brace depth while lexically shielding braces inside line comments, block comments, single/double-quoted strings, and multiline template strings. Nested declarations are not promoted to top-level evidence. Empty paths, language mismatch, malformed recognized declarations, and unbalanced lexical structure fail closed.

No new external dependency is introduced in this increment.

## Consequences

- TypeScript now has an explicit language-specific parser-adapter boundary parallel to the Rust adapter boundary.
- Deterministic evidence can be consumed by later parser/language-service upgrades without changing the provider-neutral Code Intelligence authority model.
- Comments and ordinary quoted/template strings no longer corrupt the adapter's top-level brace-depth accounting.
- The adapter remains intentionally smaller than a TypeScript compiler or language service.

## Explicit limits

B000024 is a parser-adapter foundation, not an AST-grade TypeScript compiler parser. It does not implement the complete ECMAScript/TypeScript grammar, regular-expression literal parsing, template-expression semantics, decorators/JSX/TSX semantics, module resolution, symbol binding, type checking, control/data-flow analysis, semantic reference resolution, true call graphs, LSP operations, refactor execution, source mutation, model/tool execution, repository mutation, or engine execution.

All protected merge, signing, credential, publishing, destructive, and release actions remain human-gated by repository policy and the Autonomy Governor boundary.
