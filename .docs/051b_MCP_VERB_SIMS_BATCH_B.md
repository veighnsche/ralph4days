# MCP Verb Discovery Simulations — Batch B (Sims #16–20)

Agent discovers what "exhaust pipe" tools it needs to signal back to Ralph during task execution.

---

## Simulation #16

**Project:** Social media scheduler (Python/FastAPI)
**Feature:** webhook-system
**Discipline:** backend
**Task:** Implement webhook retry logic with exponential backoff for failed deliveries
**Complication:** Task description contradicts codebase (current system uses sync delivery, not async retry queue)

### Walkthrough

Agent starts by reading the task description: "Add exponential backoff retry logic to webhook delivery." Seems straightforward. Searches for the webhook delivery handler and finds `webhooks.py` with a synchronous `deliver_webhook()` function. Agent reviews related task history and discovers three completed tasks: "Implement webhook signature validation" (2 weeks ago), "Add webhook event batching" (1 week ago), and "Create webhook admin dashboard" (3 days ago).

Opens the event batching task notes and sees: "Switched from async Celery queue to sync delivery to reduce infrastructure complexity." This directly contradicts the current task, which assumes an async queue exists. Agent checks the codebase: no Celery, no background workers, only sync HTTP calls with basic timeout handling.

Agent realizes the task description was written before the architecture decision changed. Now stuck: should retry logic be async (task spec) or sync (codebase reality)? If sync, what happens during delivery failures? If async, that's a major infrastructure change that wasn't scoped.

Agent starts writing code assuming sync retry loops, but realizes acceptance criteria say "Verify retries are tracked in audit_logs table." Queries the database schema: `audit_logs` table doesn't exist yet. A *different* task ("Implement audit-logging for webhook events") is marked pending and depends on this one.

Agent is now blocked on a dependency loop and architectural mismatch. It needs to signal back: "The task description and codebase contradict. Also, I found an unmapped dependency. What's the intent?"

### Tool Calls Made

1. → TOOL: `flag_task_contradiction(task_id: T-42, description_quote: "retry queue with exponential backoff", codebase_reality: "sync delivery only", severity: "architectural")` — WHY: Task assumes async infrastructure that was removed by a prior task. Need orchestrator to clarify intent before proceeding with contradictory implementation.

2. → TOOL: `report_unmapped_dependency(task_id: T-42, blocking_task: "T-38 (audit-logging)", type: "prerequisite", status: "pending")` — WHY: Acceptance criteria require audit table that another task is supposed to create. Agent can't verify its own work without that table existing first.

3. → TOOL: `request_clarification(task_id: T-42, question: "Should retry logic be sync (fits current codebase) or async (fits task description)?", options: ["keep-sync-add-simple-retry", "revert-to-async-queue", "rewrite-task-for-sync"], preferred: "keep-sync-add-simple-retry")` — WHY: Can't proceed with implementation without architectural guidance. Agent has a recommendation but needs explicit approval.

### Verbs Used

- `flag_task_contradiction` — Alert orchestrator when task spec conflicts with actual codebase state
- `report_unmapped_dependency` — Signal unknown inter-task dependencies (task B requires work from task A, but not marked as such)
- `request_clarification` — Ask orchestrator to resolve ambiguous design decisions before proceeding

---

## Simulation #17

**Project:** Fleet management system (C#/.NET)
**Feature:** analytics-dashboard
**Discipline:** fullstack
**Task:** Build real-time vehicle location map using Leaflet and WebSocket feed
**Complication:** Referenced file no longer exists (the coordinate-normalization utility was deleted in a refactor two sprints ago)

### Walkthrough

Agent opens the task: "Build dashboard map component. Use existing coordinate normalizer from `Utils/CoordNormalizer.cs`." Agent searches for the file. It's gone. Not in version control for the last 45 commits. Agent checks the refactoring task from two sprints ago: "Consolidate utility functions" — the task notes say "CoordNormalizer merged into VehicleDataProcessor."

Agent looks at `VehicleDataProcessor.cs`. The class has grown to 1200 lines and does normalization, filtering, sorting, and aggregation. Agent needs only the normalization piece. Extracting it would require refactoring `VehicleDataProcessor` or duplicating code.

Agent also discovers: the task description says "subscribe to WebSocket feed at `ws://data-service:8080/vehicles`" but the `.env` file has no data-service URL defined, and the environment validation script doesn't check for it. Agent checks slack notes from architecture discussions and finds: "Data service moved to Kubernetes; endpoint is dynamic." But which environment variable? Agent finds three references: `DATA_SERVICE_URL`, `VEHICLE_STREAM_ENDPOINT`, and `GPS_FEED_HOST` in different parts of the codebase.

Agent is now uncertain:
1. Should it extract the normalizer from VehicleDataProcessor (refactoring), duplicate it (code smell), or re-implement it (time cost)?
2. Which WebSocket endpoint variable is correct?

Agent needs the orchestrator to decide: is this a task for the agent to solve, or is this evidence of incomplete refactoring from a prior task?

### Tool Calls Made

1. → TOOL: `report_missing_artifact(task_id: T-61, artifact: "Utils/CoordNormalizer.cs", last_seen: 45-commits-ago, removed_by: "T-32 (Consolidate utility functions)", current_location: "Partially in VehicleDataProcessor.cs")` — WHY: Task references deleted code. Agent needs orchestrator to confirm it's safe to extract or refactor.

2. → TOOL: `flag_environment_config_mismatch(task_id: T-61, expected_var: "ws://data-service:8080/vehicles", actual_candidates: ["DATA_SERVICE_URL", "VEHICLE_STREAM_ENDPOINT", "GPS_FEED_HOST"], resolved: false)` — WHY: Three conflicting environment variable names in codebase. Agent can't determine correct endpoint without orchestrator input.

3. → TOOL: `suggest_refactoring(task_id: T-61, scope: "Extract coordinate normalization from VehicleDataProcessor into standalone utility", estimated_effort: "2 hours", blocking_current_task: true)` — WHY: To complete the task, agent needs to refactor. Should this be a separate task, or part of the current one?

### Verbs Used

- `report_missing_artifact` — Signal when a task references code that no longer exists
- `flag_environment_config_mismatch` — Alert when required config variables are ambiguous or missing
- `suggest_refactoring` — Propose refactoring work that's a prerequisite to completing the task

---

## Simulation #18

**Project:** Personal finance tracker (SvelteKit/TypeScript)
**Feature:** offline-sync
**Discipline:** frontend
**Task:** Implement conflict resolution for transaction edits made offline
**Complication:** Race condition makes naive approach fail (two tabs editing the same transaction simultaneously can corrupt state)

### Walkthrough

Agent reviews the offline-sync architecture. Previous task ("Implement local transaction cache") created an IndexedDB store that syncs on reconnect. Task ("Add timestamp-based conflict detection") implemented last-write-wins for conflicting transactions. Current task: implement better conflict resolution UI where users can choose which version to keep.

Agent starts prototyping: when sync detects conflicts, queue them in a "conflicts queue" and render a modal for user resolution. Initial code works fine in single-tab testing. But then agent realizes: what if two session tabs are open on the same transaction? Tab A edits transaction #42, goes offline. Tab B also edits transaction #42, goes offline. Both go online at the same time. Both try to sync. The local IndexedDB state could be read by both tabs simultaneously, both write their changes, and one write gets overwritten.

Agent checks the prior task ("Implement local transaction cache"). It uses simple IndexedDB writes without transactions. Reading the notes: "IndexedDB transactions seemed overkill; most users won't have multiple tabs." But the complication says they will, and it's not handled.

Agent considers three options:
1. Use IndexedDB transaction API (adds complexity, platform compatibility risk)
2. Add a global lock at the app level (but tabs are separate contexts, can't share memory)
3. Use shared worker or service worker to coordinate (significant architectural change)

Agent needs to know: is the multi-tab scenario actually a requirement, or is last-write-wins acceptable? The task acceptance criteria don't mention concurrency. But the complication suggests it's a real scenario.

Agent starts writing code using option 1 (IndexedDB transactions) but realizes this changes how the entire sync pipeline works. Every other task depends on the simple write path. Now blocked.

### Tool Calls Made

1. → TOOL: `report_concurrency_hazard(task_id: T-89, scenario: "Same transaction edited in two tabs simultaneously offline", consequence: "State corruption on simultaneous sync", severity: "critical")` — WHY: Agent discovered a race condition that current architecture doesn't handle. Orchestrator needs to decide if this is in scope.

2. → TOOL: `request_architectural_decision(task_id: T-89, question: "How should concurrent edits from multiple tabs be handled?", options: ["accept-last-write-wins", "use-indexeddb-transactions", "add-shared-worker", "document-as-limitation"], impact: "Changes sync pipeline, affects other pending tasks")` — WHY: Implementation choice affects broader architecture. Agent can't proceed without guidance.

3. → TOOL: `warn_dependency_chain_impact(task_id: T-89, affected_tasks: ["T-90 (Sync recovery)", "T-91 (Batch transaction upload)", "T-92 (Offline receipt generation)"], reason: "Any change to sync primitives requires validation across dependent tasks")` — WHY: Refactoring IndexedDB usage could break downstream tasks. Orchestrator should know about this risk before approving any direction.

### Verbs Used

- `report_concurrency_hazard` — Alert orchestrator to race conditions or concurrency issues in proposed implementation
- `request_architectural_decision` — Escalate design choices that have broad architectural impact
- `warn_dependency_chain_impact` — Signal when changes to current task could invalidate other pending tasks

---

## Simulation #19

**Project:** Internal HR portal (Next.js/TypeScript)
**Feature:** role-permissions
**Discipline:** security
**Task:** Enforce department-level permission scoping for HR admins
**Complication:** Task description contradicts codebase (permissions model changed from role-based to attribute-based three weeks ago, but task assumes roles)

### Walkthrough

Agent reads task: "Add department filter for HR admin role. Ensure admins can only access employees in their assigned department." Searches the codebase for role definitions. Finds `roles.ts` with ADMIN, MANAGER, EMPLOYEE roles. But the file has a deprecation comment: "DEPRECATED: See `permissions.ts` for attribute-based model."

Opens `permissions.ts`. The entire system has been refactored. Instead of roles, there's an `Attribute` system: `{ resource: "employee", action: "read", scope: { department: "sales" } }`. The old role-based checks are still in the code but wrapped with warnings.

Agent checks the refactoring task from three weeks ago: "Migrate to attribute-based permissions." It's marked complete, but the old role checks aren't fully removed. The task description says: "maintain backward compatibility during migration" but also says "old role model is read-only."

Current task wants to add a new feature (department scoping) to the *old* role-based system. But that's supposedly deprecated. Should agent:
1. Implement it in the role model (contradicts refactoring intent)?
2. Implement it in the attribute model (requires understanding new system)?
3. Wait for clarification?

Agent checks which system the UI actually uses. Finds two permission checks in the component: one using old `hasRole()`, one using new `can()`. Both are active. Data model is inconsistent.

Agent also discovers: the task acceptance criteria say "Verify in test_permissions.ts" but that file only tests the new attribute system. No tests exist for the old role system anymore.

Agent realizes: the refactoring task was supposed to be complete, but the codebase is in an inconsistent state. Current task can't be done correctly until someone cleans this up.

### Tool Calls Made

1. → TOOL: `flag_codebase_inconsistency(task_id: T-71, issue: "Role-based system marked deprecated but still actively used; attribute-based system partially deployed", status: "migration-incomplete", affected_task: "T-42 (Migrate to attribute-based permissions)")` — WHY: Previous task marked complete but didn't finish the job. Current task can't build on an incomplete migration.

2. → TOOL: `escalate_to_human(task_id: T-71, reason: "Task description assumes old permission model; codebase uses two inconsistent models simultaneously. Requires decision: finish migration or rollback?", required_decision: "which-permission-model-is-authoritative")` — WHY: This is beyond code logic. Orchestrator needs human judgment on whether to finish the migration or support both systems.

3. → TOOL: `block_on_prior_task(task_id: T-71, blocking_task: "T-42 (Migrate to attribute-based permissions)", required_status: "fully-complete-no-legacy-code")` — WHY: Current task is blocked by incomplete work. Orchestrator should resolve T-42 before starting T-71.

### Verbs Used

- `flag_codebase_inconsistency` — Alert orchestrator when codebase is in a transitional or inconsistent state that affects current task
- `escalate_to_human` — Signal when a decision requires human judgment beyond code logic
- `block_on_prior_task` — Formally declare that current task cannot proceed until a prior task is completed to a specific standard

---

## Simulation #20

**Project:** IoT sensor dashboard (Rust/Axum + HTMX)
**Feature:** caching-layer
**Discipline:** devops
**Task:** Add Redis caching for sensor readings with 5-minute TTL
**Complication:** Existing abstraction doesn't support what's needed (the data fetching layer uses generic trait that doesn't expose cache-friendly attributes like "is this data cacheable?")

### Walkthrough

Agent reviews the caching task. The goal: cache sensor readings in Redis, 5-minute TTL. Sensor readings come from a `DataFetcher` trait in `src/data/mod.rs`. The trait is generic:

```rust
pub trait DataFetcher {
    async fn fetch(&self, sensor_id: u32) -> Result<Reading, Error>;
}
```

Agent sees implementations: `ApiDataFetcher` (HTTP API), `DatabaseDataFetcher` (SQLite), `MockDataFetcher` (for tests). These are used polymorphically in handlers. The issue: to add caching, agent needs to know which sources *should* be cached. HTTP API? Yes. Database? Maybe (depends on update frequency). Mock? No.

Agent considers wrapping the trait:

```rust
pub trait DataFetcher {
    async fn fetch(&self, sensor_id: u32) -> Result<Reading, Error>;
    fn is_cacheable(&self) -> bool; // NEW
    fn cache_ttl(&self) -> Option<Duration>; // NEW
}
```

But this changes the trait. All seven implementations need updating. Agent searches for tasks related to trait changes. Finds "Refactor DataFetcher for pluggability" (marked pending, depends on three other tasks). This looks like the "right" place to add cache hints, but it's not ready yet.

Agent considers a workaround: check the struct name at runtime using `type_id()` or name to decide caching behavior. But this is a hack and violates Rust idioms.

Agent also discovers: Redis isn't in the dev dependencies. The `Cargo.toml` has no redis crate. Checking environment: `.env` doesn't define `REDIS_URL`. Agent searches for integration tests: none exist for the caching layer.

Agent is now stuck:
1. Can't add cache metadata to trait without breaking things
2. Can't implement runtime type-checking (not idiomatic)
3. Redis client not in deps
4. No test infrastructure for integration testing

This is bigger than the task scope suggests. Either the task is incomplete, or the codebase isn't ready.

### Tool Calls Made

1. → TOOL: `report_abstraction_limitation(task_id: T-105, abstraction: "DataFetcher trait", limitation: "Cannot express which sources are cacheable without changing trait signature", options: ["extend-trait-with-cache-metadata", "add-cacheable-wrapper-trait", "use-runtime-type-inspection", "defer-until-dataFetcher-refactor"])` — WHY: Current trait design doesn't support the metadata needed for intelligent caching. Agent needs decision on which approach to take.

2. → TOOL: `report_missing_dependencies(task_id: T-105, missing: [{"name": "redis", "required_version": "^0.24", "used_for": "caching-layer"}], dev_setup: "incomplete")` — WHY: Task requires Redis client but it's not in the project. Orchestrator needs to approve adding the dependency.

3. → TOOL: `suggest_prerequisite_task(task_id: T-105, prerequisite: "Extend DataFetcher trait with cache metadata (is_cacheable: bool, ttl: Option<Duration>)", estimated_effort: "1 hour", would_unblock: "T-105 and future caching features")` — WHY: To implement caching cleanly, the trait needs extending. This should probably be a separate task or part of the "Refactor DataFetcher" task.

4. → TOOL: `request_test_strategy(task_id: T-105, question: "How should caching layer be tested? (unit tests mocking Redis, integration tests with real Redis, just manual verification?)", impact: "Acceptance criteria depend on test coverage")` — WHY: Agent can't define acceptance criteria without knowing what "tested" means for a caching layer.

### Verbs Used

- `report_abstraction_limitation` — Alert orchestrator when existing abstractions don't support what the task requires
- `report_missing_dependencies` — Signal when required libraries/services are absent from the project setup
- `suggest_prerequisite_task` — Propose that a smaller task should precede the current one to set up preconditions
- `request_test_strategy` — Ask orchestrator for guidance on testing approach when it's ambiguous

---

## Summary of Verbs Discovered

Across all 5 simulations, the following verbs emerged naturally:

| Verb | Used In | Purpose |
|------|---------|---------|
| `flag_task_contradiction` | #16 | Alert when task spec conflicts with actual codebase |
| `report_unmapped_dependency` | #16 | Signal unknown inter-task dependencies |
| `request_clarification` | #16 | Ask orchestrator to resolve ambiguous decisions |
| `report_missing_artifact` | #17 | Signal when task references deleted code |
| `flag_environment_config_mismatch` | #17 | Alert about ambiguous or missing config |
| `suggest_refactoring` | #17 | Propose refactoring work as task prerequisite |
| `report_concurrency_hazard` | #18 | Alert to race conditions in design |
| `request_architectural_decision` | #18 | Escalate broad architectural choices |
| `warn_dependency_chain_impact` | #18 | Signal downstream consequences of changes |
| `flag_codebase_inconsistency` | #19 | Alert when codebase is in transitional state |
| `escalate_to_human` | #19 | Signal when human judgment is needed |
| `block_on_prior_task` | #19 | Formally declare blocking relationship |
| `report_abstraction_limitation` | #20 | Alert when abstractions don't support requirements |
| `report_missing_dependencies` | #20 | Signal absent libraries/services |
| `suggest_prerequisite_task` | #20 | Propose smaller task as precondition |
| `request_test_strategy` | #20 | Ask for guidance on testing approach |

### Pattern Observations

**Three classes of verbs emerged:**

1. **Status/Alert verbs** (immutable state discovery): Report facts that don't require action but inform orchestrator decisions
   - `flag_task_contradiction`, `report_missing_artifact`, `flag_environment_config_mismatch`, `flag_codebase_inconsistency`, `report_concurrency_hazard`, `report_abstraction_limitation`, `report_missing_dependencies`, `warn_dependency_chain_impact`, `report_unmapped_dependency`

2. **Decision/Request verbs** (orchestrator judgment needed): Ask orchestrator to make a choice before proceeding
   - `request_clarification`, `request_architectural_decision`, `request_test_strategy`, `escalate_to_human`

3. **Suggestion/Proposal verbs** (actionable recommendations): Propose a change that would unblock the task
   - `suggest_refactoring`, `suggest_prerequisite_task`, `block_on_prior_task`

All verbs follow a pattern: **Agent discovers something unexpected → Needs to inform Ralph → Ralph + human must decide next move.** The agent never decides to pivot, refactor other tasks, or modify dependencies on its own.
