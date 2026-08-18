# Changelog

All notable verified changes to NTPX GameForge Ω∞ are recorded here.

## 0.4.0-dev.1 — Phase 04 — Build B000013

### Verified Model Router
- Added the provider-neutral `model-router` Rust workspace crate and a shared `ProviderAdapter` catalog contract.
- Added interchangeable validated local and cloud catalog adapters without introducing provider SDKs, model invocation, network calls, credentials, paid usage, tool execution, or repository/engine authority.
- Added typed capabilities, context/output capacity, local-only/cloud-allowed privacy, optional cost/latency/reliability constraints, and deterministic request-specific integer cost estimation.
- Added deterministic lexicographic routing policies with stable provider/model identity as the final tie-breaker, making equivalent decisions independent of adapter enumeration order.
- Added ordered bounded fallback candidates and explicit fail-closed abstention when no model satisfies all hard constraints.
- Added deterministic rejection evidence for missing capabilities, insufficient capacity, privacy violations, cost/latency ceilings, and reliability floors.
- Added fail-closed validation for duplicate model identities, adapter provider/class mismatches, invalid reliability values, and repeated/empty ranking policies.
- Added SHA-256 routing evidence covering request, policy, eligible/rejected candidates, and outcome.
- Added acceptance tests for adapter interchangeability, local-only privacy, deterministic ordering/digest, policy precedence, fallbacks, abstention, rejection evidence, duplicate identities, and invalid policy.
- Safely repaired CI findings from rustfmt and one strict Clippy documentation-markdown issue without weakening routing behavior or tests.
- Implementation candidate CI run `32196581575` passed Rust formatting, strict Clippy, all workspace tests including Model Router acceptance tests, desktop frontend typecheck/build, and the existing Windows Autonomy Governor regression gate.
- Recorded the architecture/security boundary in ADR 0011: a routing decision is selection evidence only and never grants network, credential, tool, repository, engine, publishing, or release authority.
- Phase 04 is VERIFIED pending exact stamped-head CI evidence. Phase 05 Agent Orchestrator is next.

## 0.3.0-dev.7 — Phase 03 — Build B000012

### Verified Autonomy Governor completion
- Added `GovernanceRuntime` as the audited in-process authorization boundary. Supplied operations execute only after durable allow evidence; denials are audited and human-gated operations require an exact one-time request-bound approval proof.
- Added canonical governed filesystem resources: absolute-path enforcement, parent-ancestor symlink rejection, symlink-target rejection, parent canonicalization, exact canonical scope matching, parent identity capture, and immediate pre-mutation identity revalidation.
- Added `GovernedFileBatch` for multi-file all-or-reverted behavior under ordinary failures, including duplicate canonical-target preflight and reverse-order rollback when a later member fails.
- Added conservative `RecoveryScanner` discovery and explicit recovery with SHA-256 snapshot verification, target binding, non-symlink ordinary-file checks, and canonical snapshot identity comparison for Windows short/long-path aliases.
- Centralized platform-aware replacement for commit and recovery. Unix-like targets use rename replacement; Windows uses direct rename when available and a reversible target-backup fallback otherwise. The fallback is failure-safe but is not claimed to be kernel-atomic across both renames.
- Added Windows CI for `autonomy-governor` check/tests in addition to Ubuntu formatting, strict Clippy, workspace tests, and frontend validation.
- Safely repaired CI findings from rustfmt, one Clippy documentation-markdown issue, and a Windows 8.3/long-path recovery identity mismatch without weakening security or tests.
- Recorded the Phase 03 runtime/security boundary in ADR 0010, including explicit residual limits: cooperative advisory locks, tamper-evident rather than authenticated hash state, no directory-handle pinning across mutation, non-kernel-atomic Windows fallback, and non-kernel-atomic multi-file batches.
- Phase 03 is VERIFIED. Phase 04 Model Router is next. No production credential, network, repository merge, engine execution, publishing, or release authority was introduced.

## 0.3.0-dev.6 — Phase 03 — Build B000011

### Verified approved governed file-transaction increment
- Added `ApprovalContext` and `GovernedFileTransaction::commit_with_approval` to bind request-bound human approval proofs to governed local file mutations.
- Require the underlying policy evaluation to already return `RequireHumanApproval`; approval proofs cannot override denial or manufacture authority for a direct-allow path.
- Verify proof authenticity, exact request binding, and lifetime before any filesystem mutation.
- Persist verified approval evidence and consume the one-time nonce through the same cross-process locked audit append used to record the successful transaction.
- Added automatic restoration from the durable pre-write snapshot when replay or audit failure is detected after replacement.
- Added a deterministic per-target advisory transaction lock so cooperating same-target writers serialize snapshot, staging, replacement, audit/nonce consumption, and rollback.
- Added tests for successful approved commit/evidence, replay rollback preserving the prior winner, invalid/expired proof no-mutation, and denial/direct-allow no-escalation.
- Safely repaired CI findings from rustfmt and strict Clippy by applying formatter output, removing an unused import, and replacing an over-wide public API with typed approval context; no checks were weakened.
- Implementation candidate CI run `32191076927` passed Rust formatting, strict Clippy, all workspace tests, frontend typecheck, and frontend production build.
- Phase 03 remains in progress; canonical path containment/parent-directory race hardening, crash recovery, multi-file transactions, stronger platform replacement semantics, and runtime enforcement remain pending.

## 0.3.0-dev.5 — Phase 03 — Build B000010

### Verified governed transactional file-write increment
- Added exact-path governed local file transactions inside `autonomy-governor`.
- Re-evaluate deny-by-default policy inside the transaction boundary and require direct `Allow` for exact `Filesystem` + `Write` scope matching.
- Added durable same-directory pre-write rollback snapshots with SHA-256 integrity digests.
- Added same-directory staging with durable flush before rename into place.
- Added verified audit append after mutation and automatic restoration of pre-write state if audit persistence fails.
- Added explicit transaction receipts and rollback support for both replaced and newly created files.
- Added symlink-target refusal and bounded canonical transaction-ID grammar.
- Added tests for existing-file rollback, new-file rollback, denial/no mutation, scope mismatch, corrupt-audit auto-rollback, and preservation of human-approval gates.
- Added ADR 0008 and verification evidence for the transaction/rollback boundary.
- Candidate CI run `32186220917` passed Rust formatting, strict Clippy, all workspace tests, frontend typecheck, and frontend production build.
- Phase 03 remains in progress; runtime enforcement and the remaining path-hardening/crash-recovery/multi-file work remain pending.

## 0.3.0-dev.4 — Phase 03 — Build B000009

### Verified cross-process audit writer exclusion increment
- Added OS-backed exclusive advisory locking on a deterministic sibling audit lock file for all audit mutations.
- Moved full-chain verification, next-sequence allocation, hash construction, append, and durable flush inside the same writer critical section.
- Added atomic approval-nonce consumption so nonce lookup and approved-record append cannot race across cooperating processes.
- Preserved fail-closed audit-chain integrity checks and the existing tamper-evident SHA-256 hash chain.
- Added a multi-process integration test with four independent writer processes appending eight records each to one shared log and verifying one valid 32-entry sequence/hash chain.
- Added ADR 0007 documenting the advisory-lock guarantee and its bypass boundary.
- Safely repaired CI findings from rustfmt and strict Clippy without suppressing checks; lock-file open mode is explicitly non-truncating.
- Verified Rust formatting, strict Clippy, workspace tests, frontend typecheck, and frontend production build in GitHub CI run `32180449557` for the implementation candidate.
- Phase 03 remains in progress; transactional governed writes, rollback snapshots, and runtime enforcement remain pending.

## 0.3.0-dev.3 — Phase 03 — Build B000008

### Verified request-bound approval proof increment
- Added HMAC-SHA256 human approval proofs bound to the exact request digest, authority identifier, one-time nonce, issue time, and expiry time.
- Added a 32-byte minimum approval-key policy with signer/verifier secret redaction from `Debug` output.
- Added fail-closed proof verification for mismatched requests, future/expired proofs, malformed tags, and authentication failures.
- Added an audited approval path that can satisfy only an existing `RequireHumanApproval` result; approval proofs cannot override a denial or create a grant.
- Added approval evidence to durable authorization audit records while preserving compatibility for records without approval evidence.
- Added verified-audit nonce lookup and durable replay rejection for serialized writers.
- Added tests for exact approval, persisted evidence, replay after reload, request binding, expiry/future proofs, tag tampering, denial preservation, missing proofs, and weak keys.
- Added ADR 0006 documenting the human-approval and secret-management boundary.
- Verified Rust formatting, strict Clippy, workspace tests, frontend typecheck, and frontend production build for the implementation candidate.
- Phase 03 remains in progress; cross-process writer exclusion, transactional governed writes, rollback snapshots, and runtime enforcement remain pending.

## 0.3.0-dev.2 — Phase 03 — Build B000007

### Verified durable audit log increment
- Added append-only JSONL persistence for Autonomy Governor decision records.
- Added SHA-256 hash chaining across audit entries with deterministic sequence and previous-hash linkage.
- Added fail-closed verification for malformed entries, sequence gaps, chain mismatches, and digest mismatches before any append.
- Added storage flush with `sync_data` after each append.
- Added tests for persistence, deterministic chaining, tamper detection, malformed records, corrupt-log append refusal, missing logs, and stored-decision preservation.
- Added an explicit security boundary: the hash chain is tamper-evident, not an authenticity mechanism against an attacker able to rewrite the complete log.
- Verified Rust formatting, strict Clippy, workspace tests, frontend typecheck, and frontend production build before final build stamping.
- Phase 03 remains in progress; approval-proof verification, replay prevention, transactional writes, rollback snapshots, and runtime enforcement remain pending.

## 0.3.0-dev.1 — Phase 03 — Build B000006

### Verified Autonomy Governor policy core
- Added a provider-neutral, engine-neutral `autonomy-governor` Rust crate.
- Added typed principals, permissions, exact resource scopes, risk levels, and approval requirements.
- Added deny-by-default deterministic policy evaluation.
- Added protected-action escalation so credential, merge, publish, destructive, and release capabilities always require human approval.
- Added deterministic audit-ready decision records.
- Added tests for default denial, exact-scope enforcement, grants, human gates, protected actions, and deterministic records.
- Verified Rust formatting, strict Clippy, workspace tests, frontend typecheck, and frontend production build.
- Phase 03 remains in progress; no tool execution or external mutation authority is introduced by this increment.

## 0.2.0-dev.2 — Phase 02 — Build B000005

### Verified GameSpec foundation
- Completed the engine-neutral GameSpec 0.1.0 typed domain contract.
- Added canonical `SpecId` validation and fixed-algorithm deterministic generated IDs.
- Added stable JSON serialization/deserialization and semantic validation.
- Added deterministic direct-reference dependency graph extraction and dependency cycle rejection.
- Added the checked-in versioned JSON Schema and CLI schema emission.
- Added migration from the verified legacy `0.1` shape to `0.1.0`.
- Added deterministic compilation manifests.
- Added `gamespec-compiler` CLI commands for validate, inspect, graph, compile, migrate, normalize, and schema.
- Added Signal Born full reference and legacy migration fixtures.
- Added asset licensing and provenance metadata.
- Added regression, migration, graph, compiler, deterministic-ID, and CLI tests.
- Verified Rust formatting, strict Clippy, workspace tests, frontend typecheck, and frontend production build.

### Boundary
- Phase 03 Autonomy Governor is the next milestone.
- No autonomous tool authority, engine adapter, model router, or release authority is introduced by Phase 02.

## 0.2.0-dev.1 — Phase 02 — Build B000003
- Added the first verified engine-neutral GameSpec core increment.
- Preserved Phase 02 as in progress pending schema, graph, migration, compiler, and full domain completion.

## 0.1.0-dev.1 — Phase 01 — Build B000002
- Established the Rust workspace and `gameforge-core`.
- Added Tauri + React + TypeScript desktop shell.
- Added CI, governance docs, ADRs, and build metadata.
- Verified the repository foundation.
