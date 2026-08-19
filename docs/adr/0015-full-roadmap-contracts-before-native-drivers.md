# ADR 0015 — Full-roadmap contracts before native drivers

## Status

Accepted for implementation candidate; exact combined head verification pending.

## Context

Phases 06–18 require code intelligence, process/tool execution, Unreal integration, build/test/repair, memory, remote transport, project state modeling, evaluation, content production, controlled improvement, a playable Signal Born slice, and cross-platform release machinery. Implementing those directly as provider-, engine-, or platform-specific runtime code would couple shared orchestration to privileged external systems before their authority and evidence boundaries were stable.

## Decision

1. Maintain `orchestration/roadmap.manifest.json` as a machine-checkable completeness inventory for all 18 roadmap phases and required components.
2. Validate that inventory in the Rust workspace with `roadmap-contract`.
3. Introduce small phase-specific contract/substrate crates before concrete privileged drivers.
4. Keep shared core provider-neutral and engine-neutral. Unreal-specific behavior lives in `unreal-engine-bridge`; platform packaging lives in `release-pipeline`.
5. Require Agent Zero execution dispatch to consume exact Autonomy Governor authorization evidence before a concrete execution adapter may run.
6. Treat model routing, escalation, evaluation, world-model state, asset jobs, narrative graphs, improvement proposals, slice readiness, and release eligibility as evidence/decision boundaries rather than implicit execution authority.
7. Keep protected promotion, signing, publication, policy change, and production release human-gated.
8. Do not mark later phases complete merely because their contracts compile. Native/runtime drivers, platform dependencies, tests, and verification evidence remain separate acceptance work.

## Consequences

The repository can construct later phases in parallel without losing component inventory or collapsing privilege boundaries. Native/runtime implementations can replace or extend adapters incrementally while preserving deterministic shared contracts. The cost is that a contract/substrate candidate is intentionally not equivalent to a finished phase; roadmap status must continue to distinguish candidate construction from verified runtime completion.
