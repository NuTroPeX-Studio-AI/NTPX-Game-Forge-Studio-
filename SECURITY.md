# Security Policy

## Default posture

NTPX GameForge Ω∞ is offline-first, least-privilege, and deny-by-default.

## Protected actions

The following always require explicit human authorization:

- production publishing
- signing-key or protected credential use
- credential changes
- destructive migrations or data deletion
- protected-branch merges when policy requires review
- security-policy or permission-root changes
- paid infrastructure purchases
- public release promotion

## Verified Autonomy Governor guarantees

- Unknown principals and unmatched permissions/scopes fail closed.
- Protected capability classes preserve a human-approval gate even if a grant would otherwise allow the action.
- Human approval proofs are HMAC-SHA256 authenticated, bound to the exact request, expire, and carry one-time nonces.
- Approval nonces are durably consumed through the cross-process locked audit append and cannot be replayed by cooperating processes.
- Authorization records are append-only SHA-256 hash-chained JSONL and are fully verified before append.
- In-process runtime operations supplied to `GovernanceRuntime` execute only after durable authorization evidence; denied operations are audited and not invoked.
- Governed file mutation requires an absolute canonical scope, rejects symlink ancestors and symlink targets, and revalidates parent identity before entering the transaction boundary.
- Governed writes persist durable pre-write snapshots, stage bytes durably, replace through a platform-aware primitive, record authorization evidence, and automatically restore the pre-write state when audit/replay failure occurs after replacement.
- Human-approved file writes verify the exact proof before mutation and cannot use a proof to override denial.
- Multi-file batches reject duplicate canonical targets before mutation and roll earlier members back if a later member fails.
- Crash-recovery discovery is read-only. Explicit recovery verifies snapshot integrity/target binding and rejects corrupt or symlink-substituted artifacts.
- The Autonomy Governor is compiled and tested on Windows in addition to the full Ubuntu workspace CI gate.

## Verified Model Router guarantees

- Model/provider selection is provider-neutral and based only on declared catalog metadata plus a typed routing request/policy.
- Local-only privacy constraints exclude cloud models before ranking.
- Capability, context/output capacity, cost, latency, and reliability constraints fail closed by rejecting non-qualifying models with explicit evidence.
- Model/provider enumeration order is normalized and stable identity is the final tie-breaker, making equivalent request/policy/catalog inputs reproducible.
- Duplicate model identities, adapter provider/class inconsistencies, invalid reliability values, and repeated/empty ranking policies fail closed.
- The result is an ordered primary/fallback selection or explicit abstention; the router does not silently relax hard constraints to force a result.
- Routing evidence is SHA-256 digested for deterministic correlation.
- `ProviderAdapter`, `LocalProviderAdapter`, and `CloudProviderAdapter` expose catalog information only. Phase 04 adds no provider network-call or credential-use API.

## Phase 05 Agent Orchestrator candidate guarantees

- Agent/task contracts are validated before orchestration and canonicalized into a deterministic task graph.
- Agent identifiers must match their Autonomy Governor `PrincipalId`; task-governed requests must already name that same assigned principal.
- Duplicate identifiers, unknown assigned agents, unknown dependencies, self-dependencies, dependency cycles, and required-skill mismatches fail closed.
- Ready-task order and the canonical plan digest are deterministic for equivalent contracts regardless of input enumeration order.
- Per-agent `max_parallel_tasks` bounds concurrent running task state.
- Model selection is delegated to the verified Model Router. Routing abstention escalates rather than weakening hard model constraints.
- Retry budgets are immutable task-contract inputs. Exhausted retryable failures escalate rather than extending the budget automatically.
- Permanent failure/escalation/blocking propagates dependency blocking to downstream pending work.
- Governed authorization can be requested only for a running task and only when the exact request was predeclared in that task contract.
- Exact governed requests are submitted to `GovernanceRuntime::authorize`; the orchestrator cannot manufacture a permit, override denial, or bypass required human approval.
- Phase 05 adds no model invocation, network connection, credential use, tool execution, repository mutation, engine execution, publishing, or release API.

## Authority boundaries

- Selecting a cloud model does not authorize a network connection, API request, paid usage, or credential access.
- Selecting a model that advertises tool-use capability does not authorize any tool invocation.
- Scheduling or dispatching a task does not authorize that task to use a governed capability.
- A task cannot expand its own governed scopes; undeclared requests fail before Governor submission.
- Routing evidence and orchestration evidence are not Autonomy Governor permits and cannot override a denial or human approval gate.
- A returned `AuthorizationPermit` records authorization for an exact request; Phase 05 does not claim that an external operation was executed.
- Provider capability, cost, latency, and reliability values are input metadata. Structural consistency is validated, but the router does not independently authenticate or prove those provider claims.
- Future live-provider, tool, repository, engine, and Agent Zero execution adapters must keep credentials external, obtain Governor authorization for governed actions, and preserve protected-action human approval.

## Residual security boundaries

- Advisory audit/transaction locks coordinate cooperating processes only. An actor bypassing Governor APIs is outside the locking guarantee.
- Audit and rollback SHA-256 chains/digests are tamper-evident, not authenticated against an actor able to rewrite all relevant files and recompute hashes.
- Canonical path resolution plus parent identity revalidation reduces path-alias, symlink, and directory-replacement risk but does not pin a directory handle for the entire mutation. Hostile TOCTOU races are therefore not claimed to be mathematically eliminated.
- The Windows replacement fallback is reversible/failure-safe but not kernel-atomic across the target-to-backup and stage-to-target renames.
- Multi-file batches are all-or-reverted for ordinary failures, not kernel-level atomic across multiple files or crash points.
- Verified rollback snapshots are intentionally not auto-restored because a retained snapshot can represent a successful reversible transaction as well as interrupted work.
- Phase 05 orchestrator state/evidence is in-memory and serializable but not a crash-safe durable scheduler journal.
- Phase 05 has no distributed worker leases, heartbeats, cancellation propagation, or external executor. Task state therefore must not be represented as proof that real-world work occurred.
- Workflow evidence is not a cryptographically authenticated authorization audit. Governor audit evidence remains authoritative for governed capability decisions.

## Agent execution principles

- least privilege
- scoped filesystem access
- scoped network access
- ephemeral credentials
- complete audit trails
- authorization before execution
- transactional edits
- snapshot/rollback before high-risk changes
- fail closed on ambiguity or corrupted evidence
- keep selection separate from authorization and execution
- keep scheduling separate from authorization
- predeclare governed capability requests in immutable task contracts
- escalate rather than silently widening retries, scopes, or model constraints

## Secret handling

Never commit secrets, tokens, private keys, certificates, `.env` files, or production credentials. Approval-signing secrets and future model-provider/execution credentials must remain external inputs and must not be serialized or logged. Use environment variables or an approved secret store.

## Reporting

Security issues should be documented privately with reproducible evidence, impact, affected components, and remediation status. Do not publish exploitable details before remediation.
