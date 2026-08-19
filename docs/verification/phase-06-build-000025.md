# Phase 06 — Build B000025 Verification

Build ID: `NTPX-GFΩ-G0-v0.6.0-dev.11-M06-B000025`
Classification: VERIFIED implementation candidate
Phase state: IN PROGRESS
Release: false

## Verified increment

B000025 upgrades the TypeScript parser adapter to full-file AST-grade syntax parsing using Tree-sitter behind the existing provider-neutral `code-intelligence-syntax` boundary.

Top-level declaration evidence is derived from parsed syntax nodes for supported functions, classes, interfaces, type aliases, enums, namespaces, and variable declarators. Evidence includes stable kind, name, parser-derived source line, ordinal, and deterministic SHA-256 output. Parser errors fail closed; nested declarations are not promoted.

Unit coverage includes deterministic repeated parsing, multiline/generic declarations, nested-declaration exclusion, regular-expression and template-expression grammar handling, multiple variable declarators, language mismatch, and malformed TypeScript failure.

## Executed validation

Initial candidate `1f0fd170fd692b1e56206e9a4fa63fa78259fa8b`, CI `32251996573`:

- desktop TypeScript typecheck/build — PASSED
- `cargo fmt --check` — FAILED only on formatter layout drift

Formatter-equivalent changes were applied without weakening any gate.

Second candidate `16b8a25b106034a31bb07721859aa74eb70f9dfd`, CI `32252151281`:

- `cargo fmt --check` — PASSED
- compile/Clippy — FAILED on a borrow-lifetime error in Tree-sitter error-tree traversal
- desktop TypeScript typecheck/build — PASSED
- Windows Governor check passed while the Rust job exposed the compile defect

The cursor lifetime was repaired structurally by binding the iterator result before the local cursor was dropped.

Repaired implementation head `807778bf0fcf9c756f7cf0c58c5eac5f571c152a`, CI `32252319138`:

- `cargo fmt --check` — PASSED
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — PASSED
- `cargo test --workspace` — PASSED
- desktop TypeScript typecheck — PASSED
- desktop production build — PASSED
- Windows `autonomy-governor` check — PASSED
- Windows `autonomy-governor` tests — PASSED

No dedicated game-engine runtime or performance benchmark is applicable to this parser-only increment.

## Dependency evidence

B000025 adds Tree-sitter and the TypeScript grammar only to the syntax-adapter implementation boundary. No model-provider or game-engine dependency is introduced into shared core authority layers.

## Claim boundary

This verifies parser-derived TypeScript syntax evidence, not TypeScript compiler binding, module resolution, type checking, compiler/LSP semantics, true call graphs, refactor execution, or source-write authority.

No protected merge, release publication, signing, production credential use, destructive action, model/tool/network execution, or engine action occurred.

## Final-head rule

The synchronized B000025 build metadata, roadmap, architecture/ADR records, changelog/build history, and this verification record must pass the complete repository CI gate before the exact stamped head is classified VERIFIED.
