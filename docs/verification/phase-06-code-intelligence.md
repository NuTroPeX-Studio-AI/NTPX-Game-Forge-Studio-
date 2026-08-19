# Phase 06 Code Intelligence verification

Phase status: IN PROGRESS

Current increment: repository index foundation candidate

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
- [ ] exact final candidate passes Rust formatting;
- [ ] exact final candidate passes strict Clippy with warnings denied;
- [ ] exact final candidate passes full workspace tests;
- [ ] exact final candidate passes desktop frontend typecheck/build;
- [ ] exact final candidate preserves Windows Autonomy Governor check/tests.

## Evidence lineage

- Initial CI run `32210625731` found rustfmt-only differences in `code-intelligence`; desktop frontend validation passed while the Rust job stopped before Clippy/tests. Formatter output was applied without changing the authority boundary or weakening tests.

## Explicit boundaries

- The initial drivers are lexical and are not represented as AST-complete, compiler-grade, or LSP-grade semantic analysis.
- Code Intelligence evidence does not authorize source mutation, tool execution, model invocation, repository changes, network/credential use, or engine execution.
- Later refactor/application paths must consume the verified Agent Orchestrator and Autonomy Governor boundaries.
- Phase 06 remains IN PROGRESS after this increment. AST-native parsing, resolved references/call graphs, incremental caching, diagnostics, semantic search, refactor planning/application, and governed source mutation remain pending.
