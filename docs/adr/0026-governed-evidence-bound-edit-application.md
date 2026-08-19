# ADR 0026 — Governed evidence-bound edit application

Status: Accepted.

## Context

B000027 produced deterministic exact whole-file edit plans bound to repository evidence and the indexed source digest, but deliberately stopped before mutation. Phase 06 cannot be considered complete under the revised M06→M09 safe-edit threshold until a fresh plan can enter the already verified Autonomy Governor reversible-file transaction path without creating a second authorization or mutation system.

## Decision

Introduce `code-intelligence-apply` as a narrow application boundary. `code-intelligence-edits` remains pure/read-only planning. Before the Governor is entered, application:

1. recomputes the replacement-content SHA-256 and the full plan evidence SHA-256;
2. requires an existing canonical repository root;
3. accepts only non-empty repository-relative paths composed of normal path components;
4. resolves the target through the Governor filesystem-target resolver, inheriting symlink/path checks;
5. re-reads the current on-disk source and re-runs B000027 repository/source freshness validation;
6. submits the exact write request to `GovernanceRuntime::write_file` or `write_file_with_approval`;
7. returns the existing Autonomy Governor `TransactionReceipt`, including rollback and durable audit evidence.

A plan never grants authority. Direct writes still require policy `Allow`; human-gated writes still require an exact request-bound approval proof. Plan freshness or integrity cannot override denial, invent a capability, consume credentials, merge a repository, invoke a model/tool/engine, publish, sign, or release.

## Fail-closed behavior

Application rejects:

- tampered replacement bytes or plan evidence;
- repository evidence drift;
- current source digest drift;
- generated-source plans inherited from the planner;
- absolute, parent-traversal, current-directory, prefix, or otherwise non-canonical repository-relative plan paths;
- symlink/path violations rejected by the Governor;
- denied or unsatisfied human-approval requests;
- Governor audit, transaction, or rollback failures.

## Verification

B000028 acceptance includes executable coverage for successful governed mutation plus explicit rollback, stale on-disk source rejection before mutation, replacement-evidence tamper rejection, repository escape rejection even with recomputed plan correlation evidence, and preservation of a human approval gate with a successful approved path.

The exact B000028 stamped candidate passed repository CI `32261483993` and NTPX Fortress `32261483663`. The repository CI gate covers Rust formatting, strict Clippy with warnings denied, full workspace tests, desktop typecheck/build, and Windows Autonomy Governor tests.

## Residual limits

The preflight freshness check occurs immediately before entering `GovernanceRuntime`, but Phase 06 does not claim hostile compare-and-swap semantics against a non-cooperating process that races the file between those steps. The existing Phase 03 canonical path revalidation, cooperative advisory locking, platform replacement, audit, snapshot, and rollback guarantees and their documented TOCTOU/atomicity limits remain authoritative.

This boundary is whole-file exact replacement. Fine-grained semantic refactors, compiler/type semantics, true semantic call graphs, LSP semantics, embedding search, distributed execution, and Blueprint binary `.uasset` mutation are not claimed by Phase 06.
