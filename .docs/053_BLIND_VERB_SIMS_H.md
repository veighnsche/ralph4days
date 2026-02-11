# Blind Verb Simulations H — Friction Analysis (Simulations #16-20)

These simulations explore moments where a developer hits friction with only basic tools: read files, write files, search code, run commands. No Claude integration, no MCP servers, no persistent notes layer. The friction points are marked in **bold**.

---

## Simulation #16: Podcast Hosting Platform — Webhook System Implementation

**Project:** Podcast Hosting Platform (Django/Python)
**Your assignment:** Implement the webhook delivery system that pushes episode publish events to subscriber applications. The system must support retry logic, signature verification, and subscriber endpoint testing.
**These tasks are already done:** Episode model refactor (consolidates publishing logic), Subscriber management endpoints (CRUD for external apps)
**These tasks come after yours:** Webhook event filtering and delivery metrics dashboard, Subscriber webhook logs and replay UI
**Notes from previous work:** The authentication model was changed from API keys to OAuth tokens last week. Some endpoints still reference the old system. The webhook spec doc exists in `/docs/webhook_spec.md`.

---

You pull up the project and search for existing webhook references. You find three different implementations: one in `/webhooks/delivery.py` that seems ancient (2019), another half-baked in `core/signals.py` that's hooked to Django signals, and a third in the tasks module that only handles internal events. **I need to know which one is actually being used and which are dead code, but searching for function names just gives me import statements — I don't know if they're actually called.**

You read the webhook_spec.md in docs. It talks about signature verification using HMAC-SHA256 and mentions "the auth layer" without specifying whether that's API keys or OAuth tokens. You check the subscriber model and it only has `webhook_url` and `secret_key` fields — nothing about OAuth. **The spec contradicts the actual auth migration. I have no way to know if this doc is stale or if the migration is incomplete. I need to ask the previous dev but I can't leave a note that persists.**

You grep for "OAuth" in the codebase and find it's only in the user authentication endpoints, not subscriber handling. So subscribers still use secret keys. You start designing the webhook implementation: queue-based delivery with retries, signature headers, test endpoints. You write a partial implementation in a new `webhooks/unified.py` file.

Then you realize: the subscriber secret keys are being stored in plaintext in the database. The old webhook spec says signatures should use these secrets, but **if I'm signing with plaintext secrets, either the spec is wrong about how it should work, or the secret storage is a security hole. Both interpretations require fixing something fundamental, and I have no way to know which previous decision I should trust.** You search for any comments about why secrets aren't hashed. Nothing. You find a GitHub issue reference in the schema migration: `# TODO: hash secrets (see issue #247)`. You can't access GitHub from here.

You implement the webhook system with hashed secret comparison during verification, deciding that security is the safer bet. But **you're making an architectural choice based on a TODO comment, and if you're wrong, the entire delivery system breaks. The next dev will either have to debug why production webhooks fail, or refactor the whole thing again.**

By the time you're done, you've written retry logic, signature verification, and test endpoints. But you have no certainty that this is compatible with how subscribers will actually authenticate.

---

## Simulation #17: Inventory Management — File Upload Processing

**Project:** Inventory Management (React Native/Expo)
**Your assignment:** Build the CSV bulk import feature that parses inventory item files and upserts them into the database. Support multi-file uploads, rollback on partial failure, and progress reporting.
**These tasks are already done:** Database schema for inventory items (SKU, quantity, location, metadata), S3 upload endpoint with presigned URLs
**These tasks come after yours:** Conflict resolution UI (manual review for duplicates), Import history and audit trail
**Notes from previous work:** The upload endpoint was built by the contractor last month. It stores files with UUID names in the S3 bucket. Inventory items have a compound unique key (warehouse_id, sku).

---

You start by understanding the data flow. Read the inventory model schema, which defines a compound primary key on (warehouse_id, sku). Good. Read the S3 upload endpoint code. It returns a URL and a file_id UUID. You search for where files are actually parsed in the codebase. Nothing. This is new work.

You create a parser module that reads CSV, validates each row, and batches database operations. You add a `verifyCSVStructure` function to check required headers. You test it locally with sample data. You start building the upsert logic, checking for existing items by (warehouse_id, sku) and either updating quantities or inserting new records.

Then you realize: **the S3 endpoint stores files with only a UUID name. I have no way to know what the original filename was, or what warehouse the upload was for. The file itself just sits in S3 with a random name, and I have to infer context from somewhere else.** You search the endpoint code for metadata. It doesn't store any. You check if there's a request parameter that specifies warehouse_id. There isn't — the contractor just built a dumb file uploader.

You look at the task description again. It says "bulk import feature" but doesn't specify: Do I associate the file with a warehouse before parsing? After? Is the warehouse_id embedded in the CSV header or selected by the user in the UI? **The task description doesn't match the existing infrastructure. Either the S3 endpoint was built wrong, or the task description is incomplete. And I have no way to communicate back to whoever designed the endpoint — I just have to guess and hope the next person agrees with my guess.**

You make a decision: the warehouse_id will be passed as a parameter to the import function. You update your parser to accept it. But now you're designing an interface that might not match what the UI will pass. You add a comment in the code: `// TODO: verify warehouse_id comes from UI context, not hardcoded`. **But the next dev won't see that comment until they hit a runtime error.**

You finish the implementation, but you've spent 2 hours worrying about a contract that doesn't exist between the upload endpoint and the import processor. **If I were on the same team, I'd just ask. But I'm here with only code and terminal access, so I'm reverse-engineering assumptions and leaving comments that might never be read.**

---

## Simulation #18: Internal HR Portal — Role-Based Permissions Overhaul

**Project:** Internal HR Portal (Next.js/TypeScript)
**Your assignment:** Refactor the permission system from a simple boolean-based model (is_admin, is_manager) to a comprehensive role-permission matrix. Define core roles (Admin, Manager, HR, Employee) and implement granular permission checks across the application.
**These tasks are already done:** Database migration for roles table and permission tables, API endpoint updates to check role instead of booleans
**These tasks come after yours:** Permission management UI (create custom roles), Audit logging for permission-based actions
**Notes from previous work:** The API is now returning role_id instead of boolean flags. The previous permission checks are still scattered across the codebase as `if (user.is_admin)` statements.

---

You map out the work: find all permission checks in the codebase, replace them with role-based lookups, define what each role can do, make sure the enforcement is consistent. You search for `is_admin` and `is_manager` across the frontend and backend. Find 47 places. Some are in components, some in API handlers, some in utility functions.

You start refactoring. Create a `permissions.ts` module that maps roles to capabilities. Define role constants. Build a `hasPermission(user, action)` function. Replace the scattered checks one by one. You're halfway through when you notice something: in the employee directory feature, there's a permission check that says `if (user.is_manager || user.is_admin)` can edit employee records. But there's also an HR role that should probably have this permission. **The task says "implement granular permission checks", but it doesn't define what the matrix actually is. I have to invent the permission rules myself.**

You search for documentation about what each role should be able to do. There's a JIRA ticket referenced in the migration commit: `SPEC-042: Implement new permission model`. You can't access JIRA from here. You search the codebase for any comments about the permission design. Find a single TODO: `// TODO: define permission matrix in spec doc`. That's not helpful.

**I'm supposed to implement a permission system, but the specification for what permissions should exist doesn't exist. I can either guess based on existing boolean checks (which would just lock in the old design), or I can invent a new permission model from scratch. Both feel wrong.** You go with a hybrid approach: create a comprehensive permission list (create, read, update, delete per resource, plus special actions like approve_leave), assign them to roles conservatively, and add comments saying which assumptions you made.

Hours later, you finish refactoring. You've defined 5 roles, 23 permission types, and updated 47 permission checks. But **you're not confident that your permission matrix matches what the business actually wanted. The task after yours is "Permission management UI", which means someone will eventually try to create custom roles and break your hardcoded assumptions.**

You push the code. It compiles, the old boolean checks are gone, and role-based checks are in place. But you leave a comment in the code: `// Assumed hierarchy: Admin > HR > Manager > Employee. Verify with product before finalizing custom role UI.` **That comment will be buried in the code and the next dev probably won't see it until they're confused about why the custom role feature doesn't work as they expected.**

---

## Simulation #19: E-commerce Marketplace — Payment Processing Race Condition

**Project:** E-commerce Marketplace (Rails/Ruby)
**Your assignment:** Implement atomic payment processing that ensures orders transition to "paid" state only after the payment provider confirms the transaction. Handle timeout scenarios and webhook confirmation.
**These tasks are already done:** Order creation endpoint (creates pending orders), Integration with Stripe API for charge creation
**These tasks come after yours:** Refund workflow, Webhook handler for asynchronous payment confirmations
**Notes from previous work:** The Stripe integration uses the test API key. Someone wrote a comment: "Be careful with race conditions on payment confirmation — see #189." Issue #189 is closed but you can't view it.

---

You start by reading the order model and payment processing flow. The order has a `status` field: pending, paid, failed, refunded. You find the Stripe integration code that creates a charge and returns a charge_id. It looks straightforward: call Stripe, get back a charge object with status "succeeded", update the order.

You design the atomic payment flow: wrap the charge creation and order status update in a database transaction. Seems simple. You write the code:

```ruby
def process_payment(order)
  charge = Stripe::Charge.create(amount: order.total, ...)
  order.update(status: 'paid', stripe_id: charge.id)
end
```

Then you think: what if the Stripe call succeeds but the database update fails? The charge went through but the order record still says pending. A customer sees "payment failed" on their screen but their card was charged. That's bad.

You wrap it in a transaction. But then: **what if the database transaction succeeds, but the response never reaches the client? The client thinks the payment failed and tries again. Now the charge is duplicated. Stripe has idempotency keys for this, but do I use them? And if the customer refreshes the page before receiving the response, they might initiate a second payment — then the transaction will fail because the order already moved to paid state. Except the second charge might still go through at Stripe, and now you have a charge with no matching order.**

You search the codebase for how they handle this. Find a webhook handler stub that doesn't exist yet (it's marked as TODO). Find a `stripe_id` column on the order model. Realize that the task list says "Webhook handler for asynchronous payment confirmations" is AFTER yours. **So the current approach is synchronous payment confirmation, which is inherently racy. The webhook handler will fix it later. But my task is to implement it atomically without the webhook.**

You read the closed issue reference: "Be careful with race conditions on payment confirmation — see #189". You can't view #189. **I'm being warned about a specific problem that I can't look up, so I don't know if it's a known architectural issue, a previous bug, or something else.**

You implement with idempotency keys, wrap the transaction carefully, and add a flag to track "payment pending" separately from "payment confirmed". You add comprehensive error logging so if duplicate charges happen, at least you'll see them. You leave a comment: `// Atomic with idempotency, but truly atomic confirmation requires webhook (see task: Webhook handler for async payment confirmations)`. **That comment is a crutch. The next dev will read it and either understand the limitation or ignore it.**

By the time you're done, you've implemented payment processing that's safer than the naive approach but still vulnerable to the client-side timing issue. The webhook handler will fix it, but in the meantime, there's a race condition in production. You can't prevent it — your tools don't let you add a persistent note to the webhook handler task explaining the dependency.

---

## Simulation #20: Smart Home Automation — Configuration Schema Drift

**Project:** Smart Home Automation (Kotlin/Spring)
**Your assignment:** Build the device configuration loader that reads JSON device definitions from a config directory, validates them against a schema, and registers devices in the Spring context at runtime.
**These tasks are already done:** Device model classes and repositories, Config file format specification (config/devices.json schema)
**These tasks come after yours:** Dynamic device UI generation, Configuration hot-reload without restart
**Notes from previous work:** There are example config files in `config/examples/` but they look old. The device model was refactored three months ago.

---

You start by reading the device model classes. Get the structure: id, name, type, properties, capabilities. Then read the spec document for config files. It defines a JSON schema with required fields: id, name, type, properties, capabilities. Looks straightforward.

You look at the example configs in `config/examples/`. You see:
- `thermostat.json`: has id, name, type=thermostat, properties with min/max temperature, capabilities=[heating, cooling]
- `light_bulb.json`: has id, name, type=light, properties with brightness range, but NO capabilities array

You check the device model. The `capabilities` field is required. You check the schema spec. It says capabilities is required. **But the example light_bulb.json is missing it. Either the spec is wrong, the example is stale, or the model changed and someone didn't update the examples.**

You search the git history (you can do that with terminal). Find the device model refactor commit from three months ago:
```
commit abc123: "refactor: Device model restructure"
- capabilities is now a required field
- properties is now a nested object
- removed deprecated attributes field
```

Okay, so the model is current. You check when the examples were last updated:
```
commit xyz789: "docs: add example device configs"
Date: 6 months ago
```

So the examples are stale by 3 months. **The specification is correct, but the examples violate it. Should I assume the examples are aspirational and no one's actually using them? Or should I code defensively to allow missing capabilities?** You decide to be strict: validate against the spec, reject malformed devices.

You write the validator. Load each JSON file, check it against the schema, throw an error if it doesn't match. You register valid devices in Spring as beans. Deploy it to test. It rejects the light_bulb.json example because capabilities is missing.

Then you realize: **someone added that example file to the repo. If my validator rejects it, either I'm wrong, or the example is wrong. But I can't know which without context I don't have.** You search for usages of that light_bulb config. Can't find any tests, no references, no comments. It's orphaned.

You make a decision: update the examples to match the spec. You add a capabilities array to light_bulb.json with [brightness, on/off]. You assume that's what was intended. **But what if there's a version of that config file in a customer's deployment, and I just broke their system by adding a required field? Or what if the light bulb type actually doesn't support capabilities and the model is wrong?**

You leave a comment in the code: `// Device configs are validated against schema. Examples updated to match current model (3 month drift detected — see device model refactor commit abc123).` **The next person will see that comment and either trust my decision or think I was being paranoid.**

You finish the loader, but you've spent an hour untangling a specification/example/model mismatch that required context-dependent decisions. **If I could leave structured notes linking the model version, the spec version, and the decision I made, the next dev could quickly understand the reasoning. Instead I'm putting free-form comments in code, hoping they'll be found and read at the right moment.**

---

## Summary

These five simulations highlight friction points that occur in the absence of persistent context-sharing mechanisms:

1. **Dead code vs. active code ambiguity** (Podcast) — Finding what's actually used vs. deprecated
2. **Missing interface contracts** (Inventory) — Assumptions about data flow between subsystems
3. **Unspecified requirements** (HR Portal) — Designing a system when the specification doesn't exist
4. **Known unsolved problems** (E-commerce) — Warnings about issues you can't research
5. **Specification/implementation drift** (Smart Home) — Examples, models, and specs out of sync

In each case, the developer makes reasonable decisions but **lacks a way to leave structured information for the next person** beyond comments buried in code. A persistent note system (like Ralph's learnings layer) would let developers record:
- What the current state actually is (not what it should be)
- Why they chose one interpretation over another
- What decisions are temporary until the next task/ticket
- What dependencies exist between tasks

Without that layer, each subsequent dev re-discovers the same ambiguities and makes decisions in a vacuum.
