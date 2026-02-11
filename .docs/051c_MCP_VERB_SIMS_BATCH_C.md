# MCP Verb Discovery Simulations — Batch C

Independent simulation of 5 random project/feature/discipline/complication combos. Goal: discover natural verbs for agent → orchestrator communication.

---

## Simulation #21

**Project:** Personal Finance Tracker (SvelteKit/TypeScript)
**Feature:** role-permissions
**Discipline:** fullstack
**Task:** Implement role-based access control for transaction views — admins see all, users see only their own
**Complication:** A dependency task marked "done" left incomplete code

### Walkthrough

Agent starts by reading the task description and finding the prior "permission middleware" task was allegedly complete. Agent checks `src/lib/auth.ts` and finds it only has a `getUser()` function but NO `can(user, action, resource)` check. The schema update for `user_roles` table exists but the migration was only partially merged.

Agent proceeds incrementally:
1. Reads transaction fetcher (`src/routes/transactions/+page.server.ts`)
2. Discovers it calls unguarded `db.getTransactions()` with no role filtering
3. Attempts to add role check but realizes the dependency is actually broken — can't proceed without fixing the auth layer first
4. Could either: patch it themselves (scope creep) or escalate

At minute 8, agent realizes this blocks the task. The acceptance criteria say "must pass role-based test suite in `/tests/auth.integration.test.ts`" but that test file tries to import `canViewTransaction()` from the incomplete auth module.

Agent also notices the parent task status is "Done" but has comment "forgot to add the guard checks" — it was literally abandoned mid-implementation.

Agent needs to: NOTIFY orchestrator that a prior task was marked complete incorrectly, RECORD that this task is now blocked, SUGGEST that the prior task needs reopening.

### Tool Calls Made

1. → TOOL: `flag_incomplete_dependency(task_id: "auth-middleware-impl", reason: "Permission guard functions defined but guard calls not integrated", severity: "blocks-current")` — WHY: Agent discovered the prior task was marked done but is actually a stub. Orchestrator needs to know it was incomplete so it can be re-queued or flagged for human review.

2. → TOOL: `request_task_context_update(task_id: "role-permissions-impl", field: "depends_on", new_value: ["auth-middleware-impl", "user-roles-schema"])` — WHY: The task description only lists one dependency, but agent has now discovered a hidden second dependency. Orchestrator should update task metadata to reflect reality.

3. → TOOL: `escalate_for_human_review(task_id: "role-permissions-impl", reason: "Prior dependency incomplete; task cannot proceed without auth guards being wired in", suggestion: "Re-open auth-middleware-impl or provide complete auth.ts")` — WHY: This is a blocker that can't be resolved by the agent working in isolation. Orchestrator should surface to human.

### Verbs Used

- `flag_incomplete_dependency` (detect broken assumptions about prior work)
- `request_task_context_update` (update task metadata when reality differs from description)
- `escalate_for_human_review` (signal that task cannot proceed and human judgment needed)

---

## Simulation #22

**Project:** Music Production DAW Plugin (C++/JUCE)
**Feature:** caching-layer
**Discipline:** backend
**Task:** Implement DSP filter state caching to reduce real-time computation overhead during repetitive preset changes
**Complication:** Third-party library has breaking change

### Walkthrough

Agent pulls the latest JUCE framework headers as specified in `CMakeLists.txt` (version 8.0.x). The task is to add a filter state cache in `src/DSP/FilterCache.cpp` using JUCE's `ReferenceCountedObject` pool.

Agent writes the first version using `juce::ReferenceCountedArray<FilterState>` which matches the JUCE 7.x API from the prior feature branch. They compile and hit a wall: JUCE 8.0 deprecated `ReferenceCountedArray` in favor of a new pointer API.

Agent looks at git history and sees the last successful build was 3 months ago against JUCE 7.1. Current pinned version is 8.0.14. The deprecation warning became a hard error in 8.0.8+.

Agent could:
1. Revert JUCE to 7.1 (blocks other features that depend on 8.0)
2. Rewrite using new API (adds 6+ hours to a 2-hour task)
3. Work around it (hack that makes code unmaintainable)

At minute 20, agent checks the feature's prior tasks. One task says "update to JUCE 8.0 for real-time threading improvements" — it's marked done but nobody validated that all dependents would still compile.

Agent needs to: REPORT the version conflict, QUERY what JUCE version all features are pinned to, SIGNAL to orchestrator that the JUCE upgrade wasn't actually complete.

### Tool Calls Made

1. → TOOL: `report_dependency_version_mismatch(library: "juce", expected_version: "7.1.x", actual_version: "8.0.14", error_message: "ReferenceCountedArray removed — API breaking change", affected_files: ["src/DSP/FilterCache.cpp"])` — WHY: Agent hit a real blocker caused by a version mismatch. Orchestrator needs to know which dependencies are causing pain across tasks.

2. → TOOL: `query_project_constraints(constraint_type: "library_versions", library: "juce")` — WHY: Agent needs to know if JUCE 8.0 is locked globally or per-feature. Orchestrator knows the project-wide constraints.

3. → TOOL: `record_version_incompatibility(from_task: "update-juce-8.0", to_task: "caching-layer-impl", breaking_change: "ReferenceCountedArray deprecated", resolution: "pending-manual-review")` — WHY: The prior task claimed success but broke downstream work. Orchestrator should track this as a failed upgrade that needs manual intervention or rollback.

### Verbs Used

- `report_dependency_version_mismatch` (signal library version conflict)
- `query_project_constraints` (ask orchestrator about global pinned versions)
- `record_version_incompatibility` (document breaking change for cross-task visibility)

---

## Simulation #23

**Project:** Fleet Management System (C#/.NET)
**Feature:** webhook-system
**Discipline:** devops
**Task:** Implement webhook delivery retry logic with exponential backoff and dead-letter queue for failed deliveries
**Complication:** Existing abstraction doesn't support what's needed

### Walkthrough

Agent examines the existing event system in `src/Events/EventBus.cs`. There's a `IEventHandler<T>` interface that the team uses for in-process events (e.g., "VehicleLocationUpdated"). The architectural pattern is clean: publish → all subscribers notified instantly.

Task requires: webhooks to external third-party systems (HTTP POST to customer-provided URLs) with retry logic. But `IEventHandler` is synchronous, executes in the same transaction, and has no retry semantics.

Agent reads the feature spec. It says "reuse EventBus infrastructure for consistency." Agent tries to bolt retry logic onto `IEventHandler`:
- Create `IRetryableEventHandler` that extends it? Breaks Liskov — not all handlers are retryable
- Make EventBus itself handle retries? It's already doing sync dispatch — adding async + queue + DLQ changes its contract entirely
- Create a parallel `WebhookDispatcher` wrapper? That's duplicate infrastructure, violates DRY

At minute 35, agent realizes the architecture was designed for in-process events, not durable external delivery. Adding webhooks to EventBus will make it do three things (sync handlers, async retries, dead-lettering) when it should do one.

Agent needs to: PROPOSE that webhooks need a separate service, GET APPROVAL that this is OK, or GET CLEAR GUIDANCE on how to adapt the existing abstraction.

### Tool Calls Made

1. → TOOL: `request_architecture_guidance(feature: "webhook-system", issue: "Existing IEventHandler abstraction assumes in-process synchronous execution; webhook delivery is async + durable + has retry semantics", question: "Should webhooks use separate WebhookService or extend EventBus? Current task description says reuse EventBus but that seems architecturally wrong.")` — WHY: Agent hit a design decision that's outside their scope to unilaterally choose. This needs human/architecture review before proceeding.

2. → TOOL: `propose_task_split(parent_task: "webhook-system", new_task: "extract-webhooks-service", description: "Create dedicated WebhookService for durable external delivery; decouple from EventBus", estimated_effort: "8h", blocks_current_task: true)` — WHY: Agent believes this task is actually TWO tasks: (1) design/implement separate webhook infrastructure, (2) integrate with event system. Orchestrator should consider splitting.

3. → TOOL: `flag_design_gap(feature: "webhook-system", concern: "Feature spec assumes reuse of EventBus for consistency, but EventBus design doesn't support durable retry delivery; gap between spec intent and actual implementation constraints")` — WHY: There's a mismatch between what the feature spec promises (reuse) and what's possible. Orchestrator should flag this for product/architecture review.

### Verbs Used

- `request_architecture_guidance` (ask for design decision approval)
- `propose_task_split` (suggest that one task should be two)
- `flag_design_gap` (report mismatch between spec and implementation reality)

---

## Simulation #24

**Project:** Weather Station Network (Embedded C + Python backend)
**Feature:** batch-processing
**Discipline:** data
**Task:** Implement batch-aggregation of sensor readings across 500+ stations, writing hourly summaries to data warehouse
**Complication:** Race condition makes naive approach fail

### Walkthrough

Agent reads the task and finds a Python ETL script in `src/batch/aggregate_readings.py` that's supposed to run every hour. It queries raw sensor data from Postgres, computes stats (mean, median, stddev), writes to S3 Parquet files.

Agent's first instinct: select all readings from the last hour, group by station, compute, write. Simple.

They write a test using 10 sample stations and it works. But then they notice something: the task says "handle up to 500 stations with 60-second reporting intervals" — that's 30,000 readings per hour minimum.

Agent scales the test to 30,000 reads. It still works, takes 8 seconds. But then they check the monitoring dashboard. The batch window is 900 seconds (15 minutes) but the sensor ingestion pipeline has a 5-second lag. So:
- 00:00–00:15: readings from 23:45–00:05 trickling in
- 00:15–00:30: readings from 00:00–00:20 trickling in (overlap!)
- Batch window 1 writes readings up to 00:05
- Batch window 2 writes readings 00:00–00:20
- Some readings get aggregated twice. Some get missed.

Agent looks at prior work. There's a task "implement event-driven updates" marked done, but the code just has a stub comment saying "TODO: add notification trigger to ingestion pipeline". The batch process has no way to know which readings it already processed.

Agent needs to: TRACK which hour batch last completed successfully, HANDLE the overlap window, or SIGNAL that the prior task was incomplete and broke this one.

### Tool Calls Made

1. → TOOL: `detect_data_race_condition(task_id: "batch-aggregation", race_type: "duplicate-aggregation", description: "Sensor ingestion has 5s lag; batch window overlaps with prior window. Readings 00:00–00:05 will be aggregated in both hour-1 and hour-2 batches.", mitigation_required: true)` — WHY: Agent found a logic bug that would cause data corruption. Orchestrator needs to know about the race and can help determine if it's acceptable or needs fixing.

2. → TOOL: `request_prior_task_completion_check(task_id: "event-driven-updates", reason: "batch-aggregation assumes ingestion pipeline sends notifications about completed readings, but prior task only has TODO stub", verification_needed: true)` — WHY: Agent discovered that a prior task dependency is incomplete, and their task assumes it works.

3. → TOOL: `propose_idempotency_tracking(feature: "batch-processing", proposal: "Add watermark table to track last successfully aggregated timestamp; use for deduplication on retry/overlap", blocks_current_task: false)` — WHY: Agent realized a solution (idempotent batch windows) but it's not in the original task. They're signaling a need for additional infrastructure before their task can be truly correct.

### Verbs Used

- `detect_data_race_condition` (report race condition / data integrity issue)
- `request_prior_task_completion_check` (verify prior task actually works)
- `propose_idempotency_tracking` (suggest additional requirement to make task robust)

---

## Simulation #25

**Project:** Real Estate Listing Aggregator (Go/Templ)
**Feature:** analytics-dashboard
**Discipline:** frontend
**Task:** Build analytics dashboard showing listing views, clicks, conversion funnel, and market heat by region
**Complication:** Task description contradicts codebase

### Walkthrough

Agent reads the task. It says: "Build dashboard using our existing metrics service API at `/api/v2/metrics`". Agent finds the dashboard component template at `src/pages/analytics/dashboard.templ` and calls the metrics API.

The API returns data with fields: `listing_id`, `views`, `clicks`. Agent checks the task acceptance criteria: "show conversion funnel: views → clicks → contacts → showings → offers."

Agent realizes the API doesn't have `contacts`, `showings`, or `offers` fields. They check the API docs. The `/api/v2/metrics` endpoint only tracks surface-level analytics (views/clicks).

Agent digs deeper. There's a `/api/v3/metrics` endpoint in the codebase that's NEW and has all the fields needed. But the task explicitly says to use `/api/v2/metrics`. The task description also says "built on existing infrastructure" which was v2.

Agent checks git blame. The task was written 3 weeks ago. The v3 endpoint was added 2 weeks ago as part of another feature. Now task and codebase are out of sync.

Agent could:
1. Use v3 (violates task description, but makes task completable)
2. Use v2 (follows task description, but can't meet acceptance criteria)
3. Implement the missing fields in v2 (scope creep — that's backend work, not frontend)

At minute 25, agent realizes the task description is stale. They check if there's a ticket to update it. There isn't.

Agent needs to: ALERT that task description is out of date, CONFIRM which API version is correct, GET PERMISSION to use v3, or GET UPDATED acceptance criteria.

### Tool Calls Made

1. → TOOL: `detect_spec_code_divergence(task_id: "analytics-dashboard", issue: "Task specifies /api/v2/metrics but acceptance criteria (conversion funnel) requires fields only in /api/v3/metrics added 2 weeks ago", conflict: "spec outdated", resolution: "needs confirmation")` — WHY: Agent found a factual contradiction. Orchestrator should flag this for product/project manager to clarify which API is actually correct.

2. → TOOL: `request_requirement_clarification(task_id: "analytics-dashboard", field: "api_version", current_spec: "v2", available: ["v2", "v3"], reason: "v3 has required fields but task says use v2; contradicts acceptance criteria")` — WHY: Agent can't proceed without knowing which requirements are binding. Orchestrator should escalate to get a clear answer.

3. → TOOL: `record_task_metadata_stale(task_id: "analytics-dashboard", written_date: "2025-01-15", codebase_changed: "api-v3-launch", last_reviewed: null)` — WHY: Agent wants to log that the task hasn't been reviewed since a significant codebase change. Orchestrator can use this to trigger periodic task review.

### Verbs Used

- `detect_spec_code_divergence` (report contradiction between task spec and current codebase)
- `request_requirement_clarification` (ask for human clarification when requirements conflict)
- `record_task_metadata_stale` (flag that task description needs review)

---

## Unique Verbs Discovered Across All Simulations

### Blocking/Escalation Verbs
- `escalate_for_human_review` — task cannot proceed; human judgment required
- `request_architecture_guidance` — need design decision approval before continuing
- `request_requirement_clarification` — requirements are contradictory; need clarification
- `request_prior_task_completion_check` — verify that a dependency actually works

### Detection/Reporting Verbs
- `flag_incomplete_dependency` — prior task marked done but is actually incomplete
- `report_dependency_version_mismatch` — library version conflict blocking task
- `detect_data_race_condition` — logic bug / data integrity issue discovered
- `detect_spec_code_divergence` — task description contradicts current codebase
- `flag_design_gap` — spec promises something that implementation can't support

### Metadata/Tracking Verbs
- `request_task_context_update` — update task metadata (dependencies, description) based on new info
- `propose_task_split` — suggest that one task should be decomposed into multiple
- `propose_idempotency_tracking` — propose new infrastructure/pattern to make task robust
- `record_version_incompatibility` — log breaking change from prior task for visibility
- `record_task_metadata_stale` — flag that task needs review due to code changes

### Verb Patterns
1. **Escalation path:** flag → request clarification → escalate to human
2. **Metadata alignment:** detect contradiction → update task context → approve continuation
3. **Dependency validation:** check prior task → flag if incomplete → escalate or propose fix
4. **Design decisions:** propose architecture → request guidance → get approval before implementing
