# Blind Verb Simulations - Generation G

Simulations of Claude Code encountering friction in unfamiliar codebases. Each combines a random project, feature, discipline, and complication. Friction points are bolded.

---

## #11: Social Media Scheduler – Webhook System (Backend)

**Project:** Social media scheduler (Python/FastAPI)
**Your assignment:** Implement webhook delivery retry logic with exponential backoff. Webhooks failing after 3 retries should be marked failed and logged to audit trail.
**These tasks are already done:** "Webhook event schema design", "Database migration for webhook_events table"
**These tasks come after yours:** "Webhook signature verification", "Webhook dashboard UI"
**Notes from previous work:** "Webhook events table uses JSON payload column. The audit-log feature requires integration point we haven't designed yet."

---

I start by mapping the codebase structure. Find the FastAPI app, locate the webhook_events table schema. Migration looks clean—timestamp, payload, status columns present. Good.

Start implementing the retry handler. Create a background task that queries for failed webhooks, orders by created_at, applies exponential backoff formula. Standard stuff. But then I hit it: **I need an API key and webhook endpoint URL to test this locally, and I don't see credentials in the repo—just placeholder env vars. Do I hardcode a fake endpoint? Do I mock the HTTP client? The task says "webhooks failing" but I can't actually trigger real failures without a real service.**

Push ahead. Implement the retry logic assuming I'll test it later. Three retries with 2^n second delay: 2s, 4s, 8s. Mark webhook_events.status as "failed" after exhaustion. Now the complication hits: **the task says "logged to audit trail" but the audit-log feature hasn't been implemented yet. The previous notes say there's no integration point. Do I create a generic audit_logs table? Do I wait? Do I invent a contract?**

I search for any existing audit code. Nothing. Check the task list again—audit-log is marked done? No, that's a different task. I realize **I don't know what the audit-log feature actually needs. I can't even see its acceptance criteria or schema because it hasn't been built. I'm supposed to integrate with something that doesn't exist yet, and I have no way to know if my integration point will be compatible.**

Worst case: I hard-code a webhook audit entry directly to a new audit_logs table, knowing it will probably need refactoring. Add a comment. Move on. Mark webhook_events with `retried_at`, `failed_at` timestamps. Write the retry handler. It compiles.

But now **I realize I can't actually run this code to verify the retry logic works because I can't mock the HTTP client failure without understanding the existing HTTP client abstraction—is it requests? httpx? How are they handling timeouts now?** I search for existing webhook send code. Find a stub. It uses `httpx.AsyncClient`. Good, I can patch that in tests.

Except **I need to know where tests go and what the testing pattern is. Do they use pytest? fixtures? How deep do they test retries?** I could search for test files, but every minute I spend pattern-matching is a minute I'm not building. I just create a basic test file, import unittest.mock, and call it done. Commit message will be vague because the whole retry chain feels incomplete without actual failure injection.

---

## #12: Personal Finance Tracker – Offline Sync (Frontend)

**Project:** Personal Finance tracker (SvelteKit/TypeScript)
**Your assignment:** Implement offline-first sync for expense records using IndexedDB. When online, sync local changes to server; when offline, queue operations.
**These tasks are already done:** "IndexedDB schema design", "Auth token refresh on reconnect"
**These tasks come after yours:** "Conflict resolution UI", "Offline indicator badge"
**Notes from previous work:** "Token refresh works but only if you manually trigger it. Sync should auto-detect network status."

---

I begin by opening the SvelteKit project. Find the IndexedDB schema—expense table with id, category, amount, date, synced_at. Good. Write a Svelte store that wraps IndexedDB operations. Use `navigator.onLine` to detect connectivity.

Start simple: writable store that batches pending operations in an array. When online, POST them to the API. Catch responses, mark records as synced_at = now(). When offline, add to the queue. When online again, flush the queue.

Then I realize: **what if the user adds two expenses with the same timestamp while offline, then a third one comes in from another device? When I sync, do I send all three, or do I need to check server first?** The task says "sync local changes" but doesn't say if there's a conflict resolution. I search for any conflict resolution code. Find nothing. The notes say conflict resolution comes AFTER my task, so **I'm expected to build a sync system that doesn't handle conflicts, which means it will corrupt data on multi-device scenarios.**

Push ahead. Implement the basic sync. But now the real problem: **when the token refresh happens (previous task), will my sync requests re-use the old token? The notes say token refresh only works if triggered manually. Do I need to re-read the token before each request? Is there a way to hook into the auth store?** I search for the auth module. It's a global store. Try reading token from store in sync function. Add a check—if request fails with 401, manually call the refresh function, retry.

Now the race condition: **what if the user adds an expense, goes offline, adds another expense, comes back online, and the first sync request is slow? The second request might race the first, creating duplicate IDs. IndexedDB is local, so I can assign my own UUIDs, but the server needs to be idempotent about what I send.** I check the API spec. No idempotency key field. **Do I add one? Do I assume the API deduplicates by (user_id, timestamp, amount)? I can't make that call without knowing the backend's design.**

Add a uuid field to the queue locally, include it in requests, hope the backend respects it. Write a test that adds offline, goes online, halfway through sync goes offline again. Can I test the race condition? **I can write a test, but I'd need to mock the network latency, which means controlling async timing, which means I need to know the testing framework's preferences.** I use vitest's `vi.useFakeTimers()`. It probably works.

Realize **I've built a sync system that assumes the backend handles conflicts and idempotency, but I have no guarantee it does. I have no way to leave a note saying "please implement idempotency keys" that someone will actually read.** Write a commit message. Move on.

---

## #13: IoT Sensor Dashboard – Analytics Dashboard (Fullstack)

**Project:** IoT sensor dashboard (Rust/Axum + HTMX)
**Your assignment:** Build analytics dashboard showing sensor aggregates (min/max/avg) per hour. Render with HTMX for real-time updates without full-page reloads.
**These tasks are already done:** "Sensor ingestion pipeline", "Database time-series schema"
**These tasks come after yours:** "Custom time range picker", "Export to CSV"
**Notes from previous work:** "Sensor data arrives as 1000s of events per minute. The ingestion pipeline batches them hourly. Beware: aggregates must account for late-arriving data."

---

I read the schema. Sensors table has id, name, type. Sensor_readings table has timestamp, sensor_id, value. There's likely an aggregate table, let me search. Found it: sensor_aggregates with timestamp, sensor_id, min, max, avg, count. Good, someone already built the aggregation step.

Start building the HTTP endpoint. Query sensor_aggregates for the last 24 hours, group by hour, format as JSON. Then write an HTMX template that fetches this endpoint every 30 seconds, swaps the tbody of the table.

Then I look at the notes: **"late-arriving data". If sensor readings come in out-of-order and the aggregate table gets updated after the dashboard already fetched, the displayed numbers become stale. How stale is acceptable? Is there a grace period? Should the dashboard show "last updated at" with a warning if aggregates are older than 5 minutes?**

I search for any documented SLA on data freshness. Nothing. Check if there's a last_updated timestamp on the aggregates table. No. **I don't know if I should add one, if it would break the aggregation job, or if it's even possible with hourly batching. Do I ask the backend team? Is there a backend team?**

Implement the basic dashboard. Fetch aggregates, render a table with Sensor name | Min | Max | Avg. Use HTMX hx-trigger="every 30s" to poll. Then I realize: **the time-series aggregation probably uses bucketing logic (like ClickHouse or Postgres time_bucket). If I query "last 24 hours" at 3:45 PM, do I get the 3:00-4:00 bucket already closed, or is it still open? If it's open, my dashboard shows incomplete aggregates. If it's closed, I miss the current hour's data.**

Check the aggregation code. It's a background job that runs at the top of each hour. So at 3:00:15, it processes 2:00-3:00 data. My 3:45 PM query gets buckets up to 3:00, missing 45 minutes of current data. **Should the dashboard also show the current in-progress hour with real-time aggregates, or just historical? The task doesn't specify.**

Assume historical only. Build the dashboard. Then the real problem: **the existing abstraction is the aggregates table, which is hourly buckets. But what if someone wants a 15-minute view? Or a daily view? The hour-locked schema doesn't support that.** Could query raw readings and aggregate on-the-fly, but the notes warned about 1000s of events per minute—that's expensive. **Do I build another aggregates table at 15m granularity? Do I create a flexible aggregation function? Do I assume the schema is fixed and document that 15m views are out of scope?**

Add a comment about granularity, hard-code the 1-hour view, write the HTMX template. Test with curl. It works for the happy path. But **I have no way to test the late-arriving data edge case without a time-machine or a way to inject past data, and I don't know if the aggregation job has a mechanism for that.**

Commit. Note the granularity limitation. Move on.

---

## #14: Learning Management System – Role Permissions (Security)

**Project:** Learning management system (Laravel/PHP)
**Your assignment:** Enforce role-based access control for course modules. Students can view, instructors can edit, admins can delete. Implement middleware that checks role before each action.
**These tasks are already done:** "User roles table migration", "Instructor course assignment"
**These tasks come after yours:** "Permission audit logging", "Role management UI"
**Notes from previous work:** "Roles table exists (student, instructor, admin). Course assignments are in place. Watch out: there's also an old permissions table that was never fully deleted."

---

I start by mapping roles and permissions. Laravel convention is Gate::define() or Policy classes. Look at the User model. It has a role_id field. Good. Check the roles table: student, instructor, admin. Straightforward.

Write a CourseModulePolicy. Define rules: students can view only if enrolled, instructors can edit only if assigned to course, admins can always delete. Use Laravel's authorize() in controllers. Standard pattern.

Then I notice the notes: **"old permissions table that was never fully deleted". Is this table still being read anywhere? Is it conflicting with the new role-based system? If I implement the policy and someone tries to do something forbidden, which system wins—the policy or the old table?**

Search the codebase for references to the old permissions table. Find it in the migrations but not in any model. Search for queries directly to it. Nothing. But **I can't be sure it's dead code. What if there's legacy code that references it in a way I didn't search for? What if some feature branch is still using it?**

Read the notes again: "never fully deleted". That phrase suggests it's hanging around but not actively used. Maybe it's a landmine. Implement the policy anyway. If something breaks, I'll know.

But then the real problem: **the task description says "enforce role-based access control", but it doesn't say what happens when an unauthorized user tries to act. Do they see a 403 page? Are they redirected? Do they see a silent failure? The codebase probably has conventions, but I need to find them.**

Search for existing authorization failures. Find some Gate checks in blade templates that hide buttons if unauthorized. Find some controller checks that throw an AuthorizationException. Different patterns. **Do I use both? Do I standardize on one? Do I respect the existing inconsistency?**

Implement the middleware using the existing AuthorizationException pattern since it's in the controllers. Write a middleware that catches it and returns 403. Add the middleware to the course routes.

Then I realize: **the task says "enforce", but I haven't written any tests. How do I know the middleware actually catches unauthorized access? I'd need to create test users with different roles, enroll them in courses, then try forbidden actions. But the notes don't mention a test database or seeding strategy. Do I write factories? Do I use the Laravel testing helpers?** I write a basic test using PHPUnit and Laravel's test traits. Mock the roles. Call the endpoint. Assert 403. It probably works.

The last friction: **the old permissions table is still in the database schema. If someone runs migrations fresh, they'll get an old table that serves no purpose, creating confusion. Should I write a migration to drop it? But that's a separate concern from "enforce role-based access". The task scope doesn't include database cleanup.** Leave it. Add a TODO comment. **This feels dirty—I'm leaving technical debt intentionally, but I have no mechanism to escalate it or schedule it.**

Commit.

---

## #15: Multiplayer Game Lobby – Notifications (Testing)

**Project:** Multiplayer game lobby (Elixir/Phoenix)
**Your assignment:** Write comprehensive tests for the notification system. Verify that players receive notifications on match-found, friend-request, and chat messages. Include tests for notification delivery failures and retries.
**These tasks are already done:** "Notification system core implementation", "Notification delivery retries"
**These tasks come after yours:** "Notification preferences UI", "Email digest feature"
**Notes from previous work:** "Notification core is implemented. The retry logic was marked done but hasn't been fully tested. Be careful."

---

I start by finding the notification system. Elixir project structure: lib/ has Phoenix modules. Find the Notification context. It has functions: send_match_found, send_friend_request, send_chat_message. Good entry points.

The notes say retry logic is "marked done but hasn't been fully tested". So I need to write tests for it. Let me search for the retry implementation. Find a Notification.Retry module with backoff logic. Looks correct on paper. But **how do I actually test that a notification was retried? Do I mock the underlying delivery mechanism? Do I need to understand how Elixir/OTP handles job queues?**

I search for tests in the existing notification code. Find very basic tests: just calling the functions and checking they return ok. No tests for failures or retries. **The retry module exists but it's untested. This means the code might be broken and nobody knows.**

Write a test that mocks the delivery failure, calls send_match_found, verifies it retries. Use Mox for mocking. But then **I realize I don't know how the retry system is triggered. Is it a GenServer? A separate job queue? Oban? Do I need to trigger some background worker, or does the test directly exercise the retry code?**

Read the retry module more carefully. It has a retry_later() function that... schedules something? The implementation uses Process.send_after(). So it's using Erlang's built-in timer. That's clever. But **I can't use that in tests because real timers make tests slow. I'd need to mock the timer or use Mox to replace it with an instant version.**

Write a test that directly calls the retry function with a failure scenario. Verify the retry count increments. But **I'm not really testing the full path—I'm testing the retry module in isolation, not the integration where send_match_found fails and retry_later gets called.**

Search for where the retry is actually invoked. Find it in the send functions: wrap the delivery call in try/catch, call retry_later on failure. So the integration is there. But **I need to test the integration—that a failed send actually triggers the retry. That means I need a way to trigger a delivery failure in the test, which means mocking the delivery module, which means understanding the module's interface.**

Look at the delivery code. It has HTTP requests to a push notification service. Mock it using Mox. Write a test: mock delivery to fail once, verify it's called twice (original + one retry). Then **I realize I'm testing the retry mechanism in a way that assumes exactly one retry. The notes say retry logic was implemented, but how many retries? Exponential backoff? Max attempts? I have no spec.**

Search for constants. Find max_retries = 3. Backoff = 2^attempt seconds. Good. Now write a test that fails three times and verifies it stops retrying. But **the backoff is in seconds, and now I'm back to the timer problem. If the test waits 2, 4, 8 seconds for retries to execute, the test takes 14+ seconds. That's unacceptable.**

Use Mox's async option? No, that doesn't help with timers. Mock Process.send_after? That might work, but **I'd be testing that send_after was called with the right delay, not that the retry actually happens. I'd be mocking out the core mechanism I'm supposed to test.**

The friction hits hard: **I'm writing tests for a feature that uses Erlang timers, which don't work well in synchronous tests. I could refactor the code to use a test-friendly queue, but that's not the task—the task is to test the existing code. I have no tool to make Erlang timers fast in tests without either mocking them away (making the test worthless) or actually waiting (making the test slow).**

Compromise: write tests for the retry logic in isolation (mocking the delivery), and write a separate integration test that uses async and actually waits for one retry cycle (a few seconds). It's not perfect, but it covers the main paths. **I feel like I'm testing around the problem rather than testing the actual problem.**

Write the tests. They pass. Commit. But I add a comment: "Integration test doesn't cover all retry attempts due to timer constraints. Consider refactoring retry mechanism to use a job queue for testability." **This comment documents the friction, but I have no confidence anyone will act on it.**
