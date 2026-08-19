# ADR 0013 — Bounded Agent Task State Machine

Status: Accepted for Phase 05 implementation increment.

## Context

The verified Phase 05 substrate in Build B000014 declared `TaskSpec::max_attempts`, but the original pending/running/terminal state model did not retain consumed attempts once a failed task became eligible to run again. Retrying without retained attempt state would make the declared retry budget unenforceable and could permit unbounded advisory rescheduling.

## Decision

Extend the provider-neutral, engine-neutral `agent-orchestrator` state contract with `RetryPending { attempts }` and explicit `TaskEvent` inputs for start, success, failure, and escalation.

A task starts at attempt 1. A failed running attempt transitions to `RetryPending` only while `attempt < max_attempts`; the next start advances to `attempt + 1`. Failure at the exact configured maximum transitions to terminal `Failed`. Terminal success, failure, and escalation states cannot be restarted through the transition API.

Externally supplied retry/running/failed counters are checked against the task's declared retry budget before scheduling or transition. Inconsistent counters fail closed. Retry-pending work is schedulable only under the same dependency, capability, deterministic ordering, and concurrency constraints as first-attempt work.

Escalation is an explicit terminal orchestration state and requires a non-empty reason. It is evidence of orchestration state only; it does not grant permission, human approval, tool authority, model invocation, credentials, network access, filesystem/repository mutation, engine execution, publishing, merge, or release authority.

## Consequences

- retry budgets are represented explicitly and deterministically instead of being inferred from transient execution state;
- exhausted work cannot be silently rescheduled by the advisory scheduler;
- malformed retry counters fail closed;
- escalation is explicit and auditable by future evidence layers but is not itself an authorization mechanism;
- delegation, deterministic orchestration evidence collection, Model Router selection integration, and Autonomy Governor-governed action execution remain unfinished Phase 05 work.
