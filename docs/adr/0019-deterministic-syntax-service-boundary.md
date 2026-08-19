# ADR 0019 — Deterministic Syntax-Service Boundary

Status: Accepted for B000021

## Context

Phase 06 already provides deterministic lexical indexing, reference resolution, dependency evidence, incremental reuse, diagnostics, and lexical symbol search. The next roadmap dependency is a language-service/AST parsing boundary, but introducing compiler/LSP semantics in one step would be too broad for the smallest verifiable increment.

## Decision

Add the provider-neutral `code-intelligence-syntax` workspace crate. It consumes the existing `code-intelligence::SourceFile` and `Language` types through a `SyntaxService` interface and emits deterministic structural syntax evidence consisting of declaration, block-open, and block-close nodes plus a SHA-256 correlation digest.

The initial `StructuralSyntaxService` recognizes language-specific declaration prefixes and balanced brace structure. Invalid language binding and unbalanced block structure fail closed.

## Determinism

For identical source path, language, and content, node ordering, line/depth evidence, and the syntax evidence digest are deterministic. No provider, engine, model, network, credential, or runtime state participates in the result.

## Authority boundary

Syntax evidence is advisory/read-only. It grants no source mutation, tool/process execution, model invocation, network access, credential use, repository mutation, engine action, publishing, signing, merge, or release authority.

## Deliberate limits

B000021 is a language-service contract and deterministic structural-syntax substrate. It is **not** a compiler-native AST, LSP semantic model, resolved call graph, type checker, control-flow graph, or embedding-based semantic index. Braces inside comments/string literals are not compiler-tokenized by this initial structural service. Native parser adapters and compiler/LSP-grade semantics remain future M06 increments.

## Consequences

Later Rust/TypeScript/C++/Blueprint-native parser or language-server adapters can implement the same provider-neutral boundary without changing the authorization model. M06 remains IN PROGRESS until its remaining acceptance criteria are implemented and verified.
