# Phase 06 Code Intelligence verification

Phase status: VERIFIED

Final build: `NTPX-GFΩ-G0-v0.6.0-dev.14-M06-B000028`

Phase 07 Agent Zero Execution is NEXT. Phase 06 passed every required implementation, candidate, VERIFIED-evidence, synchronization, and merge-readback gate.

## Acceptance gate

### Deterministic repository intelligence

- [x] provider-neutral `code-intelligence` Rust workspace crate;
- [x] deterministic file ordering and repository evidence;
- [x] stable SHA-256 source evidence and stable symbol identifiers;
- [x] explicit Rust, TypeScript, C++, and textual Blueprint metadata language boundaries;
- [x] lexical reference evidence with explicit resolved/ambiguous/unresolved outcomes;
- [x] deterministic advisory dependency/change-impact evidence;
- [x] fail-closed incremental index reuse bound to verified prior evidence;
- [x] deterministic advisory diagnostics and lexical symbol search;
- [x] generated-file metadata remains explicit.

### Language-specific syntax substrate

- [x] provider-neutral syntax-service boundary;
- [x] Rust full-file syntax parsing through `syn`;
- [x] TypeScript full-file syntax parsing through Tree-sitter;
- [x] C++ top-level syntax parsing through Tree-sitter;
- [x] deterministic textual Blueprint metadata parser;
- [x] parser/language/path/malformed-input failures remain fail closed.

### Safe edit planning and governed application

- [x] B000027 deterministic exact whole-file edit plans bind path/language/current source/repository/replacement evidence;
- [x] generated-source and no-op plans are rejected;
- [x] stale repository or current-source evidence is rejected;
- [x] B000028 recomputes replacement-content SHA-256 and full plan correlation evidence before mutation;
- [x] B000028 requires an existing canonical repository root;
- [x] B000028 accepts only normal non-empty repository-relative plan paths and rejects traversal/absolute/prefix/current-directory paths;
- [x] target path is resolved through the Autonomy Governor filesystem boundary before source re-read;
- [x] current on-disk source is re-read and B000027 freshness validation is rerun immediately before the governed write path;
- [x] direct mutation uses `GovernanceRuntime::write_file` only;
- [x] human-gated mutation uses `write_file_with_approval` only;
- [x] a Code Intelligence plan cannot override denial or satisfy/weaken a human approval gate;
- [x] successful application returns the existing audited rollback-capable `TransactionReceipt`;
- [x] executable coverage proves successful mutation + rollback, stale source rejection, replacement tamper rejection, repository escape rejection, direct human-gate refusal, and successful exact approved application.

### Repository and merge gates

- [x] implementation Rust formatting passed;
- [x] implementation strict Clippy passed with warnings denied;
- [x] implementation full workspace tests passed;
- [x] implementation desktop frontend typecheck/build passed;
- [x] implementation Windows Autonomy Governor check/tests passed;
- [x] implementation NTPX Fortress bootstrap gate passed;
- [x] exact stamped B000028 candidate passed complete repository CI `32261483993`;
- [x] exact stamped B000028 candidate passed Fortress `32261483663`;
- [x] final VERIFIED evidence head passed complete repository CI `32262188186`;
- [x] final VERIFIED evidence head passed Fortress `32262188321`;
- [x] final synchronization head `301980124823397d72592fa3f33186898bfb0844` passed complete CI `32262420120`;
- [x] final synchronization head passed Fortress `32262420198`;
- [x] PR #17 squash-merged the exact tested head to `main` as `094da1cc35efc596568fdc77bb7f3551093e8079`;
- [x] default-branch readback confirmed B000028 / Phase 06 VERIFIED / Phase 07 NEXT;
- [x] NTPX Fortress bootstrap workflow remains present on `main`.

## Phase 06 build lineage

- B000015 — deterministic provider-neutral code index foundation.
- B000016 — deterministic lexical reference resolution.
- B000017 — deterministic advisory lexical dependency graph.
- B000018 — fail-closed deterministic incremental index reuse.
- B000019 — deterministic advisory lexical diagnostics.
- B000020 — deterministic advisory lexical symbol search.
- B000021 — provider-neutral syntax-service boundary.
- B000022/B000023 — Rust parser foundation upgraded to `syn` full-file syntax parsing.
- B000024/B000025 — TypeScript parser foundation upgraded to Tree-sitter full-file syntax parsing.
- B000026 — Tree-sitter C++ plus deterministic textual Blueprint metadata parsing.
- B000027 — deterministic evidence-bound exact whole-file edit planning with stale repository/source rejection.
- B000028 — governed evidence-bound exact edit application through the verified Autonomy Governor reversible transaction boundary.

Detailed build evidence remains in `docs/verification/phase-06-build-*.md`, `.gameforge/history/`, and the accepted Phase 06 ADRs.

## B000028 evidence lineage

Initial closure CI `32260168938` stopped the Rust job at rustfmt-only differences in `code-intelligence-apply`; frontend validation passed and no lint, test, security, or authorization check was weakened.

Formatter-equivalent repair head `d63050c8184521e6408b6bb9923af4da327f0ebf` passed complete repository CI `32260654093`; NTPX Fortress `32260654100` also passed.

The exact full B000028 candidate passed complete repository CI `32261483993` and Fortress `32261483663`. The final VERIFIED evidence head passed CI `32262188186` and Fortress `32262188321`. The final synchronization head passed CI `32262420120` and Fortress `32262420198`, then PR #17 squash-merged that exact tested Phase 06 state to `main`.

## Explicit boundaries / deferred work

Phase 06 completion is scoped to the revised minimum safe-edit substrate needed for the M06→M09 vertical loop. It does **not** claim:

- `rustc`, Clang, or TypeScript-compiler type semantics;
- a true semantic call graph;
- an LSP semantic model;
- embedding/vector semantic search;
- Blueprint binary `.uasset` parsing or mutation;
- distributed workers, remote execution, or tool/process execution;
- repository merge, network, credential, publishing, signing, or release authority;
- hostile compare-and-swap/TOCTOU elimination beyond the verified Phase 03 Governor path/race guarantees.

The freshness check immediately precedes Governor entry, but non-cooperating external processes remain outside cooperative transaction-lock guarantees. Existing Phase 03 canonical path revalidation, audit, rollback, Windows replacement, and multi-file atomicity limitations remain authoritative.
