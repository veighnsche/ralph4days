# SPEC-050: Ralph Orchestration Philosophy

**Status:** Active
**Created:** 2026-02-06
**Updated:** 2026-02-06

## Core Principle: Ralph is the Thinnest Wrapper

**Ralph is an ORCHESTRATOR, not a replacement for Claude Code.**

Ralph does NOT re-implement functionalities that Claude Code already provides. Ralph's role is to:
1. Play tasks in sequence from the database
2. Launch Claude Code instances with appropriate flags and MCP servers for each task
3. Generate deterministic prompts based on task content
4. Monitor Claude Code's output and state
5. Interact with Claude Code **exactly as a human would**

**RALPH IS NOT AI.** Ralph only implements coded, deterministic behavior. Ralph plays tasks sequentially, not in a loop. A loop emerges only if tasks create more tasks.

## Sequential Task Execution Model

### Not a Loop — A Pipeline

**Ralph plays tasks in sequence, not in a loop.** Each Claude Code session executes one pending task and exits. Ralph then launches the next session for the next task. This is a linear pipeline.

**A loop emerges only when tasks create more tasks.** When a task adds new pending tasks to the queue (including a task whose job is to generate more tasks), the pipeline keeps running because there's always a next task. But this is an emergent property, not a built-in looping mechanism.

### Human vs Agent Task Creation

**Humans DO NOT write structured tasks.** Humans provide:
- Unorganized yappings
- Ramblings about what they want
- Semi-structured Q&A responses
- Rough ideas and requirements

**Agents write structured data.** Agents parse human ramblings into:
- Structured task records (YAML)
- Features, disciplines, priorities
- Dependencies and acceptance criteria
- Organized, actionable items

### Two Task Creation Paths

Ralph must support TWO distinct task creation workflows:

#### 1. Agent-Created Tasks: "+ Create Task" (Robot Icon)
- Structured form pre-filled by an agent
- Agent has already parsed requirements
- Direct YAML database write
- Used during sequential task execution

#### 2. Human-Explained Tasks: "+ Explain Tasks" (Human Icon)
- Semi-structured Q&A form
- Human provides unorganized input
- Form turns into a deterministic prompt
- Prompt sent to Claude Code instance
- Claude Code parses and creates tasks

## Ralph's Interaction Model with Claude Code

### As-If-Human Principle

Ralph interacts with Claude Code **as if Ralph was a human user launching one-off Claude sessions**:

```
Ralph Task Queue → Pick Next Pending Task → Deterministic Prompt → Claude CLI Session → Execute Task → Task Complete → Repeat
```

Ralph can:
- Launch `claude` CLI with flags: `--output-format stream-json`, `--max-turns N`, `--project /path`
- Restart Claude instances with different MCP servers
- Insert prompts into Claude Code's input
- Run slash commands (e.g., `/commit`, `/help`)
- Monitor Claude's JSON stream output
- Parse Claude's responses for completion/errors

Ralph cannot:
- Modify Claude Code's behavior
- Replace Claude Code's agent capabilities
- Implement its own AI/LLM logic
- Override Claude Code's tool execution

### What Ralph Provides

Ralph's value-add:
1. **Project structure enforcement** - `.ralph/db/` YAML schema
2. **Task sequence orchestration** - Start/stop/pause/resume logic, task queue management
3. **Deterministic prompt generation** - Task aggregation, context injection
4. **State persistence** - Execution state, stagnation detection
5. **UI for monitoring** - Real-time Claude output display
6. **Human input collection** - Forms → Prompts

Ralph does NOT:
- Parse natural language (that's Claude's job)
- Make AI decisions (only deterministic code paths)
- Execute tools (Claude Code handles that)
- Manage git operations (Claude Code handles that)

## Dynamic MCP Server Creation

### Core to "Ralphing"

**Dynamic MCP server generation is critical to Ralph's orchestration model.**

The essence of "ralphing" is crafting the perfect environment for Claude Haiku to execute tasks:
1. **Perfect prompt** - deterministic, context-rich, task-specific
2. **Perfect toolset** - exactly the MCP tools needed, nothing more
3. **Perfect configuration** - right model, right flags, right MCP servers

### Why Dynamic MCP Servers?

Claude CLI doesn't pre-load MCP servers from a global config when launched with `--project`. Ralph needs to:
- Expose Ralph-specific tools to Claude Code instances
- Give Claude Code access to `.ralph/db/` YAML files as MCP resources
- Provide task creation/editing as MCP tools
- Change available tools based on what Claude needs to do

**Restarting Claude Code is OK.** When MCP server configuration changes, Ralph simply:
1. Stops current Claude instance
2. Regenerates MCP server scripts
3. Regenerates MCP config JSON
4. Relaunches Claude with new `--mcp-config` flag

### How Ralph Generates MCP Servers

Ralph creates MCP servers in **bash** (no pre-compilation needed):

```bash
# Ralph generates: /tmp/ralph-mcp-servers/ralph-db-server.sh
#!/usr/bin/env bash

# MCP protocol: JSON-RPC over stdio
while IFS= read -r line; do
    case "$line" in
        *'"method":"tools/list"'*)
            echo '{"tools":[{"name":"create_task","description":"Create a task in .ralph/db/tasks.yaml","inputSchema":{...}}]}'
            ;;
        *'"method":"tools/call"'*"create_task"*)
            # Parse tool arguments from JSON
            # Write to .ralph/db/tasks.yaml
            # Return success/error
            ;;
        *'"method":"resources/list"'*)
            echo '{"resources":[{"uri":"ralph://db/tasks.yaml","name":"Tasks Database"}]}'
            ;;
    esac
done
```

Ralph then generates MCP config:

```json
{
  "mcpServers": {
    "ralph-db": {
      "command": "bash",
      "args": ["/tmp/ralph-mcp-servers/ralph-db-server.sh"]
    }
  }
}
```

### MCP Servers Interact with Ralph

Ralph's generated MCP servers can:
- **Read** `.ralph/db/*.yaml` files and expose as MCP resources
- **Write** to `.ralph/db/*.yaml` files via MCP tools
- **Query** Ralph's backend via IPC (if needed)
- **Validate** task/feature/discipline data before writing
- **Lock** files during writes (using fs2 pattern from yaml_db)

This creates a feedback loop:
```
Claude Code (MCP tool call)
  → Ralph MCP Server (bash script)
    → Ralph Database (.ralph/db/tasks.yaml)
      → Ralph UI (file watcher / manual refresh)
        → User sees new tasks
```

### When to Regenerate MCP Servers

Ralph regenerates MCP servers and restarts Claude when:
1. **Task type changes** - "create tasks" needs different tools than "review code"
2. **User switches modes** - different MCP servers for task execution vs manual mode
3. **Project changes** - different `.ralph/db/` path needs new resource URIs
4. **Error recovery** - Claude instance crashed or stalled

### Example: Task Creation MCP Server

When user clicks "Explain Tasks", Ralph:

1. **Generates `ralph-task-creator.sh`:**
```bash
#!/usr/bin/env bash
# MCP server for task creation
# Exposes tools: create_task, list_features, list_disciplines
# Exposes resources: ralph://db/tasks.yaml, ralph://db/features.yaml
```

2. **Generates `/tmp/ralph-mcp-config.json`:**
```json
{
  "mcpServers": {
    "ralph-tasks": {
      "command": "bash",
      "args": ["/tmp/ralph-mcp-servers/ralph-task-creator.sh"]
    }
  }
}
```

3. **Launches Claude with MCP config:**
```bash
claude \
  --mcp-config /tmp/ralph-mcp-config.json \
  --output-format stream-json \
  --max-turns 20 \
  --project /path/to/user/project \
  < task-creation-prompt.txt
```

4. **Claude Code now has access to:**
   - `create_task(feature, discipline, title, description, ...)` tool
   - `ralph://db/tasks.yaml` resource
   - All standard Claude Code capabilities

### Benefits of Dynamic MCP Generation

1. **No global config pollution** - each Claude instance gets exactly what it needs
2. **Task-specific toolsets** - task creation gets different tools than code review
3. **Ralph-specific integration** - MCP servers understand `.ralph/db/` schema
4. **Simple implementation** - bash scripts are easy to generate/modify
5. **Restart flexibility** - regenerate and restart anytime without user config changes

### Implementation Strategy

**Phase 1: Single static MCP server**
- Generate one `ralph-db-server.sh` with all tools
- Always use same MCP config
- Focus on proving the concept

**Phase 2: Dynamic server selection**
- Generate different servers for different tasks
- Switch MCP config based on Ralph's current mode
- Optimize tool availability

**Phase 3: Advanced features**
- MCP servers can query Ralph backend via IPC
- Bi-directional communication (Claude ↔ Ralph)
- Progress updates via MCP notifications

### Technical Notes

- MCP protocol is JSON-RPC 2.0 over stdio
- Bash is sufficient for simple request/response servers
- For complex logic, Ralph can generate Python/Node MCP servers
- MCP servers are stateless (read/write to `.ralph/db/` for persistence)
- File locking prevents race conditions (same fs2 pattern as yaml_db)

## Human Task Explanation Workflow

### User Journey: "Explain Tasks"

1. **User clicks "+ Explain Tasks"** (in empty state or via button)
2. **Ralph shows Q&A form:**
   - "What do you want to accomplish?"
   - "What features are involved?"
   - "Any specific requirements or constraints?"
   - "What does success look like?"
   - (Optional additional freeform text box)

3. **User fills out form** (unorganized, rambling is fine)

4. **Ralph generates deterministic prompt:**
```
You are helping structure tasks for a project tracked in .ralph/db/.

The user wants to create tasks. Here's what they said:

**What they want to accomplish:**
{user_answer_1}

**Features involved:**
{user_answer_2}

**Requirements:**
{user_answer_3}

**Success criteria:**
{user_answer_4}

**Additional context:**
{user_answer_5}

Please:
1. Read the current project state from .ralph/db/ (tasks.yaml, features.yaml, metadata.yaml)
2. Parse the user's input into structured tasks
3. Create task records using the appropriate commands/tools
4. Ensure features and disciplines are populated
5. Set appropriate priorities and dependencies

Follow the Ralph YAML schema defined in .specs/SPEC-010-DATABASE_SCHEMA.md.
```

5. **Ralph launches Claude Code:**
```bash
claude \
  --output-format stream-json \
  --max-turns 20 \
  --project /path/to/user/project \
  < prompt.txt
```

6. **Ralph monitors Claude's JSON stream:**
   - Shows output in UI
   - Detects completion/errors
   - Updates UI state when tasks created

7. **User sees tasks appear** in task list (via file watcher or manual refresh)

## Implementation Notes

### Frontend Changes Needed

1. **Replace single "Create Task" button with dual buttons:**
   - `<Button variant="outline">+ Create Task</Button>` (robot icon) - opens CreateTaskModal
   - `<Button variant="default">+ Explain Tasks</Button>` (message-square icon) - opens ExplainTasksModal

2. **New ExplainTasksModal component:**
   - Multi-step form or single-page Q&A
   - Textarea fields for rambling input
   - Submit → sends to backend command

3. **New Claude Output Panel:**
   - Shows real-time JSON stream from Claude CLI
   - Status indicators (running, completed, error)
   - Cancel button to stop Claude process

### Backend Changes Needed

1. **New command: `explain_tasks_to_claude`**
   - Takes user input from form
   - Generates deterministic prompt
   - Launches Claude CLI subprocess
   - Streams output back to frontend
   - Returns when complete or error

2. **Subprocess management:**
   - Track Claude CLI process PID
   - Handle cancellation (SIGTERM)
   - Stream stdout/stderr parsing
   - Detect completion from JSON events

3. **File watcher (optional but nice):**
   - Watch `.ralph/db/tasks.yaml` for changes
   - Emit event to frontend when tasks added
   - Frontend auto-refreshes task list

## UI/UX Principles

### Empty States

When task list is empty, show:
```
┌────────────────────────────────────────┐
│  No tasks yet                          │
│                                        │
│  Get started:                          │
│                                        │
│  [🤖 Create Task]  Agent-structured    │
│  [💬 Explain Tasks]  Human ramblings  │
└────────────────────────────────────────┘
```

### During Claude Execution

Show real-time output:
```
┌────────────────────────────────────────┐
│  🔄 Claude is creating tasks...        │
│                                        │
│  [Streaming output panel]              │
│  > Reading .ralph/db/tasks.yaml...     │
│  > Creating feature: authentication    │
│  > Creating task #1: Login form UI     │
│                                        │
│  [Cancel]                              │
└────────────────────────────────────────┘
```

## Related Specs

- SPEC-010: Database Schema (YAML format)
- SPEC-020: Task Execution Engine (sequential task processing)
- SPEC-030: CLAUDE.md Management (prompt context)
- SPEC-040: Subprocess Management (Claude CLI invocation)

## Open Questions

1. Should "Explain Tasks" be available during running task execution or only when idle?
2. How to handle conflicts if Claude creates tasks while execution is running?
3. Should we show a "use this prompt" option for advanced users who want to copy/paste?
4. What MCP servers should be available to task-creation Claude instances?

## Summary

Ralph is a thin orchestrator that:
- Plays tasks sequentially from the database
- Generates deterministic prompts per task
- Launches Claude Code instances
- Monitors and displays output
- **Never replaces Claude Code's capabilities**

A loop emerges naturally when tasks create more tasks. By default, Ralph just plays the task queue in sequence.

This keeps Ralph simple, maintainable, and focused on what it does best: orchestrating sequential task execution.
