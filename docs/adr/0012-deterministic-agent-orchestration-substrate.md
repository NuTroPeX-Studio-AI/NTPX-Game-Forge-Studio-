# ADR 0012 — Deterministic Agent Orchestration Substrate

Status: Accepted for Phase 05 implementation increment; executable verification pending CI.

## Context

Phase 05 requires typed agent/task contracts, dependency-aware task graphs, deterministic scheduling, delegation, bounded retries, evidence, state transitions, escalation, Model Router integration, and governed execution through the Autonomy Governor. Building invocation/execution before a validated scheduling substrate would mix planning authority with execution authority and create a risk of parallel permission paths.

## Decision

Introduce an engine-neutral and provider-neutral `agent-orchestrator` crate whose first increment is advisory only. It owns typed `AgentDescriptor`, `TaskSpec`, `TaskState`, dependency-graph validation, deterministic topological ordering, and deterministic capacity/capability-aware assignment planning.

The graph fails closed on duplicate/empty task IDs, unknown/self dependencies, zero attempt budgets, and dependency cycles. Scheduling never silently relaxes dependency, capability, or concurrency constraints. Stable IDs determine equivalent scheduling choices.

This increment exposes no model invocation, tool execution, network access, credentials, filesystem/repository mutation, engine execution, publishing, merge, or release API. Later Phase 05 execution adapters must consume the verified Autonomy Governor; later model selection must consume the verified Model Router without treating a routing decision as authorization.

## Consequences

- orchestration planning is deterministic and testable before live execution is introduced;
- scheduling evidence can later be layered on a stable task/agent contract;
- bounded retry transitions, delegation/evidence semantics, Model Router integration, and governed execution remain unfinished Phase 05 work;
- no Phase 05 completion claim is made by this increment alone.
