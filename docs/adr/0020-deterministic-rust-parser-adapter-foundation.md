# ADR 0020 — Deterministic Rust Parser Adapter Foundation

Status: Accepted for B000022

## Context

B000021 established a provider-neutral syntax-service boundary and deterministic structural syntax evidence. The next prerequisite-safe Phase 06 step is to begin language-specific parser adapters without prematurely claiming compiler-native AST or LSP semantics.

## Decision

Add a Rust-specific parser adapter inside `code-intelligence-syntax`. `RustParserAdapter` consumes the existing `SourceFile` contract and emits deterministic top-level Rust item evidence for recognized functions, structs, enums, traits, modules, impls, type aliases, constants, and statics.

The adapter records stable item kind, name, source line, ordinal, and SHA-256 evidence. It recognizes ordinary, `pub`, `pub(crate)`, and `pub(super)` visibility forms, tracks brace depth so nested methods are not promoted to top-level items, validates extracted identifiers, and fails closed on empty paths, language mismatch, malformed recognized declarations, and unbalanced block structure.

## Determinism

For identical source path and content, item ordering, line/ordinal evidence, and the parser evidence digest are deterministic. No model, provider, engine, network, credential, repository runtime, or external language service participates in the result.

## Authority boundary

Parser evidence is advisory/read-only. It grants no source mutation, repository write, process/tool execution, model invocation, network access, credential use, engine action, publishing, signing, protected merge, or release authority.

## Deliberate limits

B000022 is a Rust-specific parser adapter foundation, not a `rustc` AST, macro expander, type checker, name resolver, control-flow graph, call graph, or LSP semantic service. Its brace tracking is not compiler-tokenized, so braces in comments or string literals remain outside the verified guarantee. TypeScript, C++, and Blueprint-specific adapters remain unfinished.

## Consequences

Phase 06 gains its first language-specific parsing boundary while preserving the provider-neutral architecture and rollback-safe read-only posture. Later increments can replace or augment this adapter with compiler-grade parsing behind the same authority separation. M06 remains IN PROGRESS.
