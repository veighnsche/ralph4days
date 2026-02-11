# 053: Blind Verb Simulations (Batch F)

Five more scenarios testing what happens when a developer tools set is limited to read/write/search/terminal.

---

## Simulation #6: Recipe Sharing App (Flutter/Dart) — Notifications Feature with Missing Secret

**Project:** Recipe sharing app (Flutter/Dart)
**Your assignment:** Implement the push notification system using Firebase Cloud Messaging. Add device token registration, APNs certificate configuration for iOS, and test the end-to-end notification payload delivery.
**These tasks are already done:** Push notification service layer scaffolded in `lib/services/notifications.dart`, Firebase project created and billing enabled
**These tasks come after yours:** Mobile app store submissions (iOS/Android), analytics integration for notification open rates
**Notes from previous work:** Team moved credentials to GitHub Actions secrets last sprint. APNs certificate expired and reissue is pending IT approval.

---

I start by searching the codebase for how Firebase is currently being initialized. Looking for `firebase_options.dart` or similar. Found it. The file references `FIREBASE_API_KEY` which should be injected at build time.

I check `lib/services/notifications.dart` — it's just function stubs with TODO comments. The scaffolding is there but there's no actual token registration logic yet. I look at the pubspec for dependencies: firebase_messaging is included, good.

Next I try to find where secrets are loaded. I grep for environment variable usage in the build scripts. I see references to `.env` in `.gitignore` but the file doesn't exist locally. I check the GitHub Actions workflow — there's a `firebase_config.json` being echoed into the build directory during CI.

**I need that JSON file to test locally, but I have no way to access the GitHub Actions secrets. I can't decrypt them. I don't have the APNs certificate either because IT hasn't issued it yet.** I can write the code structure that expects these values, but I can't validate that the integration actually works. Do I hardcode placeholder values? Do I create a test certificate? Do I wait? **There's no way to communicate to whoever issued that certificate that I'm blocked and need it now.**

I write the Dart code for device token registration anyway — `FirebaseMessaging.instance.getToken()` in the scaffold, then save it to Firestore. I make reasonable guesses at the structure based on Firebase documentation. But I can't test the actual handshake.

Then I remember the complication: **the APNs certificate is expired and reissue is pending IT approval.** I search the codebase for any certificate files. Nothing. I check the docs. Nothing. I look at the CI logs that exist locally — they're truncated and don't show certificate steps. I realize that even if I had the build environment set up correctly, the certificate doesn't exist yet, so the iOS build would fail at the signing step.

**I can write working Dart code, but I cannot deliver a working end-to-end integration because the infrastructure (a certificate) is stuck outside the codebase and outside my ability to fix.** I could add documentation explaining what will unblock this. I could write a test that mocks Firebase. But the actual requirement — "test the end-to-end notification payload delivery" — cannot be satisfied until IT responds. **There's no persistent way to track that this task is actually unblocked only once the certificate is ready.**

---

## Simulation #7: IoT Sensor Dashboard (Rust/Axum + HTMX) — API Rate Limiting with Contradictory Design

**Project:** IoT sensor dashboard (Rust/Axum + HTMX)
**Your assignment:** Implement per-user API rate limiting with sliding-window algorithm, configurable thresholds, and clear feedback when limits are exceeded. Ensure the approach works with horizontal scaling across multiple Axum instances.
**These tasks are already done:** Redis integration layer merged, database schema for rate-limit events created, authentication middleware installed
**These tasks come after yours:** Admin dashboard showing rate-limit violation patterns, automated alerting for abusive clients
**Notes from previous work:** Previous dev noted that "all requests should complete in <50ms even at 99th percentile." Sensor data ingest is bursty — many devices push every 30 seconds.

---

I start by reading the existing authentication middleware to understand where rate-limit checks would go. It's a simple extraction of user_id from JWT. Good insertion point.

I look at the Redis layer. It's already initialized with a `RateLimit` struct. But when I read the actual struct definition, I see it tracks `requests_per_second`. When I search the codebase for requirements, I find a comment in the task description: "configurable thresholds." But I also find an issue in the schema: there's no `threshold` column in the rate_limit_config table. There's only hardcoded constants.

**The code is already written as if thresholds are hardcoded, but the task says they should be configurable. These two statements are contradictory.** Do I ignore the existing code and build a new configurable system? Do I follow the codebase and make thresholds fixed? Do I try to retrofit the database? **I have no way to ask: which one does the project actually want?**

I read the previous notes: "all requests should complete in <50ms even at 99th percentile." Then I think about the Redis check — every request will need a round trip to Redis to update the sliding window. That's a network call. At scale, that might violate the latency requirement. But sliding-window is the only algorithm that gives accurate rate limiting.

I search for how other parts of the system handle latency. I find a caching layer, but it's for data, not for control plane decisions. I think about using local bucketing + eventual consistency with Redis, but that creates a race condition: two Axum instances might both approve requests that, globally, exceed the limit.

**The architectural requirement (horizontal scaling) and the performance requirement (sub-50ms) are pulling in opposite directions. Accurate distributed rate limiting either adds latency or becomes slightly inaccurate. There's no design document explaining which tradeoff was chosen.** I could write code that picks one option arbitrarily, but then if the next task needs the opposite, I've built in the wrong direction.

I decide to implement the strict Redis-based approach since that matches the codebase pattern, and I add a TODO about performance. But **I have no way to validate whether this will actually meet the latency requirement without running it against real production load.**

---

## Simulation #8: Personal Finance Tracker (SvelteKit/TypeScript) — File Upload with Broken Abstraction

**Project:** Personal finance tracker (SvelteKit/TypeScript)
**Your assignment:** Add CSV import for transaction history. Parse CSV rows, validate against transaction schema, and upsert records into the database. Handle malformed files gracefully.
**These tasks are already done:** File upload endpoint `/api/import/upload` created, transaction schema defined, CSV parsing library (PapaParse) installed
**These tasks come after yours:** Duplicate detection and merge UI, reconciliation status tracking
**Notes from previous work:** Previous attempt used `fs.readFileSync()` which worked locally but broke in production (serverless environment). Existing file handling uses in-memory buffers only.

---

I look at the `/api/import/upload` endpoint. It's expecting a FormData file and has a basic structure. I check how it's being called from the frontend — there's a form component that does a fetch POST.

I try to understand the existing file handling pattern by searching for other file uploads in the codebase. I find that user avatar uploads go through a different endpoint. Let me read that one. It uses in-memory buffers and directly writes to a cloud storage service (looks like S3). It never touches the filesystem. Good pattern.

Now I implement the CSV import using the same pattern. PapaParse has a streaming API for large files. I set up the CSV parsing in the `/api/import/upload` endpoint, validate each row against the transaction schema, and upsert them.

But when I test locally with a sample CSV, I realize something: the transaction schema has a field `category_id` which is a foreign key. The CSV doesn't have category IDs, it has category names like "groceries", "rent", etc. I search the codebase for how categories are stored. I find them in the database.

**Now I need to look up category names to IDs during the import. But the abstraction that currently exists assumes categories are already known at parse time.** Let me look at the transaction creation endpoint to see how it handles category resolution. It has a separate API call: user provides category name, a query returns the ID, then the transaction is created.

So the pattern in the codebase is: client calls one endpoint to resolve category names, then uses the IDs in the next call. But my CSV importer is running server-side. If I follow that pattern, I'd need to query the database inside the CSV parsing loop, once per row. For a CSV with 500 transactions, that's 500 queries.

**The abstraction (separate query for category resolution) was designed for the UI, not for batch operations. I need to change it or work around it.** I could batch the queries into a single lookup before the loop. But that means deviating from the existing pattern. Or I could do 500 queries and accept the performance hit.

I decide to do a single batch lookup of all unique category names, then build a map in memory. But now I realize I'm making a decision about the architecture without being able to discuss it. **If the next dev comes to add another batch importer (for accounts, for budgets), will they follow my pattern or the original pattern? There's no persistent place to document why I deviated.**

I implement it and add a comment in the code. But **a comment in code is lost when that file gets refactored. There's no shared knowledge base.**

---

## Simulation #9: Code Review Bot (TypeScript/GitHub API) — Dependency Task Left Incomplete

**Project:** Code review bot (TypeScript/GitHub API)
**Your assignment:** Add inline code suggestions that automatically commit on user approval. When a reviewer approves a suggestion, the bot should apply the code change, create a commit, and push it to the PR branch.
**These tasks are already done:** GitHub API client layer completed, suggestion detection engine working, approval comment parser implemented
**These tasks come after yours:** Dashboard showing suggestion acceptance rate, machine learning model for suggestion quality scoring
**Notes from previous work:** Team decided that direct commit-and-push is safer than editing through the UI — less chance of merge conflicts.

---

I start by reading the GitHub API client layer. I see methods like `getSuggestions()`, `getApprovals()`, `createCommit()`. I also see a reference to `pushToRemote()` but when I look at that function, it's just a stub that says TODO.

I look at how the codebase authenticates with GitHub. It uses a GitHub App with a private key stored in an env var. That key is used to generate installation tokens. Good, that's already set up.

Now I trace through what should happen: (1) bot detects approval comment, (2) extracts the suggested code, (3) reads the file from the repo, (4) applies the change, (5) creates a commit, (6) pushes to the branch.

Steps 1-4 are already done. But step 6 — the `pushToRemote()` function — is incomplete. I read it:

```typescript
async function pushToRemote(
  owner: string,
  repo: string,
  branch: string,
  commit: Commit
): Promise<void> {
  // TODO: implement after merge conflict strategy is decided
}
```

**The code I depend on was marked as done and merged, but it's not actually done. I need to know: what's the merge conflict strategy?** I search the codebase for merge conflict handling. Nothing. I check the PR comments for that merged task. It's archived; GitHub only shows the last 100 reviews per PR, and this is old.

I search for issues or discussions about merge strategy. I find one comment in Slack (which I don't have access to; I can only search Git). **There's a key decision being made outside the codebase, and I have no way to find it.**

I make a guess: I'll do a simple fetch + rebase + push. If there are conflicts, I'll abort and comment on the PR that there was a conflict. This is reasonable but might not match what was decided.

I implement it. Then I realize: **what if the user made changes to the same file while I was processing? The rebase will fail. Do I retry? Do I notify the user? Do I force-push?** Force-pushing to a PR branch could be dangerous. But without knowing the merge strategy decision, I'm guessing.

I add error handling that comments on the PR when a push fails. But **if this happens repeatedly, there's no persistent record of why the bot was failing.** It's just errors in logs. And when the bot times out waiting for my incomplete dependency, **I have no way to communicate backward to whoever wrote that TODO and say: this decision needs to be made.**

---

## Simulation #10: Learning Management System (Laravel/PHP) — Acceptance Criteria Are Ambiguous

**Project:** Learning management system (Laravel/PHP)
**Your assignment:** Implement multi-tenancy support for course data. Ensure that when a student from Company A logs in, they see only courses for Company A, not courses from Company B. Add tenant isolation at the query level.
**These tasks are already done:** Tenant middleware added to all routes, `tenant_id` column added to courses table, authentication system updated to extract tenant from JWT
**These tasks come after yours:** Tenant-scoped file storage, audit logging showing which tenant accessed what data
**Notes from previous work:** "Queries should be automatically filtered by tenant_id without requiring explicit WHERE clauses in every query method."

---

I start by reading the tenant middleware. It extracts tenant_id from the JWT and stores it in a request context. Good. I then look at the Course model to see if there's a query scope or trait already applied.

I find a `HasTenantId` trait that's being used. It has a `scopedToTenant()` method. But when I read the actual implementation, it's just a stub:

```php
public function scopedToTenant(Builder $query): Builder {
    // TODO: return $query->where('tenant_id', ...);
}
```

I need to know: should this automatically apply to all queries, or only when explicitly called? The note says "automatically filtered without requiring explicit WHERE clauses." That suggests a global scope. But a global scope could break certain queries.

I search the codebase for how this scope is applied. I see it's only called explicitly in some places: `Course::scopedToTenant()->get()`. In other places, there's just `Course::all()`. **Are those other places a bug? Or are they intentional (like admin queries that should see all courses)?**

I look at the acceptance criteria: "when a student from Company A logs in, they see only courses for Company A, not courses from Company B." This is clear. But it doesn't say whether admins or site operators should see all courses. It doesn't say whether reports should be cross-tenant or not. **The requirement is ambiguous about what "see" means in different contexts.**

I implement a global scope using Laravel's `GlobalScope` interface. Now every query to the Course model automatically filters by tenant_id. But then I realize: I just broke the admin panel. An admin logs in and wants to see all courses across all tenants for reporting. My global scope blocks them.

**I could add an exception to the scope. But that requires knowing: when should the scope be bypassed?** I add it to `Course::withoutGlobalScope()`. But now there's a pattern: any query that needs to ignore tenant filtering has to explicitly call `withoutGlobalScope()`. **That's the opposite of the requirement: "without requiring explicit [clauses] in every query method."**

I decide to implement it as a global scope anyway and add a comment that explains when to use `withoutGlobalScope()`. But **if someone adds a new admin query and forgets to add `withoutGlobalScope()`, that's a silent data leak: the admin won't see the error, they'll just see no results and think the data doesn't exist.** And there's no way to audit whether my assumption about what "see" means was correct until someone runs the feature.

I finish the implementation and hope the next task (audit logging) will catch any issues.

---

## Summary

These simulations illustrate why Claude Code's tools are insufficient for complex projects:

1. **#6:** Missing credentials/secrets block execution but can't be tracked persistently
2. **#7:** Contradictory requirements with no way to ask which is correct
3. **#8:** Architectural decisions made without shared context get duplicated or diverge
4. **#9:** Incomplete dependencies merged; no way to find the decision that blocks progress
5. **#10:** Ambiguous acceptance criteria create silent correctness violations

All five involve the core pattern: **the developer can write code, but cannot reliably communicate about that code, query context outside the codebase, or track what information is blocking forward progress.** Without these capabilities, even thorough code review and testing can miss systemic issues.

Ralph's role is to provide persistent, queryable context *inside* the project, generated from deterministic rules, so Claude Code can work without these communication gaps.
