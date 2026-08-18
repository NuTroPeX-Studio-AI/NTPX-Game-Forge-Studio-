# Phase 04 Model Router verification

Status: CANDIDATE — exact stamped-head CI required before merge.

Candidate build: `NTPX-GFΩ-G0-v0.4.0-dev.1-M04-B000013`

## Acceptance gate

- [x] provider-neutral `model-router` Rust workspace crate;
- [x] common `ProviderAdapter` contract with interchangeable local and cloud catalog adapters;
- [x] typed model capability descriptors;
- [x] context-window and output-capacity constraints;
- [x] local-only versus cloud-allowed privacy constraints;
- [x] deterministic integer cost estimation and optional cost ceiling;
- [x] optional latency ceiling;
- [x] optional reliability floor;
- [x] deterministic lexicographic ranking policy;
- [x] stable provider/model identity final tie-breaker;
- [x] adapter enumeration order does not change the routing decision or evidence digest;
- [x] explicit ordered bounded fallbacks;
- [x] explicit fail-closed abstention when no candidate qualifies;
- [x] rejected candidates retain deterministic exclusion reasons;
- [x] duplicate model identities fail closed;
- [x] adapter provider/class inconsistencies fail closed;
- [x] malformed/repeated routing policy dimensions fail closed;
- [x] routing evidence is SHA-256 digested;
- [x] routing is selection-only and exposes no model invocation, network, credential, filesystem, repository, engine, merge, publishing, or release authority;
- [x] Ubuntu workspace formatting passes;
- [x] Ubuntu strict Clippy passes with warnings denied;
- [x] Ubuntu workspace tests, including Model Router acceptance tests, pass;
- [x] desktop frontend typecheck and production build pass;
- [x] existing Windows Autonomy Governor check/tests remain green.

## Evidence lineage

- CI run `32196317963` identified only rustfmt differences in the initial implementation candidate; formatter output was applied without changing routing policy or tests.
- CI run `32196458325` passed formatting and identified one Clippy documentation-markdown issue; no behavior or test was weakened.
- CI run `32196581575` passed the implementation candidate: Rust formatting, strict Clippy, full workspace tests including Model Router tests, desktop frontend typecheck/build, and the Windows Autonomy Governor regression gate.
- The final B000013 documentation/build/UI stamp must pass the same CI gate on the exact branch head before this record changes from CANDIDATE to VERIFIED and before PR #13 is merged.

## Explicit boundaries

- Catalog capability, cost, latency, and reliability values are routing inputs, not independently verified provider truth.
- SHA-256 routing evidence is deterministic/tamper-evident correlation evidence, not a secret-key authenticity proof.
- Choosing a cloud model does not authorize a network connection or credential use.
- Choosing a model with tool-use capability does not authorize tool execution.
- A routing decision does not override Autonomy Governor policy or human approval gates.
- Phase 04 introduces no real provider API calls, credentials, paid usage, model inference, tool execution, repository mutation authority, engine execution, publishing, or release authority.

## Next phase

Phase 05 — Agent Orchestrator. It may consume Model Router decisions for model selection, but every governed action must continue through the verified Autonomy Governor boundary.
