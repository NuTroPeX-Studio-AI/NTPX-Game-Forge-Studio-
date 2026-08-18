# Phase 03 — Build B000008 evidence

Status: VERIFIED candidate pending exact stamped-commit CI

Build: `NTPX-GFΩ-G0-v0.3.0-dev.3-M03-B000008`

## Scope

Request-bound human approval proofs and durable replay prevention for the Autonomy Governor, without adding governed external execution authority.

## Acceptance evidence

Implementation candidate commit `c15a6ad6a0823df54d9e3712c72289905e050e3f` passed GitHub Actions CI run `32175384163`:

- Rust formatting: PASS
- strict Clippy with warnings denied: PASS
- all workspace Rust tests: PASS
- frontend dependency/install regression gate: PASS
- TypeScript typecheck: PASS
- frontend production build: PASS

## Security acceptance

- HMAC-SHA256 proof authentication: PASS
- exact request digest binding: PASS
- authority/nonce/issue/expiry binding: PASS
- future and expired proof rejection: PASS
- malformed/tampered proof rejection: PASS
- missing proof rejection for human-gated requests: PASS
- proof cannot override policy denial: PASS
- approval evidence persisted in the verified audit chain: PASS
- consumed nonce replay rejected after audit reload: PASS
- approval secrets are not serialized or exposed by signer/verifier `Debug`: PASS by code contract
- minimum approval key length is 32 bytes: PASS by test

## Repair history

Validation failures were repaired without weakening any check:

1. applied rustfmt-prescribed layout changes;
2. removed an unnecessary HMAC construction panic path instead of documenting or allowing it;
3. replaced a post-Rust-1.80 API with an MSRV-compatible implementation;
4. corrected a test fixture whose byte-array annotation was one byte shorter than the literal.

## Boundary

This increment proves that an already human-gated authorization request has a valid, exact, unexpired, unused approval proof. The signer must be invoked only by an explicit human approval boundary; it does not itself prove human presence. Replay prevention is durable for serialized writers, not atomic across concurrent processes. Cross-process writer exclusion, transactional governed writes, rollback snapshots, and runtime enforcement remain pending Phase 03 work.

The exact final stamped commit must pass the same CI gate before merge. Phase 03 remains IN PROGRESS.
