# Phase 06 Code Intelligence verification

Phase status: IN PROGRESS

Validated increment: `NTPX-GFΩ-G0-v0.6.0-dev.1-M06-B000015` — deterministic repository index foundation

## Increment acceptance gate

- [x] provider-neutral `code-intelligence` Rust workspace crate;
- [x] explicit language-driver boundary;
- [x] Rust, TypeScript, C++, and Blueprint-metadata lexical driver contracts;
- [x] deterministic file ordering independent of input enumeration order;
- [x] stable SHA-256 source content evidence;
- [x] stable symbol identifiers;
- [x] lexical reference records with source line evidence;
- [x] generated-file metadata preserved in the index;
- [x] deterministic repository SHA-256 correlation evidence;
- [x] duplicate source paths fail closed;
- [x] missing language drivers fail closed;
- [x] driver/language mismatch fails closed;
- [x] advisory change-impact output is deterministic;
- [x] no code mutation or execution authority added;
- [x] repaired implementation candidate passes Rust formatting;
- [x] repaired implementation candidate passes strict Clippy with warnings denied;
- [x] repaired implementation candidate passes full workspace tests;
- [x] repaired implementation candidate passes desktop frontend typecheck/build;
- [x] repaired implementation candidate preserves Windows Autonomy Governor check/tests.

## Evidence lineage

- Initial CI run `32210625731` found rustfmt-only differences in `code-intelligence`; desktop frontend validation passed while the Rust job stopped before Clippy/tests. Formatter output was applied without changing the authority boundary or weakening tests.
- CI run `32210746616` passed formatting and identified one strict-Clippy `doc_markdown` issue in crate documentation. The documentation was corrected rather than suppressing the lint.
- Repaired implementation candidate CI run `32210838815` passed Rust formatting, strict Clippy with warnings denied, full workspace tests, desktop frontend typecheck/build, and Windows Autonomy Governor check/tests.
- Build B000015, changelog, roadmap, ADR 0013, and this verification record were stamped after that implementation gate. The exact stamped documentation/build-metadata head must pass the same CI gate before the increment is classified VERIFIED.

## Explicit boundaries

- The initial drivers are lexical and are not represented as AST-complete, compiler-grade, or LSP-grade semantic analysis.
- Code Intelligence evidence does not authorize source mutation, tool execution, model invocation, repository changes, network/credential use, or engine execution.
- Later refactor/application paths must consume the verified Agent Orchestrator and Autonomy Governor boundaries.
- Phase 06 remains IN PROGRESS after this increment. AST-native parsing, resolved references/call graphs, incremental caching, diagnostics, semantic search, refactor planning/application, and governed source mutation remain pending.
