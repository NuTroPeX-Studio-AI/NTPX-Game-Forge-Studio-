# ADR 0014 — Delegation-bound orchestration evidence

Status: Accepted for Phase 05 Build B000016.

## Context

The Phase 05 scheduler already emits deterministic advisory assignments and the task state machine already bounds retries and escalation. The next orchestration increment requires explicit delegation and reproducible evidence without allowing delegation to become a second scheduling or authorization path.

## Decision

Add a provider-neutral, engine-neutral `agent-orchestrator-evidence` workspace crate that consumes the existing orchestrator's task IDs, agent IDs, task states, and advisory assignments.

A `DelegationDirective` records the delegator, delegate, task, and non-empty reason. A delegation is accepted only when the delegated-to agent exactly matches the scheduler assignment for that task. Unknown tasks, duplicate task IDs, missing/unknown task state, empty agent IDs, duplicate assignments, self-delegation, empty reasons, and unbound delegation fail closed.

Evidence canonicalizes assignment and delegation ordering, preserves the supplied deterministic task order, records task state, assignment, and delegation data in one versioned payload, serializes it deterministically, and emits a SHA-256 correlation digest.

## Authority boundary

Delegation and its digest are advisory evidence only. Neither is an authorization token, human approval proof, model invocation permit, tool capability, network credential, repository mutation permission, engine permission, publishing permission, or release permission. Future executable actions must still pass through the verified Autonomy Governor. Future model selection remains a separate Model Router decision.

## Consequences

- Equivalent orchestration inputs produce equivalent evidence independent of assignment/delegation vector enumeration order.
- A delegation cannot silently replace the scheduler's selected agent.
- The evidence digest provides reproducible correlation, not secret-key authenticity.
- The implementation adds no provider SDK, engine dependency, network call, credential handling, or execution adapter.
- Model Router integration and Autonomy Governor-governed execution remain separate later Phase 05 increments.
