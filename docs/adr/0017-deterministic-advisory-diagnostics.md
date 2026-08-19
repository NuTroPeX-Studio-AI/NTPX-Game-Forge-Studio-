# ADR 0017 — Deterministic Advisory Lexical Diagnostics

Status: Accepted
Date: 2026-08-19

## Context

Phase 06 already provides deterministic lexical indexing, reference resolution, advisory dependency evidence, and incremental index reuse. The next prerequisite-safe capability is diagnostics, but the current language drivers are intentionally lexical and must not be represented as compiler-, AST-, linker-, or LSP-grade semantic analysis.

## Decision

Add a separate provider-neutral `code-intelligence-diagnostics` workspace crate that consumes the existing `ReferenceGraph` as immutable input.

The first diagnostic contract:

- emits `NTPX-CI-UNRESOLVED-REFERENCE` for lexical references with no exact indexed symbol match;
- emits `NTPX-CI-AMBIGUOUS-REFERENCE` when multiple exact indexed symbols match;
- emits no diagnostic for uniquely resolved references;
- preserves source path, line, target token, severity, message, and candidate symbol IDs;
- sorts and deduplicates ambiguous candidate IDs before evidence construction;
- canonically orders diagnostic records;
- records unresolved/ambiguous counts and SHA-256 report evidence;
- remains advisory and read-only.

## Consequences

The repository gains reproducible diagnostic evidence that can be consumed by later UI, semantic-search, planning, or evaluation layers without inventing mutation authority. Equivalent reference graphs yield equivalent ordered diagnostic reports and digests.

These diagnostics are lexical evidence only. They do not prove that an import/include/name is invalid under a language compiler, package manager, linker, build system, generated-code environment, Unreal reflection system, or LSP. False positives and false negatives remain possible until later language-service/AST/compiler integrations are implemented and verified.

No source write, tool/process execution, model invocation, network access, credential use, repository mutation at runtime, engine action, publishing, signing, or release authority is introduced. Any later writable repair/refactor path must remain behind the Agent Orchestrator and Autonomy Governor boundaries.
