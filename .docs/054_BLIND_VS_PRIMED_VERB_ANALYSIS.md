# Blind vs Primed Verb Analysis

**Created:** 2026-02-11
**Source:** 20 blind simulations (053_E/F/G/H) compared against 30 primed simulations (051/051b/c/d, consolidated in 052)

## Methodology Difference

**Primed simulations (30):** Agents knew about Ralph, MCP servers, orchestration. They were told to "hallucinate MCP tool calls." This biased them toward naming tools and thinking in terms of a tool API.

**Blind simulations (20):** Agents were dropped into unfamiliar codebases with only read/write/search/terminal. No mention of Ralph, MCP, orchestrators, or tool APIs. Friction points emerged as **bold frustrations in natural language** — not named tools.

## Friction Point Extraction

Every bold passage from the 20 blind sims was extracted and categorized by what the agent actually wanted to DO, not what they called it.

### Category 1: "I need to ask someone a question" — 18/20 sims

The dominant friction. Agents constantly hit moments where they couldn't proceed without answers that weren't in the codebase.

| Sim | What they wanted to ask |
|-----|------------------------|
| #1 | "Did the previous task add the tenant column or not?" |
| #2 | "Did you finish the IndexedDB schema or not?" |
| #2 | "What's the intended scope boundary?" |
| #3 | "What are the architectural constraints for Redis mocking?" |
| #4 | "What's the business logic for orders without a seller?" |
| #5 | "Which onboarding steps are optional?" |
| #5 | "What does the progress tracking dashboard need to show?" |
| #7 | "Which does the project want — hardcoded or configurable thresholds?" |
| #7 | "Which tradeoff was chosen — latency or accuracy?" |
| #10 | "Does 'see' mean different things in different contexts?" |
| #11 | "What does the audit-log feature actually need?" |
| #12 | "Does the backend handle idempotency?" |
| #13 | "How stale is acceptable for late-arriving data?" |
| #14 | "What happens when an unauthorized user tries to act?" |
| #17 | "Does warehouse_id come from UI context or the CSV?" |
| #18 | "What permissions should each role have?" |
| #19 | "What's the race condition in issue #189?" |
| #20 | "Are the examples aspirational or is the spec wrong?" |

**Pattern:** These aren't requests for HELP — they're requests for DECISIONS. The agent has identified a fork in the road and can't pick a path without information that doesn't exist in the codebase.

### Category 2: "Something is wrong/contradictory/broken" — 16/20 sims

Agents found problems: stale docs, incomplete prior work, contradictory specs, dead code.

| Sim | What they found |
|-----|----------------|
| #1 | Previous task marked done but code is incomplete |
| #1 | No architecture doc explaining which tables need tenant_id |
| #3 | Redis conventions undocumented — key naming is guesswork |
| #4 | Orders with NULL seller_id — data integrity gap |
| #5 | Schema decisions will conflict with downstream task |
| #7 | Task says "configurable" but code is hardcoded — contradiction |
| #8 | Abstraction designed for UI, not batch operations |
| #9 | Dependency task merged with TODO stub — not actually done |
| #12 | Sync system will corrupt data on multi-device without conflict resolution |
| #13 | Hourly aggregation can't support sub-hour queries |
| #14 | Old permissions table lingers — dead code or not? |
| #14 | Inconsistent authorization patterns across codebase |
| #15 | Testing around the problem rather than testing the actual problem |
| #16 | Webhook spec contradicts auth migration |
| #16 | Three separate webhook implementations — which is active? |
| #20 | Examples violate the spec they're supposed to demonstrate |

**Pattern:** These map directly to `flag` from the primed set. The agent found a problem and wanted someone to know about it.

### Category 3: "I made a decision and want to record WHY" — 14/20 sims

Agents made judgment calls and desperately wanted to persist the reasoning.

| Sim | Decision they made |
|-----|-------------------|
| #1 | "Tenant extraction code exists but isn't wired up — I'll add the column" |
| #2 | "Added synced:boolean to IndexedDB — might conflict with previous schema" |
| #3 | "Blanket 100 RPM for all keys — could be wrong for internal services" |
| #4 | "Only fire webhooks for orders with seller_id — business logic guess" |
| #5 | "Skippable means return-to-step, not skip-ahead — assumption" |
| #8 | "Deviated from per-row category lookup to batch lookup — better but non-standard" |
| #10 | "Global scope for tenant filtering — might break admin panel" |
| #12 | "Built sync assuming backend handles idempotency — no guarantee" |
| #14 | "Standardized on AuthorizationException pattern — other pattern exists too" |
| #16 | "Hashed secrets for webhook verification — based on a TODO comment" |
| #17 | "warehouse_id passed as parameter — might not match UI" |
| #18 | "Invented permission matrix from scratch — no spec exists" |
| #19 | "Used Stripe idempotency keys — merge conflict strategy unknown" |
| #20 | "Updated stale examples to match spec — might break customer deployments" |

**Pattern:** This is `learned` + `decided` from the primed set. But blind agents expressed it differently: they didn't want to "call a tool" — they wanted to **leave a note that the next person would find at the right moment**. The frustration was always "a comment in code will be buried."

### Category 4: "I'm blocked on something external" — 8/20 sims

Something outside the codebase prevents progress.

| Sim | What blocked them |
|-----|------------------|
| #6 | Firebase secrets in GitHub Actions — can't access locally |
| #6 | APNs certificate pending IT approval |
| #6 | Can't track that this task unblocks only when certificate arrives |
| #9 | Key decision made in Slack — can't access from code |
| #11 | No API key/endpoint to test webhooks locally |
| #13 | Can't test late-arriving data without time travel |
| #15 | Timer-based retry untestable in synchronous tests |
| #19 | Issue #189 is closed and can't be viewed |

**Pattern:** Maps to `blocked` from the primed set. External dependencies, missing credentials, inaccessible context.

### Category 5: "The next person should do X" — 6/20 sims

Agent wanted to leave instructions for downstream tasks.

| Sim | What they suggested |
|-----|-------------------|
| #4 | "Define webhook payload format with stakeholders" |
| #12 | "Please implement idempotency keys on the backend" |
| #14 | "Drop the old permissions table — it's dead code" |
| #15 | "Refactor retry mechanism to use job queue for testability" |
| #19 | "Atomic confirmation requires webhook handler — dependency" |
| #20 | "Link model version, spec version, and this decision" |

**Pattern:** Maps to `suggest` from the primed set. Agent recommends an action they can't take themselves.

### Category 6: "I can't continue" — 4/20 sims

Agent hit a wall and couldn't produce meaningful output.

| Sim | Why they were stuck |
|-----|-------------------|
| #6 | Can't test end-to-end without certificate |
| #11 | Can't verify retry logic without understanding HTTP client abstraction |
| #13 | Can't test edge case without infrastructure that doesn't exist |
| #15 | Can't test timers without making tests worthless or slow |

**Pattern:** Maps to `stuck` from the primed set. But note: in blind sims, agents almost NEVER fully stopped. They pushed through with assumptions. `stuck` was rare because agents are trained to keep going.

## The Primed 9 Verbs — Do They Hold Up?

| Primed Verb | Blind Frequency | Verdict |
|-------------|----------------|---------|
| `done` | 20/20 (implicit — every sim ends with a commit) | **CONFIRMED** |
| `partial` | ~16/20 (most sims end with "works but incomplete") | **CONFIRMED** — and more common than expected |
| `stuck` | 4/20 | **CONFIRMED** — but rare; agents push through |
| `learned`/`decided` | 14/20 | **CONFIRMED** — the #2 most important verb |
| `flag` | 16/20 | **CONFIRMED** — the #3 most important verb |
| `ask` | 18/20 | **CONFIRMED AND PROMOTED** — #1 friction point |
| `suggest` | 6/20 | **CONFIRMED** — less frequent than primed sims suggested |
| `blocked` | 8/20 | **CONFIRMED** |
| `register` | ~1/20 | **DEMOTED** — primed artifact |

### Key Findings

**1. `ask` is the #1 verb, not a minor one.**

In primed sims, `ask` appeared in 16/30 (53%). In blind sims, it's 18/20 (90%). When agents don't know about an orchestrator, asking questions is their DOMINANT unmet need. Primed agents downplayed `ask` because they had other tools to compensate.

**2. `register` is a primed artifact.**

In primed sims, `register` (register a file as relevant) appeared in 20/30 (67%). In blind sims, it's basically absent. Why? Because blind agents just... edited the files. They didn't need to tell an orchestrator about them. `register` only makes sense when agents know there's a system tracking context.

**3. `learned`/`decided` is about PERSISTENCE, not signaling.**

Primed agents treated `learned`/`decided` as "tell the orchestrator what I found." Blind agents expressed it as "I wish I could leave a note the next person would find at the right moment." The need isn't signaling — it's **durable, discoverable knowledge**. A comment in code is insufficient because it's buried. The agent wants something like a sticky note attached to the task or feature.

**4. `partial` is more common than `stuck`.**

Blind agents almost never fully stop. They push through with assumptions and caveats. The natural ending state is "I did what I could, here's what remains" — that's `partial`, not `done` or `stuck`.

**5. `flag` works exactly as designed.**

The primed consolidation nailed it: one verb, structured params (severity, category). Blind agents found the same types of problems: stale specs, broken dependencies, contradictions, dead code, design gaps.

## Revised Verb Set: The Final 8

Based on blind evidence, dropping `register` and keeping `ask` prominent:

| # | Verb | Blind Freq | Purpose |
|---|------|-----------|---------|
| 1 | `done` | 20/20 | Task complete |
| 2 | `partial` | 16/20 | Did what I could, here's what remains |
| 3 | `stuck` | 4/20 | Can't continue at all |
| 4 | `ask` | 18/20 | Need a decision or clarification to proceed |
| 5 | `flag` | 16/20 | Found a problem (stale/broken/contradictory/ambiguous) |
| 6 | `learned` | 14/20 | Reusable knowledge or decision with rationale |
| 7 | `suggest` | 6/20 | Recommend action for a future task |
| 8 | `blocked` | 8/20 | External dependency prevents progress |

**83 raw verbs (primed) → 9 verbs (primed consolidation) → 8 verbs (blind-validated)**

The only change: `register` dropped. Everything else survived independent validation.

## Implications for MCP Server Design

1. **`ask` needs a response mechanism.** Unlike `flag` (fire-and-forget), `ask` expects an answer. The MCP server must either queue the question for human review or have a way to pause/resume the agent. This is the hardest verb to implement.

2. **`learned` needs discoverability.** Agents don't just want to record knowledge — they want the NEXT agent to find it at the right moment. This means learnings need to be indexed by feature/file/concept, not just stored chronologically.

3. **`partial` should be the default closing verb.** Most tasks end partially complete. Designing for `done` as the happy path and `partial` as an edge case gets it backwards. Design for `partial` as the default, with `done` as the exception.

4. **`blocked` and `stuck` are different.** `blocked` = external thing missing (credential, decision, upstream task). `stuck` = internal inability (can't figure it out, abstraction too complex, testing impossible). Different escalation paths.

5. **`suggest` is low-frequency but high-value.** Agents rarely suggest, but when they do, it's substantive: "split this task", "add idempotency keys", "drop dead code." These should be treated as first-class signals, not noise.
