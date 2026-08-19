# Security Policy

## Default posture

NTPX GameForge Ω∞ is offline-first, least-privilege, deny-by-default, and fail-closed on ambiguous or corrupted evidence.

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
- `GovernanceRuntime` executes supplied in-process operations only after durable authorization evidence; denials are audited and not invoked.
- Governed file mutation requires an absolute canonical scope, rejects symlink ancestors/targets, and revalidates parent identity before mutation.
- Governed writes persist durable pre-write snapshots, stage bytes durably, use platform-aware replacement, record authorization evidence, and expose explicit rollback receipts.
- Human-approved file writes verify the exact proof before mutation and cannot use a proof to override denial.
- Multi-file batches reject duplicate canonical targets before mutation and roll earlier members back if a later member fails under ordinary failure conditions.
- Recovery discovery is read-only; explicit recovery verifies snapshot integrity and target binding.
- The Governor is compiled/tested on Windows in addition to the full Ubuntu workspace gate.

## Verified Model Router guarantees

- Provider selection is provider-neutral and based on declared catalog metadata plus typed routing request/policy inputs.
- Local-only privacy and capability/context/output/cost/latency/reliability constraints fail closed.
- Equivalent inputs produce deterministic primary/fallback or explicit abstention evidence.
- Provider catalog adapters expose metadata only; routing does not authorize provider network calls, credentials, paid usage, or tools.

## Verified Agent Orchestrator guarantees

- Agent/task contracts are validated and canonicalized into a deterministic task graph.
- Agent identity is bound to the Autonomy Governor principal used by its governed requests.
- Duplicate IDs, unknown assignments/dependencies, self/cyclic dependencies, and skill mismatches fail closed.
- Ready ordering and plan evidence are deterministic; per-agent concurrency and retry budgets are explicit bounds.
- Model routing abstention escalates rather than silently weakening constraints.
- Governed authorization can be requested only for running tasks and only for the exact predeclared request.
- Requests are delegated to `GovernanceRuntime`; orchestration cannot manufacture a permit or bypass human approval.
- Phase 05 adds no external executor, model invocation, network connection, credential use, tool execution, repository mutation, engine execution, publishing, signing, or release API.

## Verified Phase 06 Code Intelligence guarantees

### Analysis and syntax evidence

- Repository/source/symbol/reference/dependency/search/diagnostic/syntax outputs are deterministic correlation evidence, not authority.
- Ambiguous and unresolved lexical references remain explicit; ambiguity never selects an arbitrary mutation target.
- Incremental reuse requires verified prior repository evidence and exact file path/language/generated-state/content-digest matching.
- Rust syntax uses `syn`; TypeScript and C++ syntax use Tree-sitter; textual Blueprint metadata remains a separate deterministic parser.
- These products are not represented as compiler/LSP semantic resolution, type analysis, or true semantic call graphs.

### Edit planning

- `code-intelligence-edits` remains read-only.
- Exact edit plans bind path/language/current source SHA-256, repository evidence SHA-256, replacement-content SHA-256, and plan correlation evidence.
- Generated-source, no-op, stale repository, and stale source plans fail closed.

### Governed application

- `code-intelligence-apply` is the narrow Phase 06 mutation bridge; it does not introduce a parallel permission, audit, transaction, or rollback system.
- Replacement-content and full-plan evidence are recomputed before mutation.
- Repository root must canonicalize to an existing directory.
- Plan target paths must be non-empty normal repository-relative paths; absolute, prefix, current-directory, and parent-traversal components are rejected.
- Targets are resolved through the Autonomy Governor filesystem boundary before the current on-disk source is read.
- Current source is re-read and plan freshness is revalidated immediately before entering the governed write path.
- Direct application calls only `GovernanceRuntime::write_file`; human-gated application calls only `write_file_with_approval`.
- A valid/tampered plan cannot override denial or weaken/satisfy a human approval requirement.
- Successful writes return the existing audited rollback-capable `TransactionReceipt`.
- Tests cover successful mutation + rollback, stale source refusal, replacement tamper refusal, repository traversal refusal, human-gate refusal, and exact approved application.

## NTPX Fortress bootstrap gate

- Fortress repository monitoring is carried forward as a required CI gate.
- Repository changes must stay within the configured bootstrap classification/secrets policy.
- Do not commit production secrets, private signing keys, protected credentials, `.env` files, or crown-jewel higher-classification material to satisfy tests.
- Fortress complements but does not replace the Autonomy Governor: Fortress protects/gates repository content while the Governor controls runtime capability authorization and reversible mutation.

## Authority boundaries

- GameSpec validity is not permission.
- Model selection is not network/model/tool execution authority.
- Scheduling/dispatch is not governed capability authority.
- Code analysis or an edit plan is not source-mutation authority.
- A task cannot expand its own governed scopes; undeclared requests fail before Governor submission.
- Routing/orchestration/Code Intelligence evidence cannot override a Governor denial or human approval gate.
- Future Agent Zero, provider, tool, repository, and Unreal adapters must keep credentials external and consume the Governor rather than create parallel permission paths.

## Residual security boundaries

- Advisory audit/transaction locks coordinate cooperating processes only; actors bypassing Governor APIs are outside the locking guarantee.
- Audit/rollback SHA-256 evidence is tamper-evident, not authenticated against an actor able to rewrite all relevant state and recompute hashes.
- Canonical path resolution plus parent identity revalidation reduces alias/symlink/directory-replacement risk but does not pin a directory handle for the entire mutation; hostile TOCTOU elimination is not claimed.
- B000028 rechecks source freshness immediately before Governor entry, but does not implement a kernel-level content compare-and-swap against a non-cooperating writer racing between those steps.
- Windows fallback replacement is reversible/failure-safe but not kernel-atomic across both renames.
- Multi-file batches are all-or-reverted for ordinary failures, not kernel-level atomic across multiple files/crash points.
- Verified rollback snapshots are not auto-restored because a retained snapshot may represent a successful reversible transaction or interrupted work.
- Phase 05 scheduler state is not a crash-safe durable distributed execution journal.
- Phase 06 does not add external tool/process/model/engine execution, compiler/LSP semantic guarantees, Blueprint binary mutation, repository merge, network/credential, publishing, signing, or release authority.

## Agent execution principles

- least privilege
- scoped filesystem/network access
- external/ephemeral credentials
- complete audit trails
- authorization before execution
- evidence/freshness validation before mutation
- transactional edits with rollback evidence
- fail closed on ambiguity/corruption/drift
- selection ≠ authorization ≠ execution
- scheduling ≠ authorization
- planning/analysis ≠ mutation authority
- predeclare governed capability requests
- escalate rather than silently widening retries, scopes, or model constraints

## Secret handling

Never commit secrets, tokens, private keys, certificates, `.env` files, or production credentials. Approval-signing secrets and future model-provider/execution credentials remain external inputs and must not be serialized or logged. Use environment variables or an approved secret store.

## Reporting

Security issues should be documented privately with reproducible evidence, impact, affected components, and remediation status. Do not publish exploitable details before remediation.
