# Architecture

## Current boundary

Phases 01–04 are verified foundations and Phase 05 Agent Orchestrator is the M05/B000014 verification candidate. GameSpec 0.1.0 is the canonical, engine-neutral, AI-provider-neutral machine-readable contract for reproducible game intent and project state. The Autonomy Governor is the verified authorization, evidence, approval, reversible-mutation, recovery, and in-process runtime-enforcement boundary. The Model Router is the verified provider-neutral selection boundary. The Agent Orchestrator adds deterministic workflow contracts, dependency scheduling, retry/escalation, evidence, and Governor authorization handoff without adding model invocation or external execution authority. Phase 06 Code Intelligence remains gated until the exact Phase 05 candidate is verified.

## Layers

1. **Studio UI** — Tauri + React desktop shell.
2. **Core services** — Rust workspace crates with explicit interfaces.
3. **Specifications** — verified GameSpec types, versioned schema, validation, dependency graph, migrations, deterministic compiler metadata, and CLI.
4. **Governance** — verified deny-by-default identities, permissions, exact scopes, risk classification, approval proofs/gates, durable audit evidence, canonical runtime permits, governed transactions, batch rollback, recovery, and rollback (Phase 03).
5. **Intelligence** — verified Model Router, Phase 05 Agent Orchestrator candidate, then code intelligence, project world model, and evaluation council (Phases 04–13).
6. **Execution** — Agent Zero, engine/tool adapters, build/test/repair (Phases 07–11).
7. **Production** — assets, narrative/audio/NPCs, controlled improvement, game build, and release (Phases 14–18).

## GameSpec boundary

- `GameSpec` is engine-neutral and AI-provider-neutral.
- Top-level objects use explicit stable IDs and explicit references.
- Deterministic generated IDs use a fixed algorithm and stable canonical serialization.
- Dependency-bearing references compile into a deterministic graph and dependency cycles are rejected.
- Legacy `0.1` documents can migrate to the current `0.1.0` contract.
- Asset records carry licensing and provenance metadata.
- Engine adapters must consume validated/compiled GameSpec data rather than create untracked canonical state.

## Autonomy Governor boundary

### Policy and approval

- Policy evaluation is deny-by-default.
- Principal, permission, and resource scope matching is exact.
- Protected credential, merge, publish, destructive, and release capabilities always require explicit human approval.
- Human approval proofs use HMAC-SHA256 and bind an authority identifier, the exact serialized request digest, a one-time nonce, issue time, and expiry time.
- Approval secrets are external inputs, require at least 32 bytes, and are not serialized or exposed through `Debug`.
- A proof can satisfy only an existing `RequireHumanApproval` result; it cannot override a denial or create a grant.
- Approval evidence is persisted and nonce consumption is durable/replay-safe across cooperating processes.

### Durable evidence

- Decision records are deterministic append-only JSONL audit entries.
- Audit entries are SHA-256 hash-chained by sequence and previous hash; existing entries are fully verified before append and corruption fails closed.
- Audit mutations acquire an exclusive advisory sibling lock. Full-chain verification, sequence allocation, hash construction, append, approval-nonce lookup/consumption, and durable flush occur under that writer lock.
- The hash chain is tamper-evident, not authenticated against an actor able to rewrite the complete log and recompute hashes.
- Locks are cooperative/advisory; a process that bypasses the governor API is outside the guarantee.

### Canonical runtime enforcement

- `GovernanceRuntime` is the in-process enforcement boundary: supplied operations execute only after policy evaluation and durable authorization evidence.
- Denied runtime requests are durably audited and do not invoke the supplied operation.
- Human-gated runtime operations require a valid request-bound one-time approval proof before invocation.
- Governed filesystem targets must be absolute.
- Existing parent ancestors are inspected without following symlinks; symlink ancestors and symlink targets are rejected.
- The target parent is canonicalized, the canonical resource string must exactly match policy scope, parent identity is captured, and that identity is revalidated immediately before mutation.
- On Unix, parent identity includes device/inode. On other platforms it includes the canonical parent path.

### Reversible mutation

- Governed local-file transactions re-evaluate policy internally and require exact `Filesystem` + `Write` scope matching.
- Direct transactions execute only on policy `Allow`. Human-approved transactions execute only on `RequireHumanApproval` plus a valid exact proof verified before mutation.
- Transactions use per-target advisory locks, durable SHA-256-protected pre-write snapshots, same-directory durable staging, platform-aware replacement, durable audit append, and automatic restoration when replay/audit persistence fails after replacement.
- Successful commits retain rollback snapshots and return receipts for explicit restoration.
- `GovernedFileBatch` rejects duplicate canonical targets before mutation and routes every member through `GovernanceRuntime`. If a later member fails, earlier members are rolled back in reverse order. This is all-or-reverted for ordinary failures, not kernel-level atomic multi-file replacement.

### Recovery and platform semantics

- `RecoveryScanner` discovers rollback artifacts read-only and never automatically restores them because a retained verified snapshot may represent either an intentionally reversible committed transaction or interrupted work.
- Explicit recovery re-reads and verifies snapshot SHA-256 integrity and target binding, rejects symlink/non-file snapshot substitution, validates canonical target identity, and removes the snapshot only after successful restoration.
- Existing snapshot identity is compared canonically so Windows short/long-path aliases do not create false mismatches.
- Transaction restoration and recovery share the same replacement primitive.
- Unix-like replacement uses same-filesystem rename semantics.
- Windows first attempts rename replacement and otherwise uses a reversible target-backup fallback. The fallback is failure-safe but is not claimed to be kernel-atomic across both renames.
- CI compiles/tests `autonomy-governor` on Windows in addition to the full Ubuntu workspace gate.

### Residual limits

- Canonicalization plus parent identity revalidation is path/race hardening, not a proof of hostile TOCTOU elimination: the implementation does not pin a directory handle across the complete mutation sequence.
- Snapshot and audit hashes are tamper-evident, not secret-key authenticity proofs.
- The Governor grants no implicit production credential, network, repository merge, engine, publishing, or release authority. Later adapters must consume the Governor boundary rather than bypass it.

## Model Router boundary

### Provider-neutral catalog contract

- `model-router` depends on provider-neutral typed descriptors rather than a vendor SDK.
- `ProviderAdapter` exposes read-only provider identity, locality class, and model catalog data only.
- Concrete local and cloud catalog adapters implement the same interface and validate descriptor ownership/class consistency.
- Model capabilities are typed and include text, code, reasoning, vision, audio, speech, embeddings, structured output, and tool-use declarations.
- Catalog descriptors also declare context/output capacity, expected latency, token-cost metadata, and reliability basis points.

### Constraint filtering and deterministic ranking

- Routing requests contain selection requirements and estimates, not prompt/project payload content.
- Hard constraints cover required capabilities, context/output capacity, local-only privacy, optional estimated-cost ceiling, optional latency ceiling, and optional reliability floor.
- Rejected candidates remain in evidence with deterministic explicit rejection reasons.
- Eligible candidates are ranked lexicographically using an ordered `RoutingPolicy`; integer comparisons avoid floating-point score instability.
- Provider/model identity is the final tie-breaker.
- Adapter/model enumeration is normalized before filtering/ranking, so equivalent catalogs produce the same decision regardless of adapter order.
- The result contains one primary candidate and a bounded ordered fallback list, or explicit abstention if no model qualifies.

### Routing evidence and authority separation

- The complete routing request, policy, eligible set, rejection set, and outcome form deterministic routing evidence.
- Evidence is SHA-256 digested for reproducible correlation. The digest is not an authorization token or secret-key authenticity mechanism.
- A routing decision selects a model identity only. It does not invoke the model or authorize network access, credentials, tools, filesystem/repository writes, merges, engine actions, publishing, or release.
- A selected model advertising tool-use capability still requires separately governed tool execution.
- Future provider invocation adapters must use the Autonomy Governor for governed capabilities and must preserve human approval gates for protected actions.
- Catalog cost/latency/reliability/capability data is input metadata. Phase 04 validates structure and consistency but does not independently prove that provider-declared values are current or empirically correct.

## Agent Orchestrator boundary

### Immutable workflow contracts

- `AgentContract` binds a stable orchestrator agent identifier to the same Autonomy Governor `PrincipalId`, a role, declared skills, and an explicit parallel-task ceiling.
- `TaskContract` declares one immutable unit of work: objective, dependencies, required skills, assigned agent, Model Router request/policy, bounded attempts, and exact governed requests that may later be submitted for authorization.
- Agent/task identifiers are unique and nonblank; routing request identity must equal the task identity.
- Required skills must be declared by the assigned agent.
- Every governed request in a task must already name the assigned agent principal. Principal drift fails validation.
- Dependency references must resolve and self-dependency/cycles fail closed.
- Canonical ordered contracts produce a SHA-256 plan digest independent of input enumeration order.

### Deterministic scheduling and state transitions

- Ready work requires every dependency to be `Succeeded`.
- Ready task identifiers are ordered deterministically by stable task identity.
- Per-agent `max_parallel_tasks` prevents dispatch beyond the declared concurrency ceiling.
- `dispatch_next` asks the verified `ModelRouter` for a model decision and records that routing digest in task evidence; it does not invoke the selected model.
- Routing abstention moves the task to `Escalated` without consuming an execution attempt.
- Attempt outcomes are explicit: success, permanent failure, or retryable failure.
- Retryable failures move to `RetryScheduled` only while the attempt count remains below immutable `max_attempts`; exhaustion escalates rather than silently widening the retry budget.
- Failed, escalated, or blocked prerequisites cause dependent pending/retry work to become `Blocked`.

### Governor handoff and authority separation

- A governed capability may be submitted only for a task currently in `Running` state.
- The exact request must already exist in the immutable task contract. Undeclared capability requests fail before reaching the Governor.
- `authorize_governed_request` delegates the exact request to `GovernanceRuntime::authorize`. Governor denial, missing approval, replay failure, and audit failure remain authoritative.
- A successful `AuthorizationPermit` is recorded as workflow evidence but is not interpreted as proof that an external model/tool/repository/engine action executed.
- Agent Orchestrator scheduling never grants network, credential, tool, filesystem, repository, engine, publishing, or release authority.
- Later execution adapters must consume the orchestration context and Governor permit without creating a parallel permission path.

### Phase 05 residual limits

- Orchestrator state and workflow evidence are serializable in-memory structures in Phase 05; there is no crash-safe scheduler journal yet.
- Phase 05 has no distributed queue, worker leases, heartbeats, cancellation propagation, work stealing, resource-aware global optimization, or remote execution.
- `dispatch_next` selects one next eligible task per call. Parallelism is represented/enforced by state and per-agent limits, not by spawning workers.
- Workflow evidence is not a cryptographically authenticated authorization log; governed authorization evidence remains in the Autonomy Governor audit log.
- Phase 05 implements no live model inference or external task executor.

## Non-negotiable boundaries

- Shared core crates must not depend on a specific game engine.
- Shared core crates must not depend on a specific AI provider.
- Project state that must be reproducible cannot live only in model memory.
- Destructive, credentialed, financial, production, and public-release actions require human authorization.
- Every automated write must ultimately be auditable and reversible.
- GameSpec validation does not grant agents permission to execute tools or change repositories.
- Model selection is not execution authority.
- Orchestration/scheduling is not authorization authority.
- Later model, agent, tool, network, repository, and engine adapters must obtain Governor authorization rather than invent parallel permission paths.

## Repository layout

```text
apps/                  User-facing applications
crates/                Shared Rust core services, governance, GameSpec, Model Router, and Agent Orchestrator
schemas/               Versioned GameSpec JSON Schema contracts
examples/              Reference and migration fixtures
docs/                  ADRs and verification evidence
.github/workflows/      CI validation
.gameforge/             Build/version metadata
```
