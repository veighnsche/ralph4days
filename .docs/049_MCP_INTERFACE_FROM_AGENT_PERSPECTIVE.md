# MCP Interface: From the Agent's Perspective

**Created:** 2026-02-11
**Status:** Active thinking — supersedes SPEC-070's CRUD approach

## The Spec Is Wrong

SPEC-070 designed the MCP as a database API: 30 tools across 6 categories, filtered per recipe. That's Ralph's vocabulary, not the agent's. It forces the prompt to teach Claude about Ralph's schema — burning tokens to explain something that should be obvious.

The right question isn't "what database operations does the agent need?" It's **"what does an agent naturally do when executing a task?"**

## The Natural Process Flow

Claude gets launched by Ralph. Here's what happens in Claude's head:

### Phase 1: Orientation — "Where am I? What's my job?"

Read the system prompt. Get the task details. Understand the project context, what's been done, what files matter, what previous agents learned, what the acceptance criteria are.

**MCP need:** Give me my assignment with full context.

### Phase 2: Understanding — "Let me look at the code"

Read files. Search the codebase. Understand the existing patterns. This is entirely Claude's built-in tools — Read, Grep, Glob, Bash. **The MCP server has nothing to do with this phase.**

### Phase 3: Execution — "Do the work"

Write code, run tests, iterate. Again — Claude-native tools. No MCP needed. This is where 90% of the agent's time goes.

### Phase 4: Signaling — "Something happened"

The unexpected. The API shape isn't what the task said. A dependency is broken. The approach won't work. A file the next agent should know about. **This is where the agent needs to talk back to Ralph.** Not through database CRUD — through intent.

### Phase 5: Closing — "I'm done" or "I'm stuck"

Mark completion. Leave a summary. Record what was learned. Or: signal that this can't be completed and why.

## The Insight

**The agent only needs MCP at phases 1, 4, and 5.** Orientation, signaling, and closing. The actual work (phases 2-3) is all Claude-native.

So the MCP tools shouldn't be a database API. They should be:

- **"What's my job?"** — get my task + context + history
- **"I found something"** — leave a note, flag a blocker, register a file
- **"I'm done"** / **"I'm stuck"** — signal completion or failure

**Three verbs, not thirty.**

## Why This Matters for Tokens

SPEC-070's approach requires prompt sections explaining when to use each of 30 tools, what parameters they take, which ones are available in this session. That's hundreds of tokens spent teaching Claude about Ralph.

If the tools are obvious — `my_task`, `leave_note`, `im_done`, `im_stuck` — the tool descriptions themselves are the documentation. Zero prompt tokens explaining the MCP interface. Claude already knows what "I'm done" means.

## Why This Matters for Reliability

A CRUD API gives the agent power to restructure the project. `create_task`, `update_feature`, `delete_discipline` — these are admin operations. A task execution agent has no business touching them. SPEC-070 solved this with reductive filtering (disable 18 of 30 tools per session). But that's fighting the wrong problem. If you only expose 4 intent-based tools, there's nothing to filter.

## What About the Other Recipes?

Task execution is the hot path — it runs hundreds of times. The other recipes (braindump, yap, ramble, discuss, enrichment, opus_review) are human-interactive or run infrequently. They might still use a richer tool surface. But even they benefit from intent-oriented design over CRUD.

The key realization: **different recipes might need fundamentally different MCP servers, not the same server with different filters.**

## The Feedback Loop

It's not twins. It's two halves of a loop.

```
Prompt-builder (INTAKE)          MCP server (EXHAUST)
        │                               ▲
        │  pushes context into Claude    │  receives signals from Claude
        ▼                               │
        ┌───────────────────────────────┐
        │         Claude Code           │
        │                               │
        │   reads code (native tools)   │
        │   writes code (native tools)  │
        │   runs tests (native tools)   │
        │                               │
        │   90% of the work happens     │
        │   here with zero MCP          │
        └───────────────────────────────┘
```

**Prompt-builder = intake.** Assembles context from many sources (database, files, learnings, previous attempts, feature knowledge) → pushes it into Claude as the system prompt. This is everything Claude needs to START.

**MCP server = exhaust.** Receives signals from Claude about what happened, what went wrong, what was learned. This is everything Claude needs to REPORT BACK.

**The killer detail: the MCP server is generated per-task.** It already has all the metadata baked in — task_id, feature, discipline, session context. So Claude doesn't say `set_task_status(id=7, status="done")`. Claude says `im_done()`. The server already knows it's task 7, in the auth feature, assigned to the frontend discipline. It translates the intent into the right CRUD operation and places it in the right spot in the database.

The agent never learns Ralph. It just says natural things — "I'm done", "I'm stuck", "I found something" — and the pre-bound server translates that into correct database operations with full metadata.

## Open Questions

1. What exactly does `my_task` return? Just the task row, or a pre-assembled context bundle (task + feature learnings + dependency summaries + previous comments)?
2. Should `leave_note` be one tool or split by intent? (`note_for_next_agent`, `flag_blocker`, `register_file` — still intent-based, just more specific)
3. How does `im_done` differ from the current `set_task_status`? Does it include a completion summary? Does it auto-append to progress.txt?
4. What does `im_stuck` trigger on Ralph's side? Retry? Skip? Escalate to Opus?
5. Do the human-interactive recipes (braindump, yap) even need MCP, or is the prompt + Claude's native tools enough?
