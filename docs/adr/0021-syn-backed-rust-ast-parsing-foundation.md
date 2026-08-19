# ADR 0021 — Syn-Backed Rust AST Parsing Foundation

Status: Accepted
Build: B000023
Phase: 06 Code Intelligence

## Context

B000022 established a deterministic Rust-specific parser adapter, but its declaration extraction and brace-depth logic were intentionally lexical. That foundation could not correctly treat braces inside comments or string literals as Rust syntax, could not naturally consume multiline item declarations, and was explicitly not an AST parser.

The next prerequisite-safe Phase 06 increment is to replace those line-prefix/brace heuristics with a real Rust syntax parser while preserving the provider-neutral Code Intelligence boundary and read-only authority model.

## Decision

`RustParserAdapter` now parses the complete source file with `syn` using its full Rust syntax feature set before producing supported top-level item evidence.

The adapter emits deterministic evidence for top-level functions, structs, enums, traits, modules, impl blocks, type aliases, constants, and statics. Source lines come from parser spans with `proc-macro2` span-location support. Stable output retains item kind, name, line, ordinal, and SHA-256 correlation evidence.

Unsupported top-level Rust items are ignored only after the complete file has parsed successfully. Invalid Rust fails closed with parser error evidence. Language mismatch and empty source paths also fail closed before parsing.

## Consequences

- Comments, strings, nested braces, attributes, generic declarations, and multiline item syntax are handled by a Rust grammar parser instead of brace counting.
- Nested methods remain outside top-level item evidence because output is derived from `syn::File::items`.
- The parser dependency remains isolated inside `code-intelligence-syntax`; shared GameSpec, Governor, Model Router, and orchestration contracts do not acquire a Rust compiler/provider dependency.
- `syn` and `proc-macro2` are added as workspace dependencies. The repository currently has no committed `Cargo.lock`; dependency resolution therefore remains governed by the existing workspace semver policy rather than an exact lockfile.

## Explicit limits

B000023 is AST-grade syntax parsing, not `rustc` compilation. It does not perform macro expansion, cfg evaluation, module loading, name resolution, type checking, borrow checking, control-flow/data-flow analysis, semantic reference resolution, true call graphs, LSP operations, source mutation, model/tool execution, repository mutation, or engine execution.

All protected merge, signing, credential, publishing, destructive, and release actions remain human-gated by the existing Autonomy Governor/repository policy boundary.
