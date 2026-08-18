# Changelog

All notable verified changes to NTPX GameForge Ω∞ are recorded here.

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
