# ADR 0024 — C++ AST and Blueprint Metadata Parser Coverage

Status: Accepted
Build: B000026
Phase: M06 Code Intelligence

## Context

Phase 06 already had AST-grade Rust and TypeScript syntax adapters, but safe engine-facing code intelligence still lacked parser-specific evidence for C++ and Blueprint metadata.

## Decision

Add a Tree-sitter-backed C++ parser adapter and a deterministic textual Blueprint metadata parser behind the existing provider/engine-neutral syntax boundary.

The C++ adapter extracts bounded top-level declaration evidence for functions, classes, structs, enums, namespaces, and aliases with parser-derived line/ordinal evidence and deterministic SHA-256 correlation output. Templates are unwrapped only far enough to expose supported top-level declarations; class methods are not promoted to repository-level evidence.

The Blueprint adapter parses the repository's textual metadata interchange records (`Class=`, `Function=`, `Variable=`, `ParentClass=`, and `Asset=`) and fails closed on malformed/unknown records. It is explicitly not a parser for Unreal binary `.uasset` files.

## Consequences

- Rust, TypeScript, C++, and Blueprint metadata now all have language-specific parser adapters.
- C++ parsing does not claim Clang preprocessing/compilation, overload/type resolution, template instantiation, or LSP semantics.
- Blueprint binary asset inspection remains a Phase 08 Unreal Engine adapter responsibility.
- All parser evidence remains analysis-only and grants no mutation or execution authority.

## Authority boundary

No parser adapter performs filesystem writes, process/tool execution, network access, credential use, engine execution, merge, publish, signing, or release operations. Future source mutation must pass through the verified Autonomy Governor transaction boundary.
