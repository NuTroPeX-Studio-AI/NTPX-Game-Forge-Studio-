# ADR 0011 — Provider-neutral deterministic Model Router

Status: Accepted

## Context

Phase 04 needs a model-selection layer that can compare local and cloud model catalogs without binding shared GameForge architecture to one AI vendor. Model selection must remain separate from model invocation and from the Autonomy Governor's execution authority.

## Decision

Introduce the `model-router` Rust crate as a provider-neutral, selection-only boundary.

- Provider adapters expose read-only provider identity, locality classification, and model descriptors.
- Model descriptors declare typed capabilities, context/output capacity, expected latency, token-price metadata, and reliability metadata.
- Routing requests declare required capabilities/capacity plus privacy, cost, latency, and reliability constraints. Prompt or project payload content is not part of the routing contract.
- Hard constraints are evaluated before ranking. Ineligible models remain in deterministic rejection evidence with explicit reasons.
- Eligible models are ranked lexicographically by an ordered `RoutingPolicy`; stable provider/model identity is the final tie-breaker.
- The selected primary and bounded fallback list are therefore reproducible for the same request, policy, and catalog, independent of adapter enumeration order.
- When no model satisfies all constraints, abstention is a valid fail-closed outcome.
- Routing evidence is serialized deterministically and SHA-256 digested for reproducibility and later audit correlation.
- Local and cloud catalog adapters implement the same `ProviderAdapter` interface and validate that exposed descriptors match their provider identity and locality class.

## Authority boundary

The Model Router does not:

- invoke models;
- open network connections;
- hold or use provider credentials;
- execute tools;
- mutate files or repositories;
- merge branches;
- control an engine; or
- publish/release anything.

A routing decision is selection evidence, not authorization. Any later adapter that performs a governed action must separately obtain authorization through the verified Autonomy Governor. Protected actions remain human-gated even when the selected model recommends them.

## Determinism and metadata limits

The router uses integer token/cost/reliability values and ordered comparisons rather than floating-point scoring. Adapter/model enumeration is normalized before evaluation, and identity is the final tie-breaker.

Cost, latency, reliability, and capability declarations are catalog inputs. The router validates their shape and consistency but does not independently prove that a provider's declared metadata is current or empirically correct. Future telemetry may update catalog inputs only through an explicit provenance/versioning contract; it must not silently rewrite historical routing evidence.

## Consequences

- Phase 05 Agent Orchestrator can request model selection without becoming provider-specific.
- Local-only privacy can fail closed before any future cloud invocation layer is reached.
- Fallback ordering is explicit instead of being hidden inside provider-specific retry code.
- Model selection remains reproducible and inspectable while execution authority stays centralized in the Autonomy Governor.
