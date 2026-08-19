# Phase 05 Agent Orchestrator verification

Status: UNKNOWN — first implementation increment pending executable CI evidence.

Continuous build: not advanced from B000013 until validation passes.

## Increment acceptance gate

- [x] provider/engine-neutral `agent-orchestrator` workspace crate added;
- [x] typed agent, task, capability, state, and assignment contracts;
- [x] duplicate/empty task IDs fail closed;
- [x] unknown/self dependencies fail closed;
- [x] zero attempt budgets fail closed;
- [x] dependency cycles fail closed;
- [x] deterministic topological ordering;
- [x] dependency-ready scheduling only;
- [x] capability constraints are not relaxed;
- [x] declared agent concurrency is respected;
- [x] equivalent agent choices use stable agent ID order;
- [x] unit tests cover cycle rejection, dependency readiness, stable selection, capabilities, and capacity;
- [ ] `cargo fmt --check` passes on exact candidate head;
- [ ] strict workspace Clippy with warnings denied passes;
- [ ] full workspace tests pass;
- [ ] desktop frontend typecheck/build remains green;
- [ ] Windows Autonomy Governor regression gate remains green.

## Authority boundary

This increment is advisory scheduling only. It does not invoke a model, call a tool, access credentials/network resources, mutate a filesystem/repository, execute a game engine, merge, publish, or release. Model Router integration and Autonomy Governor-governed execution remain later Phase 05 increments.
