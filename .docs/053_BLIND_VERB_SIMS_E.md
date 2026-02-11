# 053_BLIND_VERB_SIMS_E.md

Five blind verb simulations testing Claude Code's ability to navigate ambiguous tasks, missing context, and hidden contradictions on unfamiliar codebases.

---

## Simulation #1

You are Claude Code. You have been given a coding task on a project you haven't seen before. You have standard tools: read files, write files, search code, run terminal commands. That's it. No other tools.

Here is your task briefing:

**Project:** Social media scheduler (Python/FastAPI)
**Your assignment:** Add multi-tenancy isolation to the database layer. Ensure all queries automatically filter by current tenant without requiring explicit WHERE clauses in every handler.
**These tasks are already done:** "Set up Alembic migrations" (completed 3 weeks ago), "Extract tenant ID from JWT token" (completed 5 days ago)
**These tasks come after yours:** "Audit all API routes for tenant leakage" (pending), "Write tenant isolation integration tests" (pending)
**Notes from previous work:** "Tenant ID extraction working but we haven't actually verified it's being passed anywhere. The models still look like they're querying across all tenants."

Now simulate yourself executing this task. Write a stream-of-consciousness narrative (300-500 words) of what you do, what you find, and what happens. Include the complication.

### Simulation

I start by reading the models file to understand the current database structure. I find a `User` model, `Post` model, `Schedule` model — standard SQLAlchemy stuff. No tenant column anywhere. That's odd. The task says "add multi-tenancy isolation" but I need to know: are we adding the column fresh, or just adding the filtering logic?

Let me check the migrations folder. I see `001_init_schema.sql`, `002_add_auth.sql`, `003_add_schedules.sql`. Nothing about tenants. **I need to know whether the previous task ("Set up Alembic migrations") already added a tenant column that I just haven't seen yet, or if that task was about something else entirely — but I have no way to ask anyone or check commit messages in context.** The learnings say they "extracted tenant ID from JWT" but I can't verify what that code actually does without searching the whole codebase.

Let me grep for "tenant" in the codebase. I find it in auth handlers: `tenant_id = extract_tenant_from_token(request)`. But then I search the models — no `tenant_id` field. Search the main routes — some functions call `get_posts()` with no tenant parameter. Others call it with one. This is a mess.

I start reading one route handler carefully. It calls `db.query(Post).all()`. All posts. Not filtered. But the handler receives the JWT token. So either:

1. The tenant extraction is done but the filtering isn't wired up yet (my job)
2. Someone forgot to pass the tenant ID down
3. The tenant column doesn't exist in the schema yet

**I want to leave a note somewhere saying "I found the tenant extraction code in auth.py but it's not being used anywhere yet — I need to know if I should add the column to models or just the filtering logic."** But I have no persistent state to write that observation. If I search the code and lose the context, I'll have to re-search it all over again.

I decide to check the database schema directly: I open the SQLite file and inspect it. No tenant column anywhere. So I need to:

1. Add the column to models
2. Create a migration
3. Add an auto-filter somewhere

But here's the complication: When I read the "Extract tenant ID from JWT token" task notes, I realize the code they wrote stores `tenant_id` in `request.state`. But then I search the models and they don't have a relationship to tenants — there's no `Tenant` model at all. The task assumes we're doing column-level filtering on an existing model, but maybe we need a parent `Tenant` table first. **I can't know if the previous task left this half-done intentionally or if it's actually broken, and I have no way to follow the chain of what they were thinking.**

I start writing a migration to add `tenant_id` to the `Post` table, then realize I should probably add it to `User`, `Schedule`, and maybe others. **How do I know which tables need it without a design doc or architecture summary?** I'm guessing at the scope.

---

## Simulation #2

You are Claude Code. You have been given a coding task on a project you haven't seen before. You have standard tools: read files, write files, search code, run terminal commands. That's it. No other tools.

Here is your task briefing:

**Project:** Personal finance tracker (SvelteKit/TypeScript)
**Your assignment:** Implement offline-sync for transactions. When the user is offline, transactions should queue locally, then sync when connection is restored.
**These tasks are already done:** "Add service worker skeleton" (completed 10 days ago), "Create local IndexedDB schema" (completed 1 week ago)
**These tasks come after yours:** "Add conflict resolution for duplicate transactions" (pending), "Write offline sync E2E tests" (pending)
**Notes from previous work:** "Service worker is registered but not actually intercepting network requests. IndexedDB is set up but we're not using it yet."

Now simulate yourself executing this task. Write a stream-of-consciousness narrative (300-500 words) of what you do, what you find, and what happens. Include the complication.

### Simulation

I start by looking at the service worker file. It's mostly empty — just a registration stub. No fetch intercept, no sync logic. The IndexedDB schema file has collections defined: `transactions`, `users`, `accounts`. Good.

**I immediately realize I need to know: what's the contract between the frontend and the backend API?** If a transaction endpoint is `POST /api/transactions`, I need to know what it returns. But I don't have API docs. I can search for the endpoint definition, find a FastAPI route or Express handler, and read that. But what if the endpoint doesn't exist yet? What if it's defined in a separate service I can't see? I have no way to know the boundary.

Let me check the main transaction creation code. I find a form component that does `fetch('/api/transactions', { method: 'POST', body: JSON.stringify(tx) })`. Simple. Now I need to:

1. Intercept that fetch in the service worker
2. When offline, store to IndexedDB instead
3. When online, drain the queue and POST them

I start writing the service worker fetch handler. Straightforward stuff — check `navigator.onLine`, if true call fetch, if false store and return a fake 200. But immediately **I hit a wall: how do I know if a stored transaction in IndexedDB has actually been synced or not?** I need some kind of status flag. But did the "Create IndexedDB schema" task already add one? I check the schema file — no status field defined. So either:

1. I need to alter the schema
2. The previous task left it incomplete
3. I should be storing metadata separately

**I can't ask the previous person "did you finish the IndexedDB schema or not?" I just have to guess.** I decide to add a `synced: boolean` flag to the transaction records. But now I'm changing the schema they created. What if they have a reason for the structure they chose?

I start writing the sync logic: when `navigator.onLine === true`, wake the service worker, query all transactions where `synced === false`, POST them one by one, set `synced: true`. Standard stuff.

But here's the complication: **I read the list of pending tasks and see "Write offline sync E2E tests" is next.** If I make my indexedDB schema different from what the test task expects, I'm creating a mismatch. But I don't have access to their test assumptions — they haven't written them yet. I'm coding blind about what the contract should be.

I also realize I'm not sure what "offline sync" means here. Does it mean:

1. Store failed requests locally and retry when online?
2. Actively queue writes while offline and sync on reconnect?
3. Full bi-directional sync with conflict resolution?

The brief says "queue locally, then sync" which sounds like #2. But the pending task "Add conflict resolution" implies #3 might be coming. So maybe my implementation needs to be extensible for conflict resolution? Or maybe I should keep it simple and let the next task handle it? **I have no way to know the intended scope boundary because there's no architecture doc saying "offline-sync does X, conflict resolution does Y."**

I push forward, write the service worker fetch handler, write the sync trigger on `online` event, test locally. It works when I manually toggle `navigator.onLine`. I commit it. But I'm uneasy that the next person might disagree with my schema choice.

---

## Simulation #3

You are Claude Code. You have been given a coding task on a project you haven't seen before. You have standard tools: read files, write files, search code, run terminal commands. That's it. No other tools.

Here is your task briefing:

**Project:** CI/CD pipeline manager (Rust CLI)
**Your assignment:** Implement api-rate-limiting to prevent abuse. The service currently gets hammered by bad actors. Clients should be limited to 100 requests per minute per API key.
**These tasks are already done:** "Implement JWT/API key authentication" (completed 2 weeks ago), "Set up Redis cache layer" (completed 1 week ago)
**These tasks come after yours:** "Monitor rate-limit metrics in Prometheus" (pending), "Add client backoff / jitter guidance" (pending)
**Notes from previous work:** "Redis connection is working but we're only using it for session caching. No other keys in there yet. API keys are stored in the main database but not validated in the middleware — auth is only checked on specific routes."

Now simulate yourself executing this task. Write a stream-of-consciousness narrative (300-500 words) of what you do, what you find, and what happens. Include the complication.

### Simulation

I start by reading the middleware stack. I find `auth_middleware` that checks a Bearer token, but it's only applied to certain routes (`/api/v1/*`). There's no middleware applied globally. That means rate-limiting won't work unless I either:

1. Apply a global middleware for rate-limiting
2. Apply it only to `/api/v1/*`
3. Apply it per-route

The brief says "100 requests per minute per API key" — that implies I'm tracking by API key, not by IP. So I need to extract the API key from the request, check Redis, increment a counter, check if over limit, reject if needed.

Let me look at how API keys are currently used. The auth middleware extracts a Bearer token and validates it against the database. It then stores something in `request.extensions` or `request.state`. I grep for where that state is used downstream. I find:

```rust
let api_key = req.extensions().get::<String>();  // or something
```

But I'm not 100% sure of the exact API. **I need to understand the request lifecycle in this framework, and I can either read the whole auth middleware carefully or I can just start writing and see what breaks.** I'm tempted to just implement it and see what the compiler tells me, but then I realize I need to write tests, and I don't know the test setup.

I check the tests folder. I find integration tests that make HTTP requests to the service. They mock the database but do they mock Redis? I don't see any Redis mocking setup. **So either I need to add that to every test, or I need to inject Redis as a trait so I can mock it.** But refactoring the entire request handling to support dependency injection might be out of scope. This is frustrating — I don't know the architectural constraints.

I decide to just implement it: I'll add a rate-limit middleware that takes the API key from `request.extensions`, increments a Redis counter with key `rate_limit:{api_key}:{current_minute}`, and rejects if the count exceeds 100. Pretty standard.

But here's the complication: When I search the codebase for how Redis is currently used, I find that the session caching uses a different key naming scheme: `session:{user_id}` with a 1-hour TTL. If I use a similar pattern, I'm using `rate_limit:{api_key}:{minute}` which means I need to calculate the minute bucket. But `{minute}` is ambiguous — do I use Unix timestamp divided by 60? Or the minute field of the local time? Or UTC? **The previous task set up Redis but didn't document their conventions or assumptions, so I'm just guessing at a TTL/key strategy that matches whatever they decided.**

Also, the pending task "Monitor rate-limit metrics in Prometheus" suggests I might need to emit metrics. Should I do that now? The brief doesn't say so, but if I don't, I'm forcing the next person to add instrumentation after the fact. **I'm in the position of making a design choice without knowing the full context.**

I start implementing, and partway through I realize: **what if an API key doesn't have a rate limit? What if some clients should be unlimited (internal services) while others are limited?** Should I have a config table mapping API keys to rate limit tiers? Or just a blanket 100 RPM for everyone? The brief says "100 requests per minute per API key" but doesn't say "all API keys". Maybe I should make it configurable? But now I'm adding database queries to every request, and I'm not sure if that's acceptable from a performance perspective.

I just implement the blanket 100 RPM and move on. I can always make it configurable later.

---

## Simulation #4

You are Claude Code. You have been given a coding task on a project you haven't seen before. You have standard tools: read files, write files, search code, run terminal commands. That's it. No other tools.

Here is your task briefing:

**Project:** E-commerce marketplace (Rails/Ruby)
**Your assignment:** Add a webhook-system so third-party sellers can subscribe to order events (order.created, order.shipped, order.refunded, etc.) and receive real-time notifications via HTTP POST.
**These tasks are already done:** "Add seller account management" (completed 3 weeks ago), "Implement order status state machine" (completed 2 weeks ago)
**These tasks come after yours:** "Add webhook retry logic with exponential backoff" (pending), "Write webhook delivery audit logs" (pending)
**Notes from previous work:** "Order state machine is working great. Seller accounts are set up but I noticed the seller_id field on orders might not be universally populated yet — some old orders might be missing it."

Now simulate yourself executing this task. Write a stream-of-consciousness narrative (300-500 words) of what you do, what you find, and what happens. Include the complication.

### Simulation

I start by reading the Order model and the state machine implementation. The state machine has transitions like `pending -> shipped -> delivered` and also `pending -> refunded`. Good. Now I need to:

1. Create a Webhook model to store webhook subscriptions
2. Create a WebhookEvent model or table to track what was sent
3. Trigger webhook delivery when order state changes
4. Implement HTTP POST delivery with retry

I check the Order model. I see `state:string`, `seller_id:integer` (with belongs_to), various timestamps. The state machine uses something like Aasm or a custom transitions DSL. I find callbacks like `after_transition -> :notify_customer`. Perfect — I can add my own callback here.

But here's the complication: When I search for how `seller_id` is populated, I find that some orders don't have it. The notes mentioned "old orders might be missing it." I check the migrations. I see:

```ruby
create_table :orders do |t|
  # ... columns ...
end

# Later migration:
add_column :orders, :seller_id, :integer
```

So the column was added in a later migration, but old orders created before that migration still exist and are NULL. Now **I need to decide: does my webhook system require a seller_id, or should it support orders without one?** If a webhook fires for an order without a seller, should I:

1. Skip it (don't send webhook)
2. Use a default/system seller
3. Send it to all subscribed sellers

The task says "third-party sellers can subscribe to order events" — implying each seller gets their own events. But if an order has no seller_id, there's no one to notify. **I can't solve this without understanding the business logic, and I have no way to ask anyone.**

I decide to assume that my webhook system only applies to orders with a valid seller_id. If an order has no seller, no webhook fires. I'll add a guard clause in the state machine callback.

Now I design the Webhook model:

```ruby
class Webhook < ApplicationRecord
  belongs_to :seller

  EVENTS = ['order.created', 'order.shipped', 'order.refunded'].freeze

  validates :url, presence: true
  validates :event, inclusion: { in: EVENTS }
  validate :valid_url_format
end
```

Then I create a WebhookEvent or Job model to track deliveries (for the audit log task that comes after).

I hook into the Order state machine transitions. When an order transitions, I need to:

1. Identify the seller_id
2. Find all webhooks subscribed to that event by that seller
3. Queue jobs to deliver them

I use Rails' ActiveJob for this. I create a `DeliverWebhookJob` that makes an HTTP POST to the webhook URL with the order data.

But immediately **I hit a design question: what's the webhook payload format?** Should it be:

1. `{ event: 'order.created', data: { order: { id, status, ... } } }`
2. `{ order: { id, status, ... }, event: 'order.created' }`
3. Just the order JSON?

I have no API spec. **I want to put a comment in the code saying "TODO: define webhook payload format with stakeholders" but comments don't help me make progress right now.** I just pick option 1 because it feels standard (like Stripe webhooks).

I also realize the pending task is "Add webhook retry logic" — so should I implement retries now or leave them as TODO? The brief doesn't explicitly say don't do retries, but the next task is explicitly about retries. So probably I should just fire them once and let the next task add resilience. But that feels fragile. **I'm second-guessing the task scope because there's no clear boundary.**

I implement it without retries, add basic error handling (catch exceptions, log them), and move on. I test by creating a seller webhook, creating an order, and checking the HTTP requests. Works locally. I commit.

---

## Simulation #5

You are Claude Code. You have been given a coding task on a project you haven't seen before. You have standard tools: read files, write files, search code, run terminal commands. That's it. No other tools.

Here is your task briefing:

**Project:** Fleet management system (C#/.NET)
**Your assignment:** Implement an onboarding-wizard for new fleet managers. It should guide them through: company details, vehicle registration, driver info, insurance verification, and payment setup. Each step should be validatable and skippable.
**These tasks are already done:** "Build core fleet database schema" (completed 1 month ago), "Set up authentication and authorization" (completed 3 weeks ago)
**These tasks come after yours:** "Add document upload to onboarding (insurance, registration)" (pending), "Create onboarding progress tracking dashboard" (pending)
**Notes from previous work:** "The database schema has Company, Vehicle, Driver, and Payment tables. Authorization is role-based with 'admin' and 'manager' roles. Onboarding might need a new table to track progress though."

Now simulate yourself executing this task. Write a stream-of-consciousness narrative (300-500 words) of what you do, what you find, and what happens. Include the complication.

### Simulation

I start by reading the database schema and the authorization setup. I find:
- `Company` table with fields like `Name`, `Address`, `TaxId`
- `Vehicle` table with `Make`, `Model`, `VIN`, `LicensePlate`, `CompanyId`
- `Driver` table with `FirstName`, `LastName`, `LicenseNumber`, `CompanyId`
- `Payment` table with `CompanyId`, `PaymentMethod`, `Amount`, `Status`

I also check the auth roles: `admin` (global superuser), `manager` (company-scoped). The pending task "onboarding progress tracking" suggests I should probably create a table to track which step each manager is on. But the notes say "might need a new table" — so **should I create it or not?** The brief doesn't explicitly say, and the pending task suggests maybe they'll create it themselves.

I check if there's already an OnboardingProgress or similar table. There isn't. **I need to make a decision: do I create it now so my code can save progress, or do I assume progress is stored in some session state and I don't use the database?** If I don't create the table, the next task will have to add it, and then my code will need to be updated to use it. If I do create it, I might be overstepping into their work.

I decide to create it. I add an `OnboardingSession` table:

```csharp
public class OnboardingSession
{
    public int Id { get; set; }
    public int CompanyId { get; set; }
    public string CurrentStep { get; set; } // "company_details", "vehicles", etc.
    public Dictionary<string, object> CompletedSteps { get; set; } // JSON
    public DateTime CreatedAt { get; set; }
    public DateTime? CompletedAt { get; set; }
}
```

Now I need to implement the wizard. The brief says "each step should be validatable and skippable." I interpret this as:

1. Each step has validation rules
2. Users can skip to the next step (but maybe can't skip to the end?)
3. Users can go back and edit previous steps

I start building the wizard as a multi-step form. I use a controller with actions like `GET /onboarding/step/{stepName}` and `POST /onboarding/step/{stepName}`.

But here's the complication: **I read the pending task "Create onboarding progress tracking dashboard" and realize I don't know what this dashboard is supposed to show.** Does it show:
1. Admin view of all fleet managers in onboarding?
2. Personal view for each manager showing their progress?
3. Both?

If it's #1, then I might need to add audit fields to my OnboardingSession table (who completed it, when, etc.). If it's #2, I don't need anything special — I can just query OnboardingSession for the logged-in user. **I'm making database schema decisions without knowing the UI requirements of the next task.**

I also realize the task says "skippable" but doesn't define the business rules. Can you skip directly to payment setup without entering company details? That seems wrong. Do you skip optional steps? The brief mentions "company details, vehicle registration, driver info, insurance verification, and payment setup" — which ones are optional? **The acceptance criteria are ambiguous, and I have no way to ask for clarification.**

I implement it assuming:
- Company details: required
- Vehicle registration: required (at least one vehicle)
- Driver info: required (at least one driver)
- Insurance verification: required
- Payment setup: required
- All steps are sequential (you can't skip ahead, but you can skip filling in a step and come back to it)

But I'm not confident this is right.

I also notice the pending task mentions "Add document upload to onboarding" — so probably insurance verification needs to attach documents. Should I add a `DocumentUrl` field to my OnboardingSession table? Or maybe to a separate OnboardingDocuments table? **I don't want to create a structure that the next task will have to refactor, but I also don't want to leave it incomplete.** I decide to just store document URLs in the JSON, assuming the next task will normalize it into a proper table if needed.

I implement the wizard controller, add validation for each step, implement the progress saving, add an endpoint to check current progress. I test it manually, and it works. But I'm uneasy about the ambiguity — **I wish there was a way to leave a note for the next person saying "I assumed skippable means you can return to steps, not skip steps entirely — verify this assumption." But I can only commit code, not questions.**

---
