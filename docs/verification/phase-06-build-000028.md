# Phase 06 — Build B000028 verification

Status: CANDIDATE

Build: `NTPX-GFΩ-G0-v0.6.0-dev.14-M06-B000028`

## Acceptance gate

- [x] B000015 deterministic provider-neutral repository index foundation;
- [x] stable source and symbol SHA-256 evidence;
- [x] deterministic lexical reference resolution with explicit ambiguous/unresolved states;
- [x] deterministic advisory dependency/change-impact evidence;
- [x] fail-closed incremental index reuse;
- [x] deterministic advisory diagnostics and symbol search;
- [x] provider-neutral syntax-service boundary;
- [x] `syn`-backed full-file Rust syntax parsing;
- [x] Tree-sitter TypeScript full-file syntax parsing;
- [x] Tree-sitter C++ top-level syntax parsing;
- [x] deterministic textual Blueprint metadata parsing;
- [x] B000027 evidence-bound exact whole-file edit planning;
- [x] generated/no-op/stale plan rejection;
- [x] replacement-content and full-plan integrity rechecked before application;
- [x] repository-relative normal path enforcement with traversal/absolute/prefix rejection;
- [x] current on-disk source re-read and freshness-checked immediately before the governed path;
- [x] source mutation routed only through `GovernanceRuntime` reversible file transactions;
- [x] human approval gates preserved and approved path exercised;
- [x] transaction receipt exposes explicit rollback and durable authorization evidence;
- [x] implementation Rust formatting passes;
- [x] implementation strict Clippy passes with warnings denied;
- [x] implementation full workspace tests pass;
- [x] implementation desktop TypeScript typecheck/build passes;
- [x] implementation Windows Autonomy Governor tests pass;
- [x] implementation NTPX Fortress bootstrap gate passes;
- [ ] exact B000028 stamped candidate head passes the complete CI gate;
- [ ] exact B000028 stamped candidate head passes Fortress;
- [ ] final VERIFIED evidence head passes both gates;
- [ ] PR #17 merges to `main` and default-branch readback confirms Phase 06 VERIFIED / Phase 07 NEXT.

## Implementation evidence

CI `32260168938` stopped the Rust job at rustfmt-only differences in `code-intelligence-apply`; frontend passed and no behavior/lint gate was suppressed. Exact formatter-equivalent changes were applied.

Repaired implementation head `d63050c8184521e6408b6bb9923af4da327f0ebf` passed complete CI `32260654093`: Rust formatting, strict Clippy with warnings denied, full workspace tests including governed-edit application/rollback tests, desktop frontend typecheck/build, and Windows Autonomy Governor tests. Fortress gate `32260654100` also passed.

## Claim boundary

Phase 06 completion under the revised safe-edit threshold means deterministic multi-language syntax/index evidence plus exact evidence-bound whole-file edit planning and Autonomy-Governor-bound reversible application. It does **not** claim rustc/Clang/TypeScript compiler type semantics, true semantic call graphs, an LSP semantic model, embedding search, Blueprint binary `.uasset` parsing, distributed execution, or hostile TOCTOU elimination beyond the existing Governor guarantees.
