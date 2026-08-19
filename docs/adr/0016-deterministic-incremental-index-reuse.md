# ADR 0016 — Deterministic Incremental Index Reuse

Status: Accepted
Date: 2026-08-19
Phase: 06 Code Intelligence
Build: B000018

## Context

B000015 established deterministic repository indexing, B000016 added lexical reference resolution, and B000017 added advisory lexical dependency evidence. Re-indexing every unchanged source file is correct but wastes parsing/indexing work and does not establish the incremental foundation required by Phase 06.

The next increment must remain provider-neutral, engine-neutral, deterministic, read-only, and fail closed when prior evidence cannot be trusted.

## Decision

Add `build_repository_index_incremental` to the `code-intelligence` crate.

A prior `FileIndex` may be reused only when all of these inputs match exactly:

- repository path;
- language;
- generated-file state;
- SHA-256 digest of current source content.

The prior `RepositoryIndex` evidence digest is recomputed before any cached record is trusted. A mismatch returns an explicit `PriorEvidenceMismatch` and no cache reuse occurs.

Exact language-driver availability remains mandatory even for unchanged files. Incremental reuse therefore cannot silently bypass or weaken the configured language boundary.

Removed files are omitted from the new index and reported in deterministic reuse/reindex/removal counts. The newly assembled repository index always receives a freshly recomputed deterministic repository evidence digest.

## Consequences

- Unchanged files can avoid language-driver indexing work while preserving byte-equivalent deterministic index evidence.
- Content changes and generated-state changes invalidate reuse.
- Corrupted or tampered prior repository evidence fails closed before reuse.
- Equivalent source enumeration orders produce equivalent incremental results.
- This is an in-memory deterministic reuse primitive. It is not yet a persistent disk cache, distributed cache, AST cache, compiler/LSP cache, semantic dependency invalidator, or performance guarantee.
- No source mutation, model invocation, process/tool execution, network access, credential use, repository mutation at runtime, engine execution, publishing, signing, or release authority is introduced.

## Verification

Implementation candidate CI run `32221974551` passed Rust formatting, strict Clippy with warnings denied, the full Rust workspace test suite, desktop TypeScript typecheck/build, and Windows Autonomy Governor check/tests after a rustfmt-only repair from initial CI run `32221871332`.
