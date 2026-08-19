# Phase 05 Agent Orchestrator verification

Status: CANDIDATE — exact stamped-head CI required before merge.

Candidate build: `NTPX-GFΩ-G0-v0.5.0-dev.1-M05-B000014`

## Acceptance gate

- [x] provider-neutral `agent-orchestrator` Rust workspace crate;
- [x] immutable typed `AgentContract` and `TaskContract` inputs;
- [x] agent identity bound to the Autonomy Governor `PrincipalId`;
- [x] deterministic SHA-256 canonical plan digest independent of input enumeration order;
- [x] dependency DAG validation and cycle rejection;
- [x] dangling/self dependency rejection;
- [x] assigned-agent and required-skill validation;
- [x] per-agent parallel-task limits;
- [x] deterministic ready-task ordering;
- [x] Model Router integration for dispatch selection without model invocation;
- [x] routing abstention escalates without consuming an execution attempt;
- [x] bounded retries with explicit retry exhaustion escalation;
- [x] permanent failure and downstream dependency blocking;
- [x] task state/evidence records for dispatch, retries, success/failure/escalation/blocking;
- [x] exact predeclared governed-request validation;
- [x] governed requests submitted through `GovernanceRuntime::authorize` only;
- [x] Governor denials/human-approval requirements cannot be bypassed by orchestration;
- [x] orchestration introduces no model invocation, tool execution, network, credential, repository, engine, publishing, or release authority;
- [x] implementation candidate Ubuntu workspace formatting passes;
- [x] implementation candidate strict Clippy passes with warnings denied;
- [x] implementation candidate full workspace tests including Agent Orchestrator tests pass;
- [x] implementation candidate desktop frontend typecheck/build passes;
- [x] implementation candidate Windows Autonomy Governor check/tests remain green.

## Evidence lineage

- CI run `32209400118` identified rustfmt differences in the initial Phase 05 candidate. Formatter output was applied without changing orchestration policy or weakening tests.
- CI run `32209552763` passed formatting and identified two strict Clippy issues: redundant double-`must_use` and an internal dispatch panic path. The redundant attribute was removed and the panic path was replaced with a typed `OrchestratorError`; no lint was suppressed.
- CI run `32209679408` passed the repaired implementation candidate: Rust formatting, strict Clippy, full workspace tests including Agent Orchestrator acceptance tests, desktop frontend typecheck/build, and Windows Autonomy Governor regressions.
- The final B000014 documentation/build/UI stamp must pass the same CI gate on the exact branch head before this record changes from CANDIDATE to VERIFIED and before PR #15 is merged.

## Explicit boundaries

- Model routing chooses a candidate model; it does not invoke that model.
- Orchestration chooses eligible workflow work; it does not authorize external capability use.
- `GovernanceRuntime` remains the sole verified authorization boundary for governed requests.
- Task contracts cannot dynamically self-grant new governed scopes; undeclared requests fail before Governor submission.
- Phase 05 task state/evidence is serializable in-memory state, not yet a crash-safe durable distributed scheduler journal.
- An `AuthorizationPermit` proves authorization evidence for its request; Phase 05 still does not implement the later external execution adapter.

## Next phase

Phase 06 — Code Intelligence: repository parsing/indexing, symbols, references, call/dependency graphs, change-impact analysis, and refactor intelligence behind the existing governance/orchestration boundaries.