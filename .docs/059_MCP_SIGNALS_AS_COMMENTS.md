# MCP Signals as Comments — Unified Design

**Created:** 2026-02-11
**Updated:** 2026-02-11 (translation layer clarification)
**Replaces:** `task_signals` table approach
**Principle:** All agent signals are comments. No separate signal table.

## Summary

**MCP interface frozen** — agent-facing tools from 057 unchanged (`ralph.signal.done()`, etc.)

**Translation layer** — MCP server receives signal, generates human-readable text, writes to unified `task_comments` table with signal structure populated

**Benefits:**
- Unified chronological timeline (human comments + agent signals)
- Single table instead of two (simpler schema, simpler queries)
- Reuse existing comment UI components (just detect `signal_verb !== null`)
- Ralph's post-processing logic unchanged (just query different table)
- Future agent tasks can "review comments and reorganize task list" with full context

---

## Why Comments?

1. **Reuses existing infrastructure** — `task_comments` table already exists with timestamps, threading, task association
2. **Unified timeline** — human comments and agent signals appear together chronologically
3. **Simpler UI** — existing comment components render signals, no separate "Agent Activity" section
4. **Simpler schema** — one table instead of two
5. **Natural filtering** — can show/hide agent signals using comment filters

---

## Schema: task_comments (Unified)

```sql
CREATE TABLE task_comments (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  session_id TEXT,  -- Session identifier (for signals), NULL for human comments
  author TEXT NOT NULL,  -- 'human' | discipline name for agent signals

  -- Signal structure (all NULL for plain comments)
  signal_verb TEXT CHECK(signal_verb IN ('done','partial','stuck','ask','flag','learned','suggest','blocked') OR signal_verb IS NULL),
  signal_payload TEXT,  -- JSON payload, NULL for plain comments
  signal_answered TEXT,  -- For 'ask' verbs when answered, NULL otherwise

  -- Always present
  body TEXT NOT NULL,  -- Human-readable text (auto-generated for signals, user-written for comments)
  created TEXT NOT NULL DEFAULT (datetime('now'))
) STRICT;

CREATE INDEX idx_task_comments_task ON task_comments(task_id);
CREATE INDEX idx_task_comments_session ON task_comments(session_id);
CREATE INDEX idx_task_comments_verb ON task_comments(signal_verb);
```

**Two row types:**

1. **Plain comment** (human-authored):
   - `signal_verb = NULL`
   - `signal_payload = NULL`
   - Just `body` text and timestamp

2. **Agent signal** (MCP-generated):
   - `signal_verb` = one of 8 verbs
   - `signal_payload` = JSON from MCP call
   - `body` = auto-generated human-readable text
   - `session_id` = CLI session identifier

---

## Body Text Auto-Generation

MCP server generates human-readable `body` text from signal payloads:

### `done`
```typescript
// Payload: { summary: "..." }
// Body: "✓ **Done:** {summary}"
```
Example: `✓ **Done:** Implemented lobby WebSocket channel with join/leave/chat broadcasts. Fixed Phoenix 1.7.18 API change. Tests passing.`

### `partial`
```typescript
// Payload: { summary: "...", remaining: "..." }
// Body: "⊙ **Partial:** {summary}\n\n**Remaining:** {remaining}"
```
Example: `⊙ **Partial:** Wrote 12 of 18 planned test cases for CRUD operations\n\n**Remaining:** Need delete edge cases and bulk operations tests`

### `stuck`
```typescript
// Payload: { reason: "..." }
// Body: "⚠ **Stuck:** {reason}"
```
Example: `⚠ **Stuck:** Cannot proceed with delete tests until the empty URL validation question is answered — test assertions depend on the expected behavior`

### `ask`
```typescript
// Payload: { question: "...", blocking: true/false, preferred?: "...", options?: [...] }
// Body: "❓ **Ask ({blocking ? "blocking" : "non-blocking"}):** {question}"
//       + optional "\n\n**Preferred:** {preferred}"
//       + optional "\n\n**Options:**\n- {options.join('\n- ')}"
```
Example: `❓ **Ask (blocking):** Should empty URL strings be treated as validation errors or silently skipped?\n\n**Preferred:** Reject with error\n\n**Options:**\n- Reject with error\n- Skip silently\n- Auto-fill with placeholder URL`

### `flag`
```typescript
// Payload: { what: "...", severity: "...", category: "..." }
// Body: "🚩 **Flag ({severity}):** {what}\n\n**Category:** {category}"
```
Example: `🚩 **Flag (warning):** Empty URL string passes bookmark validation\n\n**Category:** bug`

### `learned`
```typescript
// Payload: { text: "...", kind: "...", scope?: "...", rationale?: "..." }
// Body: "💡 **Learned ({kind}):** {text}"
//       + optional "\n\n**Rationale:** {rationale}"
//       + optional "\n\n**Scope:** {scope}"
```
Example: `💡 **Learned (discovery):** localStorage has a 5MB quota per origin in Chromium\n\n**Rationale:** Needed to calculate max bookmarks before quota hit\n\n**Scope:** feature`

### `suggest`
```typescript
// Payload: { what: "...", kind: "...", why: "...", feature?: "..." }
// Body: "💭 **Suggest ({kind}):** {what}\n\n**Why:** {why}"
//       + optional "\n\n**Feature:** {feature}"
```
Example: `💭 **Suggest (new_task):** Add audit chain verification endpoint — stream through audit_logs, recompute hash chain, report first broken link\n\n**Why:** Verification logic is complex enough to be its own task — streaming millions of rows, pagination, caching\n\n**Feature:** audit-log`

### `blocked`
```typescript
// Payload: { on: "...", kind: "...", detail?: "..." }
// Body: "🚫 **Blocked ({kind}):** {on}"
//       + optional "\n\n**Detail:** {detail}"
```
Example: `🚫 **Blocked (external):** Redis service and credentials missing from environment\n\n**Detail:** No Redis instance found in .env.example, infrastructure code, or learnings from previous tasks`

---

## Querying and Rendering

**Frontend rendering:**

```typescript
function renderComment(comment: TaskComment): ReactNode {
  // Plain human comment
  if (comment.signal_verb === null) {
    return <CommentBubble author={comment.author} body={comment.body} created={comment.created} />
  }

  // Agent signal - render as signal card
  const payload = JSON.parse(comment.signal_payload)
  return <SignalCard
    verb={comment.signal_verb}
    payload={payload}
    body={comment.body}  // Pre-formatted human-readable text
    session={comment.session_id}
    created={comment.created}
  />
}
```

**Ralph post-processing** (unchanged from 057):

```rust
// Query signals for a session
let signals: Vec<Signal> = query_all!(
    "SELECT signal_verb, signal_payload, created
     FROM task_comments
     WHERE task_id = ? AND session_id = ? AND signal_verb IS NOT NULL
     ORDER BY created",
    task_id, session_id
)
.map(|row| Signal {
    verb: row.signal_verb,
    payload: serde_json::from_str(&row.signal_payload).unwrap(),
    created: row.created,
})
.collect();

// Find closing signal
let closing = signals.iter().rev().find(|s| matches!(s.verb, "done" | "partial" | "stuck"));
```

Ralph's post-processing logic from 057 works unchanged - just queries `WHERE signal_verb IS NOT NULL` instead of separate table.

---

## UI Rendering

**Comment component checks:**
1. Does `body` start with `[VERB]`?
2. If yes → render as **signal card** (colored border, verb icon, structured layout)
3. If no → render as **regular comment** (text bubble, markdown)

**Signal cards use existing comment UI:**
- Timestamp from `created`
- Author badge from `discipline`
- Session badge from `agent_task_id`
- Existing comment threading/reply system

**Filtering:**
- "Show agent signals" toggle in comment filter
- Filter by verb type (show only `ask`, show only `learned`, etc.)
- Filter by session (`agent_task_id`)

---

## MCP Server Translation Layer

**CRITICAL: MCP interface stays frozen.** Agent-facing tools remain exactly as specified in 057:

```typescript
ralph.signal.done({ summary: "..." })
ralph.signal.ask({ question: "...", blocking: true })
// ... all 8 verbs unchanged
```

**Translation layer** in MCP server:

1. Receive signal call with JSON payload (unchanged)
2. Generate human-readable body text from payload
3. Write to `task_comments` with signal structure populated

```typescript
// Agent calls (unchanged from 057):
ralph.signal.done({ summary: "Implemented feature X with tests passing" })

// MCP server translates to:
INSERT INTO task_comments (
  task_id, session_id, discipline, agent_task_id,
  signal_verb, signal_payload, body
) VALUES (
  env.RALPH_TASK_ID,
  env.RALPH_SESSION_ID,
  env.RALPH_DISCIPLINE,
  env.RALPH_AGENT_TASK_ID,
  'done',
  '{"summary":"Implemented feature X with tests passing"}',
  '✓ **Done:** Implemented feature X with tests passing'
)
```

The MCP server generates `body` text automatically. No changes to agent-facing API.

---

## Post-Processing (Unchanged from 057)

Ralph's loop_engine queries signals from unified table:

```rust
// Query all signals for this session
let signals: Vec<Signal> = query_all!(
    "SELECT signal_verb, signal_payload, created
     FROM task_comments
     WHERE task_id = ? AND session_id = ? AND signal_verb IS NOT NULL
     ORDER BY created",
    task_id, session_id
)
.map(|row| Signal {
    verb: row.signal_verb,
    payload: serde_json::from_str(&row.signal_payload).unwrap(),
    created: row.created,
})
.collect();

// Find closing signal (last done/partial/stuck)
let closing = signals.iter().rev().find(|s| matches!(s.verb, "done" | "partial" | "stuck"));
```

**All post-processing rules from 057 apply unchanged.** The only difference is querying `WHERE signal_verb IS NOT NULL` instead of separate `task_signals` table. The JSON payload structure and all verb semantics remain identical.

---

## Migration Path

### 1. Update task_comments schema

```sql
-- Add signal columns to task_comments
ALTER TABLE task_comments ADD COLUMN session_id TEXT;
ALTER TABLE task_comments ADD COLUMN author TEXT NOT NULL DEFAULT 'human';
ALTER TABLE task_comments ADD COLUMN signal_verb TEXT CHECK(signal_verb IN ('done','partial','stuck','ask','flag','learned','suggest','blocked') OR signal_verb IS NULL);
ALTER TABLE task_comments ADD COLUMN signal_payload TEXT;
ALTER TABLE task_comments ADD COLUMN signal_answered TEXT;

-- Migrate existing human comments (already have discipline, agent_task_id, priority, body, created)
-- They become plain comments with signal_verb = NULL, author = 'human' or discipline name

-- Create indexes
CREATE INDEX idx_task_comments_session ON task_comments(session_id);
CREATE INDEX idx_task_comments_verb ON task_comments(signal_verb);
```

### 2. Migrate task_signals data

```sql
-- Insert all task_signals as structured comments
INSERT INTO task_comments (task_id, session_id, author, signal_verb, signal_payload, body, created)
SELECT
    ts.task_id,
    ts.session_id,
    'agent',  -- or infer from session metadata
    ts.verb,
    ts.payload,
    -- Generate body text from payload (implementation detail)
    CASE ts.verb
        WHEN 'done' THEN '✓ **Done:** ' || json_extract(ts.payload, '$.summary')
        WHEN 'partial' THEN '⊙ **Partial:** ' || json_extract(ts.payload, '$.summary') || '\n\n**Remaining:** ' || json_extract(ts.payload, '$.remaining')
        -- ... other verbs
    END,
    ts.created_at
FROM task_signals ts;

-- Drop old table
DROP TABLE task_signals;
```

### 3. Update MCP server

Update `mcp-dev-server.ts` to write to unified `task_comments` table with signal structure populated.

### 4. Update UI components

Update comment rendering to detect `signal_verb !== null` and render as SignalCard vs CommentBubble.

### 5. Update loop_engine

Update post-processing to query `WHERE signal_verb IS NOT NULL` instead of separate table.

---

## Benefits Over Separate Table

| Feature | `task_signals` | `task_comments` |
|---------|----------------|-----------------|
| Unified timeline | ❌ Separate UI section | ✅ One chronological view |
| Human + agent dialogue | ❌ Siloed | ✅ Naturally threaded |
| Existing UI | ❌ New components | ✅ Reuse comment cards |
| Query complexity | ❌ JOIN signals + comments | ✅ Single table |
| Schema complexity | ❌ Two tables | ✅ One table |
| Filtering | ❌ Custom logic | ✅ Standard comment filters |

---

## Example Thread

All rows in `task_comments` table for task #4, session "sess-001":

| id | author | signal_verb | body | created |
|----|--------|-------------|------|---------|
| 101 | human | NULL | Also test unicode URLs please. | 2026-02-11 10:23:15 |
| 102 | frontend | flag | 🚩 **Flag (blocking):** Unicode URLs cause double-encoding in localStorage keys<br><br>**Category:** bug | 2026-02-11 10:25:42 |
| 103 | frontend | ask | ❓ **Ask (blocking):** Should empty URL strings be treated as validation errors or silently skipped?<br><br>**Preferred:** Reject with error<br><br>**Options:**<br>- Reject with error<br>- Skip silently<br>- Auto-fill with placeholder URL | 2026-02-11 10:26:10 |
| 104 | human | NULL | Reject with error — bookmarks without URLs are meaningless. | 2026-02-11 10:28:03 |
| 105 | frontend | done | ✓ **Done:** Added validation to reject empty URLs with clear error message. All 18 CRUD tests passing including unicode URLs. | 2026-02-11 10:35:21 |

**UI renders this as:**
1. Human comment bubble: "Also test unicode URLs please."
2. Red flag card: "🚩 Flag (blocking): Unicode URLs cause double-encoding..." with category badge
3. Yellow ask card: "❓ Ask (blocking): Should empty URL strings..." with options listed
4. Human comment bubble: "Reject with error — bookmarks without URLs are meaningless."
5. Green done card: "✓ Done: Added validation to reject empty URLs..." with checkmark icon

**Ralph's post-processing sees:**
- Signals 102, 103, 105 (filters `WHERE signal_verb IS NOT NULL`)
- Closing signal: `done` (#105) → task.status = "completed"
- Blocking ask (#103) + closing is done → informational only (agent resolved it)
- Flag (#102) stored for human review

All in one chronological timeline. Human comments and agent signals naturally interleaved. Simple, unified, elegant.
