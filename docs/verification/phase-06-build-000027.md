# Phase 06 — Build B000027 Verification

Build ID: `NTPX-GFΩ-G0-v0.6.0-dev.13-M06-B000027`
Classification: VERIFIED implementation candidate
Phase state: IN PROGRESS
Release: false

## Verified increment

B000027 adds the dedicated provider-neutral `code-intelligence-edits` crate for deterministic whole-file replacement planning bound to exact repository and source evidence.

The planner records exact path/language, expected source SHA-256, replacement SHA-256 and content, repository-index evidence SHA-256, and deterministic plan evidence SHA-256. It rejects generated sources, stale indexed source, repository-evidence drift during freshness checks, post-plan source drift, path/language mismatch, and exact no-op replacements.

## Validation lineage

Initial candidate head `1326f6c4b7e40f83e0b412a9b648f08412cc9134`, CI `32256232976`:
- desktop TypeScript typecheck/build — PASSED
- Rust formatting — FAILED only on rustfmt layout drift

Formatter-equivalent changes were applied without weakening validation, linting, tests, or security controls.

Repaired implementation head `ebdb89eab9916b21c16e794cd0dcb848037ccd27`, CI `32256357339`:
- `cargo fmt --check` — PASSED
- strict workspace Clippy with `-D warnings` — PASSED
- complete Rust workspace tests — PASSED
- desktop TypeScript typecheck — PASSED
- desktop production build — PASSED
- Windows Autonomy Governor check — PASSED
- Windows Autonomy Governor tests — PASSED

No Unreal runtime, engine integration, network, credential, or performance benchmark is applicable to this read-only planning increment.

## Claim boundary

B000027 verifies deterministic edit-plan construction and stale-evidence rejection only. It does not write files, apply patches, execute tools, invoke models, run engines, merge repositories, publish, sign, or release.

The next Phase 06 increment must connect only fresh plans to the verified Autonomy Governor reversible mutation path.

## Final-head rule

The synchronized B000027 metadata/ADR/changelog/history/evidence head must pass the complete repository CI gate before the exact stamped head is classified VERIFIED.
