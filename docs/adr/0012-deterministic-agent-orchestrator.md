# ADR 0012 — Deterministic Agent Orchestrator

## Status

Accepted for Phase 05.

## Context

Phase 04 can deterministically select a model but deliberately grants no execution authority. Phase 05 needs a project-level coordinator that can validate agent/task contracts, order dependency-aware work, bound retries and concurrency, collect workflow evidence, and hand governed capability requests to the verified Autonomy Governor without becoming a second authorization system.

## Decision

Add a provider-neutral `agent-orchestrator` Rust crate with these boundaries:

- `AgentContract` binds one stable orchestrator identity to the same `PrincipalId` used by the Autonomy Governor, plus role, declared skills, and an explicit parallel-task limit.
- `TaskContract` is immutable workflow input: objective, dependencies, required skills, assigned agent, routing request/policy, bounded attempts, and the exact governed requests the task may later ask the Governor to authorize.
- `TaskGraph` canonicalizes agent/task input into ordered maps, rejects duplicate identifiers, dangling dependencies, self-dependencies, skill mismatches, principal drift, and cycles, and emits a deterministic SHA-256 plan digest.
- Ready scheduling is dependency-aware and deterministic. Stable task identity is the ordering rule in this phase; agent concurrency is bounded by `max_parallel_tasks`.
- `dispatch_next` delegates model selection to the verified `ModelRouter`. A selected model is dispatch evidence only. Routing abstention escalates the task without consuming an execution attempt.
- Task attempts use explicit states and bounded retry. Retry exhaustion escalates; permanent failure fails; downstream tasks depending on failed/escalated/blocked work become blocked.
- Workflow evidence records dispatch, routing abstention, success, retry, failure, escalation, blocking, and successful Governor authorization handoffs.
- A governed request can be submitted only while its task is running and only if the request exactly matches an immutable predeclared request in that task contract. The orchestrator then calls `GovernanceRuntime::authorize`; it never manufactures a permit or bypasses human approval.
- The orchestrator performs no model invocation, tool execution, network access, credential use, repository mutation, engine action, publishing, or release operation.

## Consequences

The architecture now has three distinct control boundaries:

1. **Model Router** — chooses a suitable model identity.
2. **Agent Orchestrator** — chooses eligible workflow work and records state/evidence.
3. **Autonomy Governor** — decides whether a governed capability is authorized.

None of the first two imply the third. Execution adapters introduced in later phases must consume both orchestration context and a valid Governor permit where governed action is required.

## Residual limits

- Phase 05 workflow state/evidence is in-memory and serializable but not yet a crash-safe durable scheduler journal.
- Phase 05 schedules one next task per `dispatch_next` call; it enforces per-agent concurrency but does not yet provide a distributed worker queue, leases, heartbeats, cancellation propagation, or resource-aware global optimization.
- Evidence messages are workflow records, not cryptographically authenticated audit records. Governed authorization evidence remains the responsibility of the Autonomy Governor audit log.
- Task contracts predeclare exact governed requests. Dynamic capability expansion requires a new validated contract; agents cannot self-grant new scopes.
- A dispatch result does not invoke a model and an authorization permit does not itself execute the requested external operation.

## Verification

Phase 05 is complete only after the exact stamped M05/B000014 candidate passes repository CI: Rust formatting, strict Clippy, full workspace tests including orchestrator acceptance tests, frontend typecheck/build, and Windows Autonomy Governor regressions.