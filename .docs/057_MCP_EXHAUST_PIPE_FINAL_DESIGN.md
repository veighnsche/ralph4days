# MCP Exhaust Pipe — Final Design

**Created:** 2026-02-11
**Source:** 12 research documents (051–056), 80+ simulations (30 primed, 20 blind, 30+ loop validation), sequential-thinking refinement
**Status:** Design complete. Ready for implementation.

---

## Architecture

Single `task_signals` table. Single reusable MCP server binary. All 8 verbs are `INSERT INTO task_signals`. Intelligence lives in Ralph's post-processing (Rust), not the MCP server.

The MCP server is parameterized via environment variables (`RALPH_TASK_ID`, `RALPH_SESSION_ID`, `RALPH_DB_PATH`). Every tool call is the same operation: parse params → bind verb-specific flat columns → INSERT. The MCP server is truly dumb — just an INSERT pipe.

### Execution Flow

1. Ralph builds a prompt (task description, context, learnings, prior work)
2. Ralph sets env vars and launches `claude --mcp-config ... --output-format stream-json`
3. Agent runs, writes code, calls MCP tools to signal back
4. Agent exits
5. Ralph reads all signals from `task_signals` for this session
6. Ralph's post-processing updates task status, stores learnings, routes flags
7. Ralph picks up the next task

---

## Database Schema

```sql
CREATE TABLE task_signals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    discipline_id INTEGER REFERENCES disciplines(id) ON DELETE SET NULL,
    session_id TEXT,
    verb TEXT NOT NULL CHECK(verb IN ('done','partial','stuck','ask','flag','learned','suggest','blocked')),

    -- Closing verbs
    summary TEXT,    -- done, partial
    remaining TEXT,  -- partial
    reason TEXT,     -- stuck

    -- ask
    question TEXT,
    options TEXT,    -- newline-separated string[] from tool input
    preferred TEXT,
    blocking INTEGER CHECK(blocking IN (0,1) OR blocking IS NULL),

    -- flag
    what TEXT,       -- flag, suggest
    severity TEXT CHECK(severity IN ('info','warning','blocking') OR severity IS NULL),
    category TEXT CHECK(category IN ('bug','stale','contradiction','ambiguity','overlap','performance','security','incomplete_prior') OR category IS NULL),

    -- learned
    kind TEXT,
    scope TEXT CHECK(scope IN ('project','feature','task') OR scope IS NULL),
    rationale TEXT,

    -- suggest
    why TEXT,
    feature_id INTEGER REFERENCES features(id) ON DELETE SET NULL,

    -- blocked
    "on" TEXT,
    detail TEXT,

    -- human answer to ask
    answer TEXT,

    created TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_task_signals_task ON task_signals(task_id);
CREATE INDEX idx_task_signals_discipline ON task_signals(discipline_id);
CREATE INDEX idx_task_signals_session ON task_signals(session_id);
CREATE INDEX idx_task_signals_verb ON task_signals(verb);
CREATE INDEX idx_task_signals_task_verb ON task_signals(task_id, verb);
CREATE INDEX idx_task_signals_feature ON task_signals(feature_id);
```

The `session_id` tracks which CLI session produced each signal — a task attempted multiple times (partial → re-queue) produces signals across multiple sessions.

---

## The 8 Verbs

### Closing Verbs (exactly ONE per session — last one wins)

#### 1. `done`

Signal that the task is fully complete and tested.

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `summary` | string | yes | What was accomplished. Include key decisions and outcomes. |

**Ralph action:** task.status → "completed", task.completed_at → now

**Payload example:**
```json
{
  "summary": "Implemented lobby WebSocket channel with join/leave/chat/game_starting broadcasts. Fixed Phoenix 1.7.18 API change. Tests passing."
}
```

#### 2. `partial`

Signal that progress was made but the task couldn't be fully completed. The remaining work will be picked up in a future session.

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `summary` | string | yes | What was accomplished so far. |
| `remaining` | string | yes | What still needs to be done and why you stopped. Be specific. |

**Ralph action:** task stays "pending" (re-queued). `remaining` is injected into the next session's prompt as continuation context.

**Payload example:**
```json
{
  "summary": "Implemented hash chaining on audit log writes with SHA-256 and SELECT FOR UPDATE serialization.",
  "remaining": "Verification endpoint not included — streaming through millions of rows needs its own task. Suggested as separate task via suggest()."
}
```

#### 3. `stuck`

Signal that you cannot make meaningful progress on this task.

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `reason` | string | yes | Why you're stuck. Be specific about what's blocking your ability to work. |

**Ralph action:** increment stagnation counter. If 3+ stuck sessions → task.status = "failed". Otherwise → re-queue with different approach hint or escalate to Opus review.

**Payload example:**
```json
{
  "reason": "Can't test GPU allocation without GPU host. Config is written but untested on actual hardware. Need GPU-enabled Docker host."
}
```

### Signal Verbs (zero or more per session)

#### 4. `ask`

Ask a question that you need answered to do this task well. If `blocking: true`, you should also call `partial` to pause the task until the question is answered.

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `question` | string | yes | The specific question you need answered. |
| `options` | string[] | no | If you've identified possible answers, list them. |
| `preferred` | string | no | If you have a recommendation, state it. |
| `blocking` | boolean | yes | True if you cannot proceed without the answer. False if nice-to-know but you can continue with your best judgment. |

**Ralph action:**
- `blocking: false` → stored, surfaced in UI for human review. Informational.
- `blocking: true` + closing verb is `partial`/`stuck` → task.status = "needs_input". Question queued for human. When answered, re-queue task with answer injected into prompt.
- `blocking: true` + closing verb is `done` → agent resolved it themselves. Question becomes informational.

**Payload example:**
```json
{
  "question": "Should retry logic be sync (fits current codebase) or async (fits task description)?",
  "options": ["keep-sync-add-simple-retry", "revert-to-async-queue", "rewrite-task-for-sync"],
  "preferred": "keep-sync-add-simple-retry",
  "blocking": true
}
```

#### 5. `flag`

Report a problem you discovered during this task.

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `what` | string | yes | Clear description of the problem. |
| `severity` | enum | yes | `info` (FYI, no action needed now), `warning` (should be addressed, doesn't block this task), `blocking` (this task can't be fully completed because of this) |
| `category` | enum | yes | `bug`, `stale`, `contradiction`, `ambiguity`, `overlap`, `performance`, `security`, `incomplete_prior` |

**Ralph action:** severity determines routing:
- `info` → stored, included in next session's prompt context
- `warning` → stored, surfaced in UI for human review
- `blocking` → stored, may affect task re-queuing strategy, escalated

Category is metadata for the human reviewer, not routing logic for Ralph.

**Payload example:**
```json
{
  "what": "Route-level middleware (task #2) and new Eloquent scopes can disagree — middleware might allow a request that returns empty due to scope, or scope might allow data the middleware should block.",
  "severity": "warning",
  "category": "bug"
}
```

#### 6. `learned`

Record knowledge that will be useful for future tasks on this project.

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `text` | string | yes | The knowledge to record. Should be specific enough that a different developer on a different task would benefit from it. |
| `kind` | enum | yes | `discovery` (factual finding about the codebase), `decision` (judgment call with rationale), `convention` (project pattern to follow) |
| `rationale` | string | no | For decisions: why you chose this approach and what alternatives you rejected. |
| `scope` | enum | no | `project` (always relevant), `feature` (relevant to same feature's tasks), `task` (only relevant if this task is re-attempted). Default: `feature`. |

**Ralph action:** stored in `task_signals`. Prompt-builder queries by feature + scope when building prompts for future tasks. `scope: "project"` learnings are included in ALL prompts. `scope: "feature"` learnings are included in prompts for tasks in the same feature.

**Payload example:**
```json
{
  "text": "Using SELECT FOR UPDATE to serialize hash chain writes.",
  "kind": "decision",
  "rationale": "Concurrent inserts could fork the chain. Serialization via row lock is acceptable because audit writes are not high-throughput.",
  "scope": "feature"
}
```

#### 7. `suggest`

Recommend an action that should be taken but is outside the scope of your current task.

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `what` | string | yes | What should be done. |
| `kind` | enum | yes | `new_task`, `split`, `refactor`, `alternative`, `deprecate` |
| `why` | string | yes | Why this is needed. |
| `feature` | string | no | Which feature this relates to (for `new_task` suggestions). |

**Ralph action:**
- `kind: "new_task"` → auto-create task with `origin: "agent"` marker. Surfaced in UI with robot icon. Human can approve, modify, or reject.
- All other kinds → stored for human review. Don't auto-create anything.

**Payload example:**
```json
{
  "what": "Add audit chain verification endpoint — stream through audit_logs, recompute hash chain, report first broken link",
  "kind": "new_task",
  "why": "Verification logic is complex enough to be its own task — streaming millions of rows, pagination, caching",
  "feature": "audit-log"
}
```

#### 8. `blocked`

Report that you cannot complete part of your task because something outside your control is missing or broken.

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `on` | string | yes | What is blocking you. |
| `kind` | enum | yes | `upstream_task` (dependency on another task that's incomplete) or `external` (credentials, services, human decisions, infrastructure) |
| `detail` | string | no | Additional context about the blocker. |

**Ralph action:**
- `kind: "upstream_task"` → find the referenced task, add dependency link. task.status = "blocked". Auto-resolves when the dependency completes.
- `kind: "external"` → task.status = "blocked". Surfaced in UI for human resolution.

Only actionable when closing verb is NOT `done`. If agent calls `blocked` + `done`, the blocker is treated as informational (agent worked around it).

**Payload example:**
```json
{
  "on": "Redis service and credentials missing from environment",
  "kind": "external",
  "detail": "No Redis instance found in .env.example, infrastructure code, or learnings from previous tasks."
}
```

---

## Post-Processing Rules

Ralph's loop_engine runs this pipeline after every CLI session:

1. **Read** all `task_signals` for this `session_id`, ordered by `created`
2. **Identify closing verb:** the last `done`/`partial`/`stuck` signal
3. **No closing verb?** Infer `stuck(reason: "session ended without closing signal")`
4. **Process closing verb:**
   - `done` → task.status = "completed", task.completed_at = now
   - `partial` → task stays "pending" (re-queued), `remaining` appended to session context for next attempt
   - `stuck` → increment stagnation counter. 3+ → task.status = "failed". Otherwise → re-queue (potentially with Opus review hint)
5. **Process signal verbs:**
   - `ask(blocking: true)` + closing is `partial`/`stuck` → task.status = "needs_input"
   - `ask(blocking: true)` + closing is `done` → informational only
   - `ask(blocking: false)` → informational only
   - `flag` → stored; severity determines UI prominence
   - `learned` → stored; prompt-builder queries at build time
   - `suggest(kind: "new_task")` → auto-create task with agent origin
   - `suggest(other kind)` → stored for human review
   - `blocked(kind: "upstream_task")` + closing is NOT `done` → add dependency, task.status = "blocked"
   - `blocked(kind: "external")` + closing is NOT `done` → task.status = "blocked"
   - `blocked` + closing is `done` → informational only
6. **Stagnation check:** hash database + progress.txt + learnings.txt, compare with pre-session hash
7. **Next task:** pick next pending task, or pause if all remaining tasks are blocked/needs_input

---

## Verb Combination Rules

- **Closing verbs are exclusive:** exactly one per session. Last one wins if multiple are called.
- **Signal verbs are additive:** multiple calls of the same verb are fine (e.g., 3 flags + 2 learnings).
- **`done` overrides all blockers:** if agent calls `blocked` then later calls `done`, the blocker is informational.
- **`partial`/`stuck` activate blockers:** `blocked` and `ask(blocking: true)` only change task status when the closing verb is `partial` or `stuck`.
- **No closing verb = stuck:** if the agent crashes or times out without calling a closing verb, Ralph infers stuck.

---

## What's NOT in the MCP Server

Things deliberately excluded based on simulation evidence:

- **No read tools.** All input comes from the prompt. Agents never needed `get_task`, `list_features`, etc. across 80+ sims.
- **No `register` verb.** File tracking is handled by Ralph via `git diff` between sessions. Dropped after blind validation showed 1/20 frequency (primed artifact).
- **No CRUD on other tasks.** Agent signals. Ralph acts. Agent never modifies tasks, features, or metadata directly.
- **No `cancel`/`skip` verb.** `stuck(reason: "task is obsolete")` covers this. Appeared 0 times in 50+ sims.
- **No `progress` verb.** Ralph monitors the CLI stream output for real-time visibility. Adding a progress verb would be noisy.
- **No deduplication logic.** MCP server is a dumb INSERT pipe. Deduplication of repeated learnings is Ralph's post-processing job.

---

## Prompt-Builder Responsibilities (Not MCP, But Critical)

The MCP server is output-only. The prompt-builder is input-only. They're complementary:

1. **Learning retrieval:** Query `task_signals WHERE verb = 'learned'` filtered by feature + scope. Merge with human-authored `features.learnings`. Include relevant learnings in prompt.
2. **Answer injection:** When human answers an `ask`, prepend to next session's prompt: `"ANSWER to your question '{question}': {answer}"`
3. **Continuation context:** When re-queuing a `partial` task, include the `remaining` text and prior session's `summary` in the prompt.
4. **Flag context:** Include prior `flag` signals (especially `blocking` severity) so the agent is aware of known problems.
5. **File context:** Use `git diff` from prior sessions to populate `context_files` for subsequent tasks.

---

## Future Work

1. **Learning deduplication** — If 3 sessions all learn the same thing, the prompt grows. Post-processing should deduplicate.
2. **Learning relevance ranking** — Not all learnings for a feature are relevant to every task in that feature. May need keyword/concept matching.
3. **`ask` answer UI** — Human needs an interface to see blocked questions and provide answers.
4. **`suggest(kind: "new_task")` approval flow** — Auto-created tasks need review before entering the main queue.
5. **Stagnation interaction with signals** — If a session produces `partial` with meaningful learnings, should that reset the stagnation counter? Currently stagnation is purely hash-based.

---

## Validation

**83 raw verbs** (30 primed simulations across 4 independent agents) → **9 consolidated verbs** (category analysis) → **8 blind-validated verbs** (20 blind simulations, `register` dropped) → **8 stress-tested verbs** (sequential-thinking analysis against hardest sims).

Stress-tested against:
- Incomplete permission migration with two coexisting models (sim #19) ✓
- WebSocket performance trap requiring architectural decision (sim #28) ✓
- Code review bot with stub dependency and inaccessible Slack context (sim #9) ✓
- Weather station batch race condition with data duplication (sim #24) ✓

Every friction point from every simulation maps cleanly to one or more of the 8 verbs.

---

## Research Trail

| Doc | Content | Sims |
|-----|---------|------|
| 051 | Primed sims batch A | #1–15 |
| 051b | Primed sims batch B | #16–20 |
| 051c | Primed sims batch C | #21–25 |
| 051d | Primed sims batch D | #26–30 |
| 052 | Consolidation: 83 → 9 verbs | — |
| 053_E | Blind sims batch E | #1–5 |
| 053_F | Blind sims batch F | #6–10 |
| 053_G | Blind sims batch G | #11–15 |
| 053_H | Blind sims batch H | #16–20 |
| 054 | Blind vs primed analysis: 9 → 8 verbs | — |
| 055 | Ralph loop validation | #21–23 |
| 056 | Iteration 1 (30 more Haiku sims) | 30 sims |
| **057** | **This document: final design** | — |
