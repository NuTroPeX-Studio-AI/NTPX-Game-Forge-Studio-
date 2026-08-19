# Phase 06 Verification — Build B000019

Build ID: `NTPX-GFΩ-G0-v0.6.0-dev.5-M06-B000019`
Outcome: VERIFIED
Phase state: IN PROGRESS
Release: false

## Verified increment

Deterministic advisory lexical diagnostics over the existing verified Phase 06 `ReferenceGraph` evidence.

Verified behaviors:

- uniquely resolved references emit no diagnostic;
- unresolved lexical references emit stable `NTPX-CI-UNRESOLVED-REFERENCE` warnings;
- ambiguous lexical references emit stable `NTPX-CI-AMBIGUOUS-REFERENCE` warnings;
- ambiguous candidate symbol IDs are sorted and deduplicated before evidence construction;
- diagnostic records are canonically ordered;
- unresolved/ambiguous counts are explicit;
- equivalent reference enumeration produces identical report ordering and SHA-256 evidence;
- the diagnostics layer is provider-neutral, engine-neutral, advisory, and read-only.

## CI evidence

Initial candidate CI `32226203012` failed only the Rust formatting gate. The CI log showed rustfmt-only output differences in `crates/code-intelligence-diagnostics/src/lib.rs`. The formatter-equivalent changes were applied exactly; no lint, test, validation, or security requirement was relaxed.

Repaired implementation candidate head: `8285b83d8053ddfabbe9f5a2341c6fb30a466da4`
CI run: `32226278375`
Conclusion: success

Passed:

- `cargo fmt --check`
- strict workspace Clippy with warnings denied
- complete Rust workspace test suite, including new diagnostics tests
- desktop TypeScript typecheck
- desktop production build
- Windows Autonomy Governor check
- Windows Autonomy Governor tests

## Classification boundary

VERIFIED applies to the deterministic lexical diagnostics increment and its repository gates. It does not classify the whole of M06 as complete.

Not claimed: compiler/LSP semantic diagnostics, AST-native parsing, package/build-system resolution, true call graphs, semantic search, refactor application, persistent cache performance, source mutation, engine execution, publishing, signing, or release authority.
