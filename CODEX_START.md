# Codex Start — NTPX GameForge Studio Lab

## Source of truth

This repository is the canonical NTPX GameForge Studio Lab implementation. Before making architectural or behavioral changes, read:

1. `AGENTS.md`
2. `ARCHITECTURE.md`
3. `SECURITY.md`
4. `ROADMAP.md`
5. `PROJECT_BRIEF_GAMEFORGE_STUDIO_LAB.md`
6. `docs/reference/GAME_DEVELOPER_ECOSYSTEM.md`
7. `data/registry/game-development-tools.json`

## Current verified state

Do **not** restart from an earlier G000/G001 concept scaffold. The repository roadmap records Phases 01–06 as VERIFIED.

Current target:

**Phase 07 — Agent Zero Execution**

The implementation must preserve all verified boundaries from prior phases:

- GameSpec remains the canonical engine-neutral machine-readable source of reproducible game intent and state.
- Model Router selection is evidence, not execution authority.
- Agent Orchestrator scheduling is not authorization.
- Code Intelligence edit plans are evidence, not permission.
- `GovernanceRuntime` remains the sole verified authorization boundary for governed mutation/execution.
- Human-gated actions remain human-gated.
- NTPX Fortress remains a carried-forward repository gate.

## Phase 07 objective

Implement isolated execution workers that can execute explicitly declared process/tool work under governance without bypassing the Autonomy Governor, Agent Orchestrator, Code Intelligence, or Fortress controls.

### Required capabilities

1. **Execution worker contract**
   - immutable worker identity
   - immutable task/execution request identity
   - declared executable/tool identity
   - exact arguments
   - declared working directory
   - declared environment-variable allowlist
   - declared filesystem scopes
   - declared network policy
   - declared timeout/resource limits
   - expected artifact/evidence contract

2. **Governed execution boundary**
   - every governed process/tool execution passes through the existing Governor boundary
   - no worker-local privilege expansion
   - no hidden fallback to unrestricted shell execution
   - human approval remains request-bound where required

3. **Isolation**
   - explicit child-process lifecycle
   - cancellation
   - timeout enforcement
   - bounded stdout/stderr capture
   - exit-code/signal evidence
   - deterministic working-directory handling
   - environment filtering
   - resource-limit abstraction
   - sandbox backend abstraction for later platform-specific hardening

4. **Evidence**
   - execution request digest
   - authorization decision evidence
   - worker/run identity
   - process start/end timestamps
   - normalized command metadata
   - exit status
   - bounded logs
   - produced-artifact metadata/digests
   - cancellation/timeout/resource-limit evidence

5. **Orchestrator integration**
   - dispatched task contracts map to exact execution requests
   - execution cannot expand the task contract
   - retry behavior remains bounded by the validated task contract
   - failures escalate rather than silently broadening permissions

6. **Code Intelligence integration**
   - source changes continue through the existing evidence-bound planning/application path
   - Phase 07 must not introduce direct raw source mutation

7. **Testing**
   - successful allowed execution
   - Governor denial
   - human-approval-required flow
   - invalid executable/tool contract
   - invalid working directory
   - environment filtering
   - timeout
   - cancellation
   - non-zero exit
   - output truncation/bounds
   - artifact digesting
   - task-contract scope expansion rejection
   - Windows regression coverage where the existing platform matrix requires it

## Developer Ecosystem integration

The GameForge Studio Lab ecosystem registry is reference/discovery data. During Phase 07, do not install or activate every listed engine/tool.

Instead:

- expose a typed registry loader if one does not already exist;
- make tool execution contracts reference stable registry/tool IDs where useful;
- distinguish `known`, `installed`, `licensed`, `authenticated`, `callable`, and `governed` states;
- never infer install/licensing/access from registry presence;
- preserve engine/tool adapters as the integration boundary.

## Definition of done

Phase 07 is complete only when implementation, tests, and repository evidence demonstrate the execution worker boundary without weakening existing governance/security guarantees.

Report completion using repository terminology: `VERIFIED`, `FAILED`, `INFERRED`, `BLOCKED`, or `UNKNOWN`.

## Do not do yet

Do not jump ahead to Phase 08 Unreal Engine Bridge, broad engine installation, production publishing, credentialed release actions, or unrestricted autonomous shell/network access merely to demonstrate execution.

## Completion report

When Phase 07 work is ready, report:

1. files changed
2. architectural decisions
3. exact commands/tests executed
4. test/CI/Fortress evidence
5. security/governance implications
6. residual limitations
7. blockers, if any
8. exact Phase 08 handoff recommendation
