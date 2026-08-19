# ADR 0025 — Evidence-Bound Exact Edit Planning

Status: Accepted
Phase: 06 Code Intelligence
Build: B000027

## Context

The revised Phase 06 completion threshold requires a deterministic edit-planning substrate before any source mutation is connected to the Autonomy Governor. Planning must fail closed when the indexed repository or target source changes between analysis and a later write.

## Decision

Add a dedicated provider-neutral `code-intelligence-edits` crate above the verified read-only Code Intelligence index.

An `ExactEditPlan` represents one whole-file replacement and binds the proposal to:

- exact source path and language;
- the indexed current-content SHA-256 digest;
- the proposed replacement-content SHA-256 digest;
- the complete replacement content;
- the repository-index evidence SHA-256 digest; and
- a deterministic plan evidence SHA-256 digest.

Planning requires an exact match between supplied source state and indexed language, generated-file state, and content digest. Generated sources and exact no-op replacements are rejected.

`verify_edit_plan_fresh` performs a second read-only check immediately before a future mutation boundary. Repository evidence drift, path/language mismatch, generated source state, or source-content drift fails closed.

## Authority boundary

An edit plan is evidence only. It is not a filesystem write permit, Autonomy Governor permit, repository operation, patch application, tool invocation, engine action, credential, merge, publish, signing, or release capability.

B000027 deliberately performs no source mutation. The following Phase 06 increment must bind a fresh verified edit plan to the existing `GovernanceRuntime` and reversible transaction primitives rather than creating another write path.

## Consequences

- Planning becomes deterministic and independently testable.
- Stale source and stale repository snapshots are rejected before mutation authority is considered.
- Generated files remain protected until a later explicit policy exists.
- Whole-file replacement is intentionally narrower than semantic refactoring; richer edit primitives can be added later without weakening this evidence contract.
