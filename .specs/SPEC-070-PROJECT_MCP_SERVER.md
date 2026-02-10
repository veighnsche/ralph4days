# SPEC-070: Project-Scoped MCP Server

**Status:** Draft
**Created:** 2026-02-10
**Depends on:** SPEC-050 (Orchestration Philosophy)

## Core Concept

Each Ralph project gets a **generated bun MCP server** inside `.ralph/mcp/`. This server is the project's API surface for Claude. Ralph manages it deterministically — generating, regenerating, and configuring it as the project evolves.

The server exposes the **full suite** of tools for interacting with the project database. But each Claude session only sees a **subset** — Ralph writes a context file before each launch that disables tools irrelevant to the current recipe + discipline combination.

**Reductive, not additive.** The full toolkit exists. Ralph carves away what doesn't belong.

## Architecture

```
.ralph/
├── db/
│   └── ralph.db              ← SQLite database (existing)
├── mcp/
│   ├── server.ts             ← Generated bun MCP server
│   ├── package.json          ← bun dependencies (better-sqlite3, @modelcontextprotocol/sdk)
│   ├── bun.lockb             ← Lock file
│   └── context.json          ← Written by Ralph before each Claude session
├── learnings.txt
└── progress.txt
```

**Ralph generates `.ralph/mcp/` on project initialization** and regenerates when the schema or toolset changes (e.g. new discipline with custom MCP servers, migration bump).

**Ralph writes `.ralph/mcp/context.json` before every Claude CLI launch.** This file controls which tools are visible to the current session.

## Why Bun, Not Bash

The current implementation generates bash MCP scripts with `sed`-based JSON parsing and raw `sqlite3` calls. This works but is fragile:

- No proper JSON parsing (regex extraction breaks on nested objects)
- No parameterized queries (SQL injection surface via `json_escape`)
- No error handling beyond `set -euo pipefail`
- No type safety on tool parameters
- Painful to extend (every new tool = 20+ lines of bash)

Bun solves all of this:
- `better-sqlite3` for proper parameterized queries
- Native JSON parsing
- `@modelcontextprotocol/sdk` for protocol compliance
- TypeScript for type-safe tool definitions
- Starts in <50ms (negligible overhead)
- Single binary runtime (no node_modules bloat with bun)

## The Full Tool Suite

### Category 1: Task Tools

| Tool | Description | Parameters |
|------|-------------|------------|
| `list_tasks` | List all tasks with status, priority, feature, discipline | `filter_status?`, `filter_feature?`, `filter_discipline?` |
| `get_task` | Get full task details including comments and dependencies | `id` |
| `create_task` | Create a new task | `feature`, `discipline`, `title`, `description?`, `priority?`, `status?` (draft\|pending), `acceptance_criteria?`, `depends_on?`, `tags?`, `context_files?`, `output_artifacts?`, `hints?`, `estimated_turns?` |
| `update_task` | Update task fields | `id`, `title?`, `description?`, `priority?`, `acceptance_criteria?`, `depends_on?`, `tags?`, `context_files?`, `output_artifacts?`, `hints?`, `estimated_turns?` |
| `delete_task` | Delete a task (fails if other tasks depend on it) | `id` |
| `set_task_status` | Set task status | `id`, `status` (draft\|pending\|in_progress\|done\|blocked\|skipped) |
| `enrich_task` | Enrich draft task with pseudocode, promote to pending | `id`, `pseudocode`, `acceptance_criteria?`, `context_files?` |

### Category 2: Comment Tools

| Tool | Description | Parameters |
|------|-------------|------------|
| `add_task_comment` | Add a comment to a task | `task_id`, `author`, `body`, `discipline?`, `priority?` |
| `update_task_comment` | Edit an existing comment | `task_id`, `comment_id`, `body` |
| `delete_task_comment` | Delete a comment | `task_id`, `comment_id` |

### Category 3: Feature Tools

| Tool | Description | Parameters |
|------|-------------|------------|
| `list_features` | List all features with descriptions | (none) |
| `get_feature` | Get full feature details including learnings and context files | `name` |
| `create_feature` | Create a new feature | `name`, `display_name`, `description?`, `acronym?`, `knowledge_paths?`, `context_files?`, `architecture?`, `boundaries?`, `dependencies?` |
| `update_feature` | Update feature fields | `name`, `display_name?`, `description?`, `acronym?`, `knowledge_paths?`, `context_files?`, `architecture?`, `boundaries?`, `dependencies?` |
| `delete_feature` | Delete a feature (fails if tasks reference it) | `name` |
| `append_feature_learning` | Add a learning to a feature (deduplicates automatically) | `feature_name`, `text`, `source?` (auto\|agent\|human), `reason?`, `task_id?` |
| `add_feature_context_file` | Register a relevant file path on a feature | `feature_name`, `file_path` |

### Category 4: Discipline Tools

| Tool | Description | Parameters |
|------|-------------|------------|
| `list_disciplines` | List all disciplines | (none) |
| `get_discipline` | Get full discipline details | `name` |
| `create_discipline` | Create a new discipline | `name`, `display_name`, `icon`, `color`, `acronym?`, `system_prompt?`, `skills?`, `conventions?` |
| `update_discipline` | Update discipline fields | `name`, `display_name?`, `icon?`, `color?`, `system_prompt?`, `skills?`, `conventions?` |
| `delete_discipline` | Delete a discipline (fails if tasks reference it) | `name` |

### Category 5: Project Tools

| Tool | Description | Parameters |
|------|-------------|------------|
| `get_project_info` | Get project title, description, creation date | (none) |
| `get_project_progress` | Get task completion stats (total, done, by status, by feature) | (none) |

### Category 6: State File Tools

| Tool | Description | Parameters |
|------|-------------|------------|
| `append_learning` | Append to `.ralph/learnings.txt` | `text` |
| `read_learnings` | Read `.ralph/learnings.txt` | (none) |
| `append_progress` | Append to `.ralph/progress.txt` | `text` |
| `read_progress` | Read `.ralph/progress.txt` | (none) |

**Total: 30 tools**

## The Reductive Filter

### How It Works

Before each Claude CLI launch, Ralph writes `.ralph/mcp/context.json`:

```json
{
  "session_id": "task-exec-7-2026-02-10T14:30:00Z",
  "recipe": "task_execution",
  "discipline": "frontend",
  "feature": "authentication",
  "task_id": 7,
  "disabled_tools": [
    "create_feature",
    "delete_feature",
    "update_feature",
    "create_discipline",
    "delete_discipline",
    "update_discipline",
    "delete_task",
    "create_task",
    "update_task",
    "enrich_task",
    "delete_task_comment",
    "update_task_comment"
  ]
}
```

The MCP server reads `context.json` on `tools/list` and filters out disabled tools. Claude never sees them — it can't even attempt to call them.

### Recipe Filters

Each recipe defines which tool categories are **allowed**. Everything else is disabled.

#### `braindump` — Project setup from unstructured input
**Purpose:** Parse user ramblings into features, disciplines, and tasks.
```
ALLOWED:
  create_feature, create_discipline, create_task
  list_features, list_disciplines, list_tasks
  get_feature, get_discipline
  get_project_info

DISABLED: everything else
  (no updates, no deletes, no comments, no status changes, no state files)
```
**Rationale:** Braindump creates the project skeleton. It shouldn't modify existing data or execute tasks.

#### `yap` — Task-focused conversation
**Purpose:** Refine and create tasks within existing features/disciplines.
```
ALLOWED:
  list_tasks, get_task, create_task, update_task
  list_features, list_disciplines
  set_task_status
  get_project_info

DISABLED: everything else
  (no feature/discipline CRUD, no comments, no enrichment, no state files)
```

#### `ramble` — Feature refinement
**Purpose:** Discuss and refine a specific feature's design.
```
ALLOWED:
  list_features, get_feature, create_feature, update_feature
  append_feature_learning, add_feature_context_file
  list_tasks
  get_project_info

DISABLED: everything else
  (no task CRUD, no discipline CRUD, no state files)
```

#### `discuss` — Discipline configuration
**Purpose:** Refine a discipline's persona, conventions, MCP servers.
```
ALLOWED:
  list_disciplines, get_discipline, update_discipline
  get_project_info

DISABLED: everything else
  (read-heavy, only updates to targeted discipline)
```

#### `task_execution` — Haiku executing a single task
**Purpose:** Execute one task. Minimal tool surface for maximum focus.
```
ALLOWED:
  get_task
  set_task_status
  add_task_comment
  append_learning, append_progress
  add_feature_context_file
  read_learnings, read_progress
  get_project_info

DISABLED: everything else
  (no creation, no deletion, no updates to tasks/features/disciplines)
```
**Rationale:** Task execution agents should ONLY: do the work, mark it done, record what they learned, and note progress. They should not restructure the project.

#### `opus_review` — Periodic Opus review of feature progress
**Purpose:** Review completed work, adjust priorities, record learnings.
```
ALLOWED:
  list_tasks, get_task
  set_task_status
  update_task (priority and description only)
  create_task (can spawn follow-up tasks)
  add_task_comment
  list_features, get_feature, update_feature
  append_feature_learning
  append_learning, append_progress
  read_learnings, read_progress
  get_project_info, get_project_progress

DISABLED:
  delete_*, create_feature, create_discipline, *_discipline, enrich_task
```
**Rationale:** Opus reviews are powerful but scoped. They can adjust the plan (new tasks, priority changes, learnings) but shouldn't restructure features or disciplines.

#### `enrichment` — Draft task promotion
**Purpose:** Add pseudocode and acceptance criteria to draft tasks.
```
ALLOWED:
  list_tasks, get_task
  enrich_task
  update_task
  create_task (can split a draft into multiple tasks)
  list_features, get_feature
  list_disciplines
  get_project_info

DISABLED: everything else
  (no status changes, no deletes, no comments, no state files)
```

### Discipline Overrides

On top of recipe filters, disciplines can further restrict tools via `disabled_tools` in the discipline record:

```sql
-- Example: documentation discipline can't create tasks or change priorities
UPDATE disciplines
SET mcp_tool_overrides = '{"additional_disabled": ["create_task", "delete_task", "set_task_status"]}'
WHERE name = 'documentation';
```

The final filter is:

```
disabled = recipe_disabled_tools ∪ discipline_additional_disabled
enabled = ALL_TOOLS - disabled
```

This means a discipline can only REMOVE tools that the recipe allows — it can never ADD tools the recipe disabled. Strictly reductive.

### Filter Precedence

```
1. Start with ALL 30 tools
2. Remove tools not in recipe's ALLOWED list
3. Remove tools in discipline's additional_disabled list
4. Result = what Claude sees in tools/list
```

## Server Implementation

### `server.ts` (Generated)

```typescript
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import Database from "better-sqlite3";
import { readFileSync } from "fs";
import { join } from "path";

const PROJECT_PATH = "__PROJECT_PATH__";  // Ralph replaces on generation
const DB_PATH = join(PROJECT_PATH, ".ralph", "db", "ralph.db");
const CONTEXT_PATH = join(PROJECT_PATH, ".ralph", "mcp", "context.json");

const db = new Database(DB_PATH, { readonly: false });
db.pragma("journal_mode = WAL");
db.pragma("foreign_keys = ON");

// Load context to determine which tools are enabled
const context = JSON.parse(readFileSync(CONTEXT_PATH, "utf-8"));
const disabledTools = new Set(context.disabled_tools ?? []);

const server = new McpServer({
  name: "ralph-project",
  version: "1.0.0",
});

// Tool registration helper — skips if disabled
function registerTool(name, schema, handler) {
  if (disabledTools.has(name)) return;
  server.tool(name, schema, handler);
}

// --- Task Tools ---

registerTool("list_tasks", { /* ... */ }, async (params) => {
  const rows = db.prepare(`
    SELECT t.id, t.title, t.status, t.priority, t.feature, t.discipline,
           f.display_name as feature_display, d.display_name as discipline_display
    FROM tasks t
    JOIN features f ON t.feature = f.name
    JOIN disciplines d ON t.discipline = d.name
    ORDER BY t.id
  `).all();
  return { content: [{ type: "text", text: JSON.stringify(rows, null, 2) }] };
});

// ... 29 more tools ...

const transport = new StdioServerTransport();
await server.connect(transport);
```

### `package.json` (Generated)

```json
{
  "name": "ralph-mcp-server",
  "private": true,
  "type": "module",
  "dependencies": {
    "@modelcontextprotocol/sdk": "^1.0.0",
    "better-sqlite3": "^11.0.0"
  }
}
```

### MCP Config (Generated per session)

Ralph generates a temp config for `--mcp-config`:

```json
{
  "mcpServers": {
    "ralph": {
      "command": "bun",
      "args": ["run", "/path/to/project/.ralph/mcp/server.ts"]
    }
  }
}
```

If the discipline defines additional MCP servers (e.g. `shadcn-ui` for frontend), they're merged:

```json
{
  "mcpServers": {
    "ralph": {
      "command": "bun",
      "args": ["run", "/path/to/project/.ralph/mcp/server.ts"]
    },
    "shadcn-ui": {
      "command": "npx",
      "args": ["-y", "@anthropic-ai/shadcn-ui-mcp"]
    }
  }
}
```

## Lifecycle

### Generation

Ralph generates `.ralph/mcp/` when:
1. **Project initialization** — `initialize_ralph_project()` creates the MCP server alongside the database
2. **Schema migration** — After running new SQLite migrations, regenerate to match new schema
3. **Manual regeneration** — User triggers via UI (e.g. after editing discipline MCP server configs)

Generation is deterministic: same project state → same `server.ts` output.

### Session Launch

Before each Claude CLI launch, Ralph:
1. Computes `disabled_tools` from recipe + discipline
2. Writes `.ralph/mcp/context.json`
3. Generates temp MCP config JSON (merging ralph server + discipline servers)
4. Launches: `claude --mcp-config /tmp/ralph-session-{id}.json --output-format stream-json ...`

### Dependency Installation

On first generation, Ralph runs `bun install` in `.ralph/mcp/`. This creates `bun.lockb` and installs `better-sqlite3` + `@modelcontextprotocol/sdk`. Subsequent regenerations skip install if `package.json` hasn't changed.

## Resources (Future)

MCP resources expose read-only data that Claude can pull on-demand (instead of being injected into the prompt). Future expansion:

| Resource URI | Description |
|-------------|-------------|
| `ralph://features/{name}` | Full feature details |
| `ralph://disciplines/{name}` | Full discipline details |
| `ralph://tasks/{id}` | Full task details |
| `ralph://progress` | progress.txt contents |
| `ralph://learnings` | learnings.txt contents |

Resources are always available regardless of tool filtering — they're read-only and add zero risk.

## Migration Path

### Phase 1: Generate server, keep bash fallback
- Ralph generates `.ralph/mcp/server.ts` with full tool suite
- Prompt builder still generates bash scripts as fallback
- Test bun server with manual terminal sessions
- Validate tool parity between bash and bun implementations

### Phase 2: Switch to bun server, remove bash generation
- All prompt recipes use bun server
- Remove bash MCP generation from `crates/prompt-builder/src/mcp/`
- `context.json` filtering goes live

### Phase 3: Add resources, optimize prompt size
- Feature knowledge delivered via MCP resources instead of prompt injection
- Prompt shrinks (less file content injected)
- Claude pulls what it needs on-demand

## Open Questions

1. **Should `context.json` include the task/feature context too?** The server could pre-load task details and expose them as a resource, reducing prompt size.
2. **Should the server support subscriptions?** MCP supports notifications — the server could notify when another session modifies the database.
3. **Should `bun install` happen at project init or on first launch?** Init is cleaner but adds latency to project creation.
4. **Version pinning** — Should `package.json` pin exact versions or use ranges? Exact is reproducible but requires regeneration on updates.
