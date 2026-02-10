# MCP Server Design Session — State of Mind Dump

**Created:** 2026-02-10
**Status:** Active brainstorm — not a spec, a thinking document

## What We Decided

1. **Each project gets a generated bun MCP server** at `.ralph/mcp/server.ts`
2. **Full tool suite, reductively filtered.** All 30 tools exist. Ralph disables irrelevant ones per session via `context.json`.
3. **Ralph manages the server deterministically.** Generate on init, regenerate on schema change, write context before each launch.
4. **`prompt-builder` and `mcp-builder` are siblings.** One assembles what Claude reads, the other assembles what Claude can do. They must be coordinated.

## What Exists Today (Reality Check)

### Current MCP Implementation (prompt-builder crate)

**17 tools defined** in `crates/prompt-builder/src/mcp/tools.rs`:

| Tool | Category | Used By Recipes |
|------|----------|-----------------|
| `create_feature` | Create | braindump |
| `create_discipline` | Create | braindump |
| `create_task` | Create | braindump, yap, enrichment |
| `update_task` | Update | yap, enrichment |
| `update_feature` | Update | ramble, opus_review |
| `update_discipline` | Update | discuss |
| `set_task_status` | Status | yap, task_exec, opus_review |
| `enrich_task` | Enrichment | enrichment |
| `list_features` | Read | braindump, yap, ramble, enrichment |
| `list_disciplines` | Read | braindump, discuss, enrichment |
| `list_tasks` | Read | yap, ramble, opus_review, enrichment |
| `append_learning` | State | task_exec, opus_review |
| `add_context_file` | State | task_exec |

**Missing from current implementation but in SPEC-070:**

| Tool | Why Missing | Priority |
|------|-------------|----------|
| `get_task` | Bash couldn't do rich single-item queries cleanly | High — agents need task details |
| `get_feature` | Same | High |
| `get_discipline` | Same | Medium |
| `delete_task` | Intentionally excluded from agents | Revisit — opus review may need this |
| `delete_feature` | Intentionally excluded | Low — too destructive |
| `delete_discipline` | Intentionally excluded | Low |
| `add_task_comment` | Bash couldn't handle well | High — retry history is critical |
| `update_task_comment` | Not needed by agents | Low |
| `delete_task_comment` | Not needed by agents | Low |
| `add_feature_context_file` | Only task-level was exposed | Medium |
| `append_progress` | Only `append_learning` existed | High — progress.txt is core |
| `read_learnings` | Agents couldn't read state files | High |
| `read_progress` | Same | High |
| `get_project_info` | Context was injected into prompt | Medium — redundant if in prompt |
| `get_project_progress` | Not implemented in sqlite-db either | Medium |

### Current Generation (Bash)

- `crates/prompt-builder/src/mcp/mod.rs` generates a bash script
- `crates/prompt-builder/src/mcp/tools.rs` defines tool schemas + bash handlers
- `crates/prompt-builder/src/mcp/helpers.rs` has bash helper functions
- Scripts use raw `sqlite3` CLI with `sed`-based parameter extraction
- Generated into a temp directory, pointed at via `--mcp-config`

### What Needs to Change

**New crate: `crates/mcp-builder/`** — Responsible for:
1. Generating `.ralph/mcp/server.ts` from tool definitions
2. Generating `.ralph/mcp/package.json`
3. Computing `disabled_tools` from recipe + discipline
4. Writing `.ralph/mcp/context.json` per session
5. Generating MCP config JSON (merging ralph server + discipline servers)
6. Running `bun install` when needed

**Prompt-builder changes:**
- Remove bash MCP generation (`mcp/mod.rs`, `mcp/tools.rs`, `mcp/helpers.rs`)
- Recipes declare which tools they ALLOW (already do this, just differently)
- Prompt sections must not reference tools the filter removed

## The Coordination Problem

The prompt says "When done, use `set_task_status` to mark the task complete." But what if the discipline filter disabled `set_task_status`? The prompt references a tool that doesn't exist.

**Solution:** The mcp-builder computes the final enabled tool set. The prompt-builder receives this set and conditionally includes/excludes instruction sections. They share a common "session plan" struct:

```
SessionPlan {
    recipe: Recipe,
    discipline: &Discipline,
    feature: &Feature,
    task: Option<&Task>,
    enabled_tools: HashSet<ToolName>,  // Computed by mcp-builder
    mcp_config_path: PathBuf,          // Written by mcp-builder
}
```

The prompt-builder reads `enabled_tools` and skips instructions for disabled tools. The `task_exec_instructions` section currently says:

```
When done:
1. Use set_task_status to mark task as done
2. Use append_learning to record what you learned
3. Commit your changes
```

If `append_learning` is disabled, instruction #2 gets omitted. The prompt and the toolset stay in sync.

## Scenarios That Need More Tools

This is the hard part. Every scenario below reveals nuance in what tools agents need.

### Scenario 1: Agent Discovers a Blocking Issue

Haiku is executing task #7 (build login form). It discovers the auth middleware returns `{ token }` not `{ user, token }`. Task #3 (auth API) was marked done but is actually incomplete.

**What the agent needs to do:**
- Add a comment on task #7 explaining the blocker
- Set task #7 status to `blocked`
- Optionally: set task #3 status back to `in_progress` or `pending`
- Optionally: create a NEW task to fix the middleware

**Tools needed:** `add_task_comment`, `set_task_status`, maybe `create_task`

**Question:** Should task_execution agents be able to reopen OTHER tasks? That's powerful but dangerous. Currently SPEC-070 only gives task_exec `set_task_status` for the current task. Should there be a `report_blocker` tool that's safer?

### Scenario 2: Agent Finds the Codebase Changed

Haiku is executing task #12. The prompt says "follow the pattern in RegisterForm.tsx" but RegisterForm.tsx was refactored by a previous task and no longer matches the expected pattern.

**What the agent needs to do:**
- Update the task's `context_files` to include the new file
- Maybe update the task's `hints` to reflect the new pattern
- Add a comment noting what changed

**Tools needed:** `update_task` (context_files, hints), `add_task_comment`

**Question:** Should task_execution agents be able to modify their own task record? SPEC-070 says no — only `set_task_status` and `add_task_comment`. But stale hints/context_files cause repeated failures.

### Scenario 3: Opus Review Finds Feature Scope Creep

Opus is reviewing the "authentication" feature. It notices 3 tasks are actually about "authorization" (role-based access control), not "authentication" (login/tokens).

**What the agent needs to do:**
- Create a new feature "authorization"
- Move those 3 tasks to the new feature (update task.feature)
- Update feature descriptions to clarify boundaries

**Tools needed:** `create_feature`, `update_task` (change feature), `update_feature`

**Question:** SPEC-070 doesn't give opus_review `create_feature`. Should it? Feature creation during review seems reasonable for scope management.

### Scenario 4: Enrichment Agent Realizes Task is Too Big

The enrichment agent is promoting task #15 from draft to pending. While writing pseudocode, it realizes this is actually 3 separate tasks.

**What the agent needs to do:**
- Create 3 new tasks (more specific)
- Set dependencies between them
- Delete or skip the original task #15
- Ensure the new tasks reference the right feature/discipline

**Tools needed:** `create_task`, `set_task_status` (skip original), `delete_task`

**Question:** Should enrichment be able to `delete_task`? Or should it `set_task_status` to `skipped` with a comment? Skipping is safer and preserves history.

### Scenario 5: Agent Needs to Read a Project File

Haiku is executing a task. The prompt injected `src/hooks/useAuth.ts` but the agent realizes it also needs to read `src/middleware/auth.ts` which wasn't in the context.

**Current behavior:** Claude CLI can read files natively via its built-in tools. The MCP server doesn't need a `read_file` tool because Claude already has one.

**But:** The agent might want to REGISTER that file as context for future sessions.

**Tools needed:** `add_context_file` (already exists for tasks), `add_feature_context_file`

### Scenario 6: Agent Encounters Rate Limits

This is Ralph's problem, not the MCP server's. Ralph monitors the Claude CLI JSON stream for `rate_limit_error` events. But what if the agent itself wants to signal "I'm stuck and should stop"?

**What the agent needs to do:**
- Signal that it can't complete the task right now
- Provide a reason

**New tool idea:** `signal_abort` — Agent tells Ralph to stop this session. Parameters: `reason`. Ralph reads this from the stream and handles it.

**Question:** Is this an MCP tool or a special output format? MCP tools return results to Claude, but this is Claude telling Ralph something. Could be a tool that writes to a known file that Ralph watches.

### Scenario 7: Multi-Task Awareness

Haiku is executing task #7. It knows task #8 is next (same feature, same discipline). It wants to leave a note for the next session about something it discovered.

**What the agent needs to do:**
- Add a comment on task #8 (the NEXT task, not the current one)

**Tools needed:** `add_task_comment` with no restriction on which task_id

**Question:** SPEC-070 gives task_exec `add_task_comment` but should it be restricted to the current task only? Or should agents be able to annotate ANY task? Annotating the next task is incredibly useful for continuity. Annotating random tasks is noise.

### Scenario 8: Braindump Creates Duplicate Features

User braindumps a project. The agent creates feature "auth" and feature "authentication". Two features for the same thing.

**What the agent needs to do:**
- Detect the duplicate
- Merge or delete one

**Tools needed:** `delete_feature` (or `merge_features`)

**Question:** Braindump currently can't delete. Should it be able to? The risk is creating-then-deleting in a loop. Maybe braindump should have `list_features` and be instructed to check for duplicates before creating.

### Scenario 9: Agent Wants to Understand Dependencies

Haiku is executing task #7 which depends on tasks #3 and #5. It wants to know what those tasks produced — not just their titles, but their comments (which contain completion summaries).

**Tools needed:** `get_task` (for other tasks, not just the current one)

**Question:** Should task_exec be able to `get_task` for any task, or only the current one + dependencies? Reading dependency details seems safe and useful.

### Scenario 10: Feature Knowledge Saturation

After 20 tasks in the "auth" feature, the learnings list is enormous. The agent appends a learning that's essentially a duplicate of an existing one.

**Current behavior:** `append_feature_learning` deduplicates via Jaccard similarity. If it's a near-duplicate, it increments `hit_count` instead of adding a new entry.

**But:** The agent doesn't know what learnings already exist. It might waste a tool call on a duplicate.

**Tools needed:** `get_feature` (which includes learnings) — so the agent can check before appending.

**Question:** Should `get_feature` be in task_exec's allowed list? Currently it's not (SPEC-070 only gives `add_feature_context_file` for features). But reading feature learnings before appending would reduce waste.

## Tool Ideas Not Yet in SPEC-070

| Tool | Description | Scenario |
|------|-------------|----------|
| `report_blocker` | Mark current task blocked with reason, optionally flag another task | Scenario 1 |
| `signal_abort` | Agent requests session termination | Scenario 6 |
| `get_dependency_details` | Get details of tasks this task depends on | Scenario 9 |
| `search_tasks` | Full-text search across task titles/descriptions | Agents looking for related work |
| `get_feature_learnings` | Read just the learnings for a feature (lighter than get_feature) | Scenario 10 |
| `update_task_context` | Update context_files/hints on current task only | Scenario 2 |
| `skip_task` | Sugar for set_task_status(skipped) + required comment | Scenario 4 |
| `spawn_subtasks` | Create multiple tasks at once, auto-link dependencies | Scenario 4 |
| `move_task_to_feature` | Change a task's feature assignment | Scenario 3 |

## The mcp-builder Crate

### Responsibilities

1. **Define the canonical tool registry** — All 30+ tools with names, descriptions, parameter schemas
2. **Define recipe filter profiles** — Which tools each recipe allows
3. **Define discipline override schema** — How disciplines can further restrict
4. **Generate `.ralph/mcp/server.ts`** — The actual bun MCP server source
5. **Generate `.ralph/mcp/package.json`** — Dependencies
6. **Compute session tool set** — `fn compute_enabled_tools(recipe, discipline) -> HashSet<ToolName>`
7. **Write `context.json`** — Per-session filter file
8. **Generate MCP config JSON** — For `--mcp-config` flag

### Relationship to prompt-builder

```
                    ┌─────────────────┐
                    │  SessionPlan    │
                    │                 │
                    │  recipe         │
                    │  discipline     │
                    │  feature        │
                    │  task           │
                    │  enabled_tools ←──── mcp-builder computes this
                    │  mcp_config    ←──── mcp-builder writes this
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │                             │
     ┌────────▼─────────┐        ┌─────────▼────────┐
     │  prompt-builder   │        │   mcp-builder    │
     │                   │        │                  │
     │  Reads enabled_   │        │  Generates       │
     │  tools to decide  │        │  server.ts       │
     │  which instruction│        │  context.json    │
     │  sections to      │        │  mcp-config.json │
     │  include          │        │                  │
     └──────────┬────────┘        └────────┬─────────┘
                │                          │
                ▼                          ▼
          prompt.txt              .ralph/mcp/context.json
          (what Claude reads)     (what Claude can do)
```

### Why Not Keep It In prompt-builder?

1. **Separation of concerns** — Prompt text vs tool availability are independent dimensions
2. **Independent regeneration** — Tool set changes when disciplines change; prompts change when task content changes
3. **Testability** — Test tool filtering without prompt assembly, and vice versa
4. **The server is a build artifact** — It's generated TypeScript, not a Rust runtime thing. Different enough to warrant its own crate.

## What I Still Don't Know

1. **How granular should discipline overrides be?** Per-tool? Per-category? Per-recipe-per-tool?
2. **Should the MCP server validate against context.json at call time too?** (Belt + suspenders: filter on `tools/list` AND reject calls to disabled tools)
3. **How does the server handle concurrent sessions?** Two terminals open for different tasks = two different context.json needs. Does each session get its own server instance?
4. **Should tools return structured data or formatted text?** Structured (JSON) is better for programmatic use. Formatted (markdown tables) is better for LLM consumption. Maybe both?
5. **How do we version the server?** If SPEC-070 adds a tool, old projects have old `server.ts`. Need a version check + regeneration trigger.
6. **What about the `resources/list` endpoint?** MCP resources are pull-on-demand data. Feature knowledge could be a resource instead of prompt-injected. But resources add complexity to the server.

## Next Steps

1. Enumerate MORE scenarios (this doc has 10, we need 30+)
2. For each scenario, identify exact tools needed
3. Stabilize the tool registry (no more than 40 tools — more = more confusion for Claude)
4. Design the `mcp-builder` crate API
5. Prototype ONE tool in bun (e.g. `list_tasks`) to validate the server architecture
6. Test with Claude CLI manually before automating
