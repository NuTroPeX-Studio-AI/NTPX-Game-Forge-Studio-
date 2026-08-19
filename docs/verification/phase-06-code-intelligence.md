# Phase 06 Code Intelligence verification

Phase status: IN PROGRESS

Latest validated increment: `NTPX-GFΩ-G0-v0.6.0-dev.2-M06-B000016` — deterministic lexical reference resolution

## B000015 acceptance gate — deterministic index foundation

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
- [x] exact B000015 stamped head passed Rust formatting, strict Clippy, full workspace tests, desktop frontend typecheck/build, and Windows Autonomy Governor check/tests in CI `32211032036`.

## B000016 acceptance gate — lexical reference resolution

- [x] repository-level reference graph consumes deterministic `RepositoryIndex` evidence;
- [x] unique exact lexical target/symbol-name matches resolve to stable symbol ID, path, and line evidence;
- [x] duplicate exact symbol matches remain explicitly `Ambiguous` and select no arbitrary winner;
- [x] missing matches remain explicitly `Unresolved` rather than being discarded;
- [x] candidate ordering is deterministic;
- [x] equivalent source enumeration order produces equivalent reference-graph evidence;
- [x] reference graph carries deterministic SHA-256 correlation evidence;
- [x] unit coverage exercises resolved, ambiguous, unresolved, and order-determinism behavior;
- [x] no compiler/LSP-grade semantic claim is introduced;
- [x] no mutation, execution, network, credential, engine, publish, signing, or release authority is introduced;
- [x] repaired implementation candidate passed Rust formatting, strict Clippy, full workspace tests, desktop frontend typecheck/build, and Windows Autonomy Governor check/tests in CI `32214521151`.

## Evidence lineage

### B000015

- CI `32210625731` found rustfmt-only differences in `code-intelligence`; formatter output was applied without changing the authority boundary or weakening tests.
- CI `32210746616` passed formatting and identified one strict-Clippy `doc_markdown` issue; the documentation was corrected rather than suppressing the lint.
- Repaired implementation candidate CI `32210838815` passed the full repository gate.
- Exact stamped B000015 head CI `32211032036` passed the same full repository gate. B000015 is VERIFIED.

### B000016

- CI `32214476226` failed only on rustfmt drift in `crates/code-intelligence/src/resolution.rs`; frontend validation continued successfully and the Rust job stopped before Clippy/tests.
- Formatter output was applied exactly. No lint, validation, test, or security control was weakened.
- Repaired implementation candidate CI `32214521151` passed Rust formatting, strict Clippy with warnings denied, full workspace tests, desktop frontend typecheck/build, and Windows Autonomy Governor check/tests.
- ADR 0014 and build-history evidence record the deterministic lexical-resolution boundary.
- The exact stamped B000016 documentation/build-metadata head must pass the same CI gate before the final B000016 classification is VERIFIED.

## Explicit boundaries

- The drivers and B000016 resolver remain lexical. They are not represented as AST-complete, compiler-grade, linker-grade, or LSP-grade semantic analysis.
- Exact target/name correlation does not model scopes, aliases, namespaces, imports, overloads, templates/generics, type inference, macros, or engine reflection semantics.
- Ambiguity remains explicit and never grants permission to choose or mutate a target.
- Code Intelligence evidence does not authorize source mutation, tool execution, model invocation, repository changes, network/credential use, or engine execution.
- Later refactor/application paths must consume the verified Agent Orchestrator and Autonomy Governor boundaries.
- Phase 06 remains IN PROGRESS after B000016. AST-native parsing/language services, compiler/LSP-grade resolution, call/dependency graphs, incremental caching, diagnostics, semantic search, refactor planning/application, and governed source mutation remain pending.
