# Phase 06 Code Intelligence verification

Phase status: IN PROGRESS

Latest validated increment: `NTPX-GFΩ-G0-v0.6.0-dev.3-M06-B000017` — deterministic advisory lexical dependency graph

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
- [x] repaired implementation candidate passed Rust formatting, strict Clippy, full workspace tests, desktop frontend typecheck/build, and Windows Autonomy Governor check/tests in CI `32214521151`;
- [x] exact stamped B000016 head passed the same complete repository gate in CI `32214681258`.

## B000017 acceptance gate — advisory lexical dependency graph

- [x] dependency graph consumes the deterministic B000016 reference graph;
- [x] only uniquely resolved references become dependency edges;
- [x] edges retain source path/reference line/target token and stable target symbol ID/path/line evidence;
- [x] ambiguous references never become guessed dependency edges and remain explicitly counted;
- [x] unresolved references never become guessed dependency edges and remain explicitly counted;
- [x] dependency edges are canonically ordered;
- [x] equivalent source enumeration order produces equivalent dependency-graph evidence;
- [x] dependency graph carries deterministic SHA-256 correlation evidence;
- [x] unit coverage exercises resolved-edge creation, ambiguity/unresolved fail-closed behavior, and order determinism;
- [x] no AST/compiler/LSP semantic or call-graph claim is introduced;
- [x] no mutation, execution, network, credential, engine, publish, signing, or release authority is introduced;
- [x] repaired implementation candidate passed Rust formatting, strict Clippy, full workspace tests, desktop frontend typecheck/build, and Windows Autonomy Governor check/tests in CI `32217787844`;
- [x] exact stamped B000017 candidate head passed the same complete repository gate in CI `32218018982`.

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
- Exact stamped B000016 head CI `32214681258` passed the same complete repository gate with build metadata, roadmap, ADR 0014, verification evidence, and changelog records present.
- B000016 is VERIFIED.

### B000017

- CI `32217736330` failed only on rustfmt drift in `crates/code-intelligence/src/dependency.rs`; desktop frontend typecheck/build passed and Rust Clippy/tests did not run after the formatting failure.
- Exact formatter output was applied. No lint, validation, test, or security control was weakened.
- Repaired implementation candidate CI `32217787844` passed Rust formatting, strict Clippy with warnings denied, full workspace tests, desktop frontend typecheck/build, and Windows Autonomy Governor check/tests.
- ADR 0015 records the deterministic advisory dependency-graph boundary.
- Exact stamped B000017 candidate head CI `32218018982` passed the same complete repository gate with build metadata, roadmap, ADR 0015, verification evidence, build history, and changelog records present.
- B000017 is VERIFIED. The final synchronized verification-status head is subject to the same repository CI gate before this evidence record is considered fully synchronized.

## Explicit boundaries

- The drivers, B000016 resolver, and B000017 dependency graph remain lexical. They are not represented as AST-complete, compiler-grade, linker-grade, or LSP-grade semantic analysis.
- B000017 is a file-to-symbol dependency graph derived only from uniquely resolved lexical evidence; it is not a call graph.
- Exact target/name correlation does not model scopes, aliases, namespaces, imports, overloads, templates/generics, type inference, macros, dynamic dispatch, or engine reflection semantics.
- Ambiguity remains explicit and never grants permission to choose or mutate a target.
- Code Intelligence evidence does not authorize source mutation, tool execution, model invocation, repository changes, network/credential use, or engine execution.
- Later refactor/application paths must consume the verified Agent Orchestrator and Autonomy Governor boundaries.
- Phase 06 remains IN PROGRESS after B000017. AST-native parsing/language services, compiler/LSP-grade semantic resolution and call graphs, incremental caching, diagnostics, semantic search, refactor planning/application, and governed source mutation remain pending.
