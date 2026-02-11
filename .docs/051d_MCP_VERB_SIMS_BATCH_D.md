# MCP Verb Discovery Simulations — Batch D

> Simulations #26–#30. Independent verb discovery from realistic agent task execution.

---

## Simulation #26

**Project:** Recipe sharing app (Flutter/Dart)
**Feature:** user-authentication
**Discipline:** frontend
**Task:** Implement biometric login for Android (fingerprint/face recognition)
**Complication:** Required API/service/secret is missing — Android Keystore setup docs reference a Google Cloud project that doesn't exist in credentials.json

### Walkthrough

Agent starts by reviewing the biometric feature spec: support fingerprint + facial recognition on Android, gracefully fall back to password. Existing codebase has a `BiometricService` stub but no integration with `local_auth` plugin. Agent reads the acceptance criteria: "User can authenticate without password using device biometrics; fallback to PIN when no biometric sensor available."

Agent installs `local_auth` and begins wiring it into the login flow. Writes a `BiometricProvider` widget, adds state management in the auth store. Then hits the complication: the task description mentions "configure Android Keystore per security-setup.md" but that document references a Google Cloud project credential file that doesn't exist. The codebase has a placeholder `android/app/google-services.json` but it's empty or uses a different project ID than what the docs assume.

Agent needs to signal back: this is a blocking dependency. There's no point iterating further without the actual credential file. The task isn't "stuck on my skill" — it's "stuck on external resource availability." Agent digs through commit history, searches for backup configs, finds nothing conclusive. Realizes the dependency task ("set up GCP auth") was marked done but never delivered.

Agent wants to:
1. Signal that a resource is missing and block themselves
2. Mark this as blocking other work
3. Suggest which dependency task should have provided it
4. Leave a marker so orchestrator can prioritize getting that resource

Agent simulates needing to report: "This task depends on `setup-gcp-auth` being ACTUALLY complete (not just marked done). I can't proceed without `google-services.json` containing the real project ID."

### Tool Calls Made

1. → TOOL: `block_on_dependency(task_id: "biometric-android", blocking_task: "setup-gcp-auth", reason: "Missing google-services.json with valid GCP project credentials")` — WHY: Explicitly signal that progress is halted, not by code complexity, but by unmet external dependency.

2. → TOOL: `verify_external_resource(resource_type: "credential-file", path: "android/app/google-services.json", required_fields: ["project_id", "api_key"])` — WHY: Agent could proactively check if a resource exists and is valid before spending tokens on workarounds.

3. → TOOL: `pause_and_escalate(severity: "blocking", message: "GCP credentials missing. Task cannot proceed. Requires setup-gcp-auth to deliver android/app/google-services.json with real project ID.", suggested_action: "fetch-from-vault-or-regenerate")` — WHY: Distinguish between "I'm thinking" (normal pause) and "I need human intervention" (escalation).

### Verbs Used

- `block_on_dependency`
- `verify_external_resource`
- `pause_and_escalate`

---

## Simulation #27

**Project:** Internal HR portal (Next.js/TypeScript)
**Feature:** role-permissions
**Discipline:** backend
**Task:** Add support for custom role definition (admin can create roles with subset of permissions)
**Complication:** Task overlaps with work in different feature — the `notifications` feature has a parallel PR that adds its own permission-scope system, causing schema conflict

### Walkthrough

Agent begins by understanding the feature: currently roles are hardcoded (admin, manager, employee). New task: make them editable, allow admins to compose custom roles by selecting individual permissions. Acceptance criteria: "Custom roles persist to database; admins see role editor in dashboard; role changes apply to all users with that role immediately."

Agent sketches the schema: `roles` table with (id, name, description, created_by, created_at), `role_permissions` junction table. Starts writing migrations. Then checks what permission types exist—reads `src/lib/permissions.ts`. All looks good.

But then: agent finds a branch (`feature/notification-scopes`) that's been in progress for 3 weeks. That PR also adds a `permission_scopes` table for notifications-only permission granularity. It has a similar structure to what the agent is about to create. The two PRs will conflict in the database schema, and there's no coordination.

Agent realizes this is a design problem, not a code problem. Both features want to solve "permission subsets" but in slightly different ways. Agent could:
- Merge both approaches into a unified permission system (big refactor, many affected files)
- Compromise on a hybrid approach
- Choose one and deprecate the other
- Wait for the other PR to land first and adapt

But agent can't make this call unilaterally. It's an architectural decision. Agent wants to flag this collision and suggest a design meeting or coordinated merge strategy.

Agent also wants to know: "What's the status of that conflicting PR? Is it active?" to decide whether to wait or proceed.

### Tool Calls Made

1. → TOOL: `detect_schema_conflict(new_table: "role_permissions", conflict_with: "permission_scopes", branch: "feature/notification-scopes", severity: "design-conflict")` — WHY: Automatically discover and report structural conflicts that can't be resolved by simple merges.

2. → TOOL: `query_branch_status(branch: "feature/notification-scopes", info_requested: ["last-commit-date", "author", "pr-status", "blocked-by"])` — WHY: Agent needs to know if the conflicting work is active, stalled, or abandoned before deciding strategy.

3. → TOOL: `request_design_review(question: "Should role-permissions and notification-scopes be unified into a single permission-scopes system, or kept separate?", blocking_on_answer: true)` — WHY: Signal that this task cannot proceed without a design decision that involves human judgment or stakeholder input.

4. → TOOL: `suggest_dependency_ordering(task: "role-permissions-custom", depends_on: "notification-scopes-finalize", rationale: "Both modify permission architecture; notification-scopes should land first to avoid double-refactor")` — WHY: Suggest a sequential ordering to the orchestrator to avoid rework.

### Verbs Used

- `detect_schema_conflict`
- `query_branch_status`
- `request_design_review`
- `suggest_dependency_ordering`

---

## Simulation #28

**Project:** IoT sensor dashboard (Rust/Axum + HTMX)
**Feature:** analytics-dashboard
**Discipline:** fullstack
**Task:** Add real-time metrics view (CPU, memory, sensor count) with WebSocket streaming
**Complication:** Performance requirements make obvious approach unviable — naive WebSocket approach would broadcast 10K updates/second across 200+ sensors; naive database polling would lock the DB

### Walkthrough

Agent understands the requirement: show live metrics (CPU, memory, active sensor count) updating every 500ms. Current approach: poll the database every 500ms, render via HTMX swap. Works fine for 10 sensors. At scale (200+ sensors with 50Hz update rate), it's untenable.

Agent sketches options:
1. WebSocket per client (simple, but 200 concurrent clients × 50Hz = 10K messages/sec server-side)
2. Aggregate metrics in Redis, subscribe to updates (better, but needs Redis and pub/sub logic)
3. In-process aggregation thread (fast, but tightly couples server state)
4. Time-series database like InfluxDB (robust, but adds infrastructure)

Agent starts coding option 1 (naive WebSocket) as a proof of concept. Writes `ws/metrics.rs` with a broadcast channel, spawns a task to accumulate sensor readings. Gets 100 lines in, then does a rough calculation: 200 concurrent clients × 50Hz sensor update rate × 4KB per message = 40 MB/sec network load. Database would need custom indexes just to keep up with query rate.

Agent realizes: the acceptance criteria don't specify the actual performance constraint. "Real-time" could mean 10Hz or 1Hz. Agent also needs to know: how many concurrent clients are expected? What's the acceptable latency? Network budget?

Agent has three paths forward:
- Implement naive version, accept it won't scale (works for MVP)
- Implement with Redis (needs new infrastructure, agent can't provision it)
- Ask: what are the actual performance constraints?

Agent could also suggest: maybe the dashboard should show aggregated metrics (per-sensor-group) instead of per-sensor, which would reduce update volume by 100x.

Agent wants to report this dilemma and get clarity on constraints before continuing, rather than shipping something that will need a major refactor once users hit scale limits.

### Tool Calls Made

1. → TOOL: `flag_performance_concern(task: "metrics-realtime", concern_type: "architecture-mismatch", details: "Naive WebSocket approach requires ~40MB/sec throughput at scale. Need clarification: expected concurrent users? Acceptable latency? Aggregation strategy?", severity: "high")` — WHY: Signal that the task has a performance trap that needs stakeholder input before implementation.

2. → TOOL: `request_acceptance_criteria_clarification(task: "metrics-realtime", missing_info: ["concurrent-client-count", "update-frequency-requirement", "latency-budget-ms"], blocking: true)` — WHY: Rather than guess, explicitly ask for missing acceptance criteria details.

3. → TOOL: `suggest_alternative_approach(original_task: "metrics-realtime", alternative: "Implement aggregated metrics (per-sensor-group) instead of per-sensor; reduces update volume 100x and fits naive approach", estimated_impact: "Reduces scope to 1-day task instead of 3-day architectural work", requires_approval: true)` — WHY: Propose scope reduction to unblock the task while respecting constraints.

4. → TOOL: `dependency_request(needs: "redis-infrastructure", reason: "Production-ready real-time metrics require Redis pub/sub for sub-linear scaling", who_provides: "devops", urgency: "blocker-if-choosing-redis-path")` — WHY: Explicitly request infrastructure provisioning from the right discipline.

### Verbs Used

- `flag_performance_concern`
- `request_acceptance_criteria_clarification`
- `suggest_alternative_approach`
- `dependency_request`

---

## Simulation #29

**Project:** Social media scheduler (Python/FastAPI)
**Feature:** notifications
**Discipline:** backend
**Task:** Implement email notification service (transactional emails for schedule confirmations, post failures)
**Complication:** Third-party library has breaking change — `sendgrid-python` v4.0 changed auth signature, but Dockerfile still pins v3.x; upstream task didn't update either

### Walkthrough

Agent starts by reviewing the notifications feature: we need to send transactional emails. Task is to add email delivery for two events: "schedule confirmed" and "post failed." Acceptance criteria: "Emails sent within 5 seconds of event; include relevant context (post ID, scheduled time); unsubscribe link present."

Agent checks existing dependencies in `pyproject.toml`: `sendgrid==3.10.0`. Quick research: v4.0 has breaking changes in the `Mail` API and auth. Docs say migrate by changing auth from tuple to keyword args. Seems straightforward.

Agent writes the email service:
```python
from sendgrid import SendGridAPIClient
from sendgrid.helpers.mail import Mail

mail = Mail(
    from_email="noreply@scheduler.app",
    to_emails=user.email,
    subject="Post scheduled",
    html_content=html
)
client = SendGridAPIClient(sg_key=SENDGRID_API_KEY)
client.send(mail)
```

Runs locally, works fine. But then: agent notices the `Dockerfile` in `deployment/` still pins `sendgrid==3.10.0`. Checks the upstream dependency task that was supposed to update SendGrid—it's marked "done" but the PR was merged without actually bumping the version or updating the Dockerfile. The code would work in local dev (because agent's venv has v4.x) but fail in production (Dockerfile installs v3.x).

Agent realizes: this is a trap. The agent code is correct for v4, but production will run v3, causing auth errors at runtime. Agent could:
1. Downgrade to v3 API (undo their work, use deprecated auth style)
2. Update Dockerfile + `pyproject.toml` to v4 (violates single-responsibility: this task shouldn't care about dependency versions)
3. Wait for upstream to actually deliver (but it was marked done and probably won't revisit)

Agent's code is correct, but the environment is inconsistent. Agent wants to flag this: "My code assumes SendGrid v4 per latest docs, but Dockerfile uses v3. These are incompatible."

### Tool Calls Made

1. → TOOL: `detect_dependency_version_mismatch(service: "sendgrid", code_uses: "v4.x-api", container_pins: "v3.10.0", error_likely: "authentication-failure-in-production", blocking: true)` — WHY: Catch environment/code mismatches before deployment.

2. → TOOL: `verify_upstream_completion(task_id: "upgrade-sendgrid-to-v4", marked_done: true, check: "pyproject.toml and Dockerfile actually updated?", result: "incomplete")` — WHY: Detect when "done" tasks didn't actually deliver what was assumed.

3. → TOOL: `request_environment_sync(issue: "Dockerfile and code are using different major versions of the same library", action_required: "Update Dockerfile to sendgrid==4.x OR revert code to sendgrid v3 API", who_owns: "devops-or-dependency-maintainer")` — WHY: Flag that the environment is out of sync and needs coordination.

4. → TOOL: `suggest_rework_scope(current_approach: "Use SendGrid v4 API", alternative: "Downgrade to v3 API to match Dockerfile pin", trade_offs: "v3 is deprecated but matches production environment; v4 requires Dockerfile update that's not this task's responsibility")` — WHY: Present the choice clearly so agent can decide on the path forward.

### Verbs Used

- `detect_dependency_version_mismatch`
- `verify_upstream_completion`
- `request_environment_sync`
- `suggest_rework_scope`

---

## Simulation #30

**Project:** CLI database migration tool (Go)
**Feature:** migration-system
**Discipline:** devops
**Task:** Add dry-run mode (preview migrations without applying them)
**Complication:** Existing abstraction doesn't support what's needed — `MigrationRunner` was designed for serial execution only; dry-run needs to fork state without side effects

### Walkthrough

Agent starts by reading the migration system architecture: `MigrationRunner` struct holds database connection, execution log, and state. Current flow: read migration file, parse SQL, execute, log, move to next. Task: add `--dry-run` flag that previews all pending migrations without actually executing them.

Acceptance criteria: "User runs `migrate --dry-run`; output shows SQL that would execute, in order, without modifying database; exit code 0 if no errors detected."

Agent thinks straightforward: just add a flag, skip the `Execute()` call, print the SQL. But there's a catch: some migrations have conditional logic—the SQL depends on what's in the database. Example:

```go
-- 003_add_index.sql
-- Skip if index already exists
CREATE INDEX idx_users_email ON users(email) IF NOT EXISTS;
```

Another migration might have:

```go
-- 004_drop_legacy_column.sql
-- Only drop if the column still exists (was added in 003)
ALTER TABLE users DROP COLUMN legacy_field;
```

In dry-run, agent can't actually check if the index exists (no database changes). So the dry-run output might claim "drop legacy column" when in reality that step would fail if 003 was already applied.

Agent looks at `MigrationRunner` source: it's a struct designed for stateful serial execution. To support dry-run, agent would need to either:
1. Clone the database state for dry-run (expensive, might not be allowed by database permissions)
2. Refactor `MigrationRunner` to separate "analyze" from "execute" (big refactor, touches 10+ migration files)
3. Create a second `DryRunRunner` (code duplication)
4. Hack: use transaction semantics (run in a transaction, then rollback) — but this might trigger locks or side effects

Option 1 (clone) is ideal but requires a new permission that might not exist. Option 2 is clean but large. Option 3 is quick but violates DRY. Option 4 is fragile.

Agent wants to discuss this with the architect: "The `MigrationRunner` wasn't designed for this. I can implement it, but it'll be hacky. Should I refactor the abstraction to support both execution modes, or accept a limited dry-run?"

Agent also wants to know: "Do we have database permissions to clone for dry-run checks?"

### Tool Calls Made

1. → TOOL: `architecture_conflict(feature: "migration-system", task: "dry-run-mode", conflict: "MigrationRunner designed for serial execution only; dry-run needs stateless analysis or transaction rollback", severity: "medium")` — WHY: Explicitly flag that the abstraction needs to change, not just the code.

2. → TOOL: `request_permission_check(resource_type: "database", operation: "clone-for-dry-run-verification", needed_for: "Accurate dry-run with conditional SQL", blocking_if_denied: true)` — WHY: Discover whether an infra assumption is valid before designing around it.

3. → TOOL: `propose_refactoring(current_design: "MigrationRunner with serial-only execution", proposed_design: "Split into Analyzer (stateless preview) + Executor (stateful apply)", scope_impact: "Medium refactor; affects 10+ migration files", benefit: "Enables dry-run, testing, rollback previews", decision_needed: true)` — WHY: Propose an abstraction change and ask for approval before committing to implementation.

4. → TOOL: `suggest_workaround(constraint: "Don't want to refactor abstraction", approach: "Implement dry-run via transaction+rollback; note: may trigger locks, not 100% accurate for conditional SQL", limitations: "Output may differ from actual execution if conditions change during deployment")` — WHY: Offer a pragmatic short-term path with known trade-offs.

5. → TOOL: `document_design_decision(decision: "Dry-run via transaction rollback instead of full refactor", rationale: "Accepted trade-off: conditional migrations may show inaccurate preview", record_for: "Future refactoring or feature enhancement")` — WHY: Log the architectural decision so future maintainers understand the compromise.

### Verbs Used

- `architecture_conflict`
- `request_permission_check`
- `propose_refactoring`
- `suggest_workaround`
- `document_design_decision`

---

## Summary: Verbs Discovered (Batch D)

**Resource & Dependency Management:**
- `block_on_dependency` — task is halted by incomplete upstream work
- `verify_external_resource` — check if external file/credential exists and is valid
- `pause_and_escalate` — distinguish normal pauses from issues needing human intervention
- `dependency_request` — explicitly request infrastructure/services from other disciplines

**Conflict Detection & Design:**
- `detect_schema_conflict` — discover structural/database conflicts with concurrent work
- `query_branch_status` — check status of related work to decide sequencing
- `request_design_review` — flag decision points that require stakeholder input
- `suggest_dependency_ordering` — recommend task ordering to avoid rework
- `architecture_conflict` — abstraction can't support what's needed
- `propose_refactoring` — suggest refactoring and ask for approval

**Performance & Constraints:**
- `flag_performance_concern` — warn that naive approach won't meet scale requirements
- `request_acceptance_criteria_clarification` — ask for missing details before proceeding
- `suggest_alternative_approach` — propose scope reduction or redesign
- `request_permission_check` — discover whether infra assumption is valid

**Version & Environment Consistency:**
- `detect_dependency_version_mismatch` — code/container using different versions
- `verify_upstream_completion` — check if "done" task actually delivered
- `request_environment_sync` — flag out-of-sync environments needing coordination
- `suggest_rework_scope` — present trade-offs between implementation paths

**Documentation & Decisions:**
- `document_design_decision` — log architectural compromises for future reference
- `suggest_workaround` — offer pragmatic short-term path with known limitations

