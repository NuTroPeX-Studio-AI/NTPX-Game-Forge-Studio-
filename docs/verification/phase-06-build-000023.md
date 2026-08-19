# Phase 06 — Build B000023 Verification

Build ID: `NTPX-GFΩ-G0-v0.6.0-dev.9-M06-B000023`
Classification: VERIFIED implementation candidate
Phase state: IN PROGRESS
Release: false

## Verified increment

B000023 replaces the B000022 line-prefix/brace-counting Rust adapter internals with full-file `syn` parsing while preserving the existing provider-neutral `code-intelligence-syntax` API boundary.

The adapter now derives top-level evidence from a parsed Rust syntax tree for functions, structs, enums, traits, modules, impl blocks, type aliases, constants, and statics. Evidence retains stable kind/name/line/ordinal fields and deterministic SHA-256 correlation output. Parser span locations provide source-line evidence.

Unit coverage verifies deterministic repeated parsing, nested-method exclusion, braces inside comments/strings, multiline generic function declarations, language mismatch, malformed Rust fail-closed behavior, and ignoring unsupported top-level items only after successful full-file parsing.

## Executed validation

Initial implementation head `de489a03ee19a411333bd59cb68038e6360c3f72` ran in GitHub Actions CI `32245900914`:

- desktop TypeScript typecheck/build — PASSED
- Rust formatting — FAILED due only to rustfmt layout drift in the new adapter
- later Rust steps were skipped by the workflow after the format failure

The exact formatter-required change was applied with no lint suppression or validation/security relaxation.

Repaired implementation head `748c3045e76f681e752d7d5e152a7ef5e4364ada` passed GitHub Actions CI `32245983720`:

- `cargo fmt --check` — PASSED
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — PASSED
- `cargo test --workspace` — PASSED
- desktop TypeScript typecheck — PASSED
- desktop production build — PASSED
- Windows `autonomy-governor` check — PASSED
- Windows `autonomy-governor` tests — PASSED

No dedicated game-engine runtime or performance benchmark job exists in the current CI workflow, so B000023 makes no engine-runtime or performance claim. The existing Windows Governor regression gate provides the applicable security regression evidence for this increment.

## Dependency evidence

B000023 adds workspace dependencies on `syn` with the `full` feature and `proc-macro2` with `span-locations`. The repository does not currently commit a root `Cargo.lock`; exact transitive dependency pinning therefore remains outside the existing repository policy and is not claimed by this build.

## Claim boundary

B000023 verifies AST-grade Rust syntax parsing through `syn`, not `rustc` compilation or semantic analysis. It does not establish macro expansion, cfg evaluation, module loading, name/type resolution, borrow checking, control/data-flow analysis, compiler/LSP semantic resolution, true call graphs, embedding semantic search, refactor execution, or source mutation authority.

No protected merge, release publication, signing, production credentials, destructive operation, model/tool/network execution, or engine action occurred.

## Final-head rule

The synchronized B000023 build metadata, roadmap, ADR, changelog/build history, and this verification record must pass the complete repository CI gate on the final branch head before that exact head is classified VERIFIED.
