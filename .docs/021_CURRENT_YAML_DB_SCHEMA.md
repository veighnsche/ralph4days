# Current YAML DB Schema (As Implemented)

**Created:** 2026-02-07
**Status:** Reference

This document captures the **current YAML database schema** as implemented in code. This is the source of truth for behavior parity when porting to SQLite.

**Note:** There is no production data to migrate. All data to date is mock. Do not spend time on data migration logic.

## Source Files

- `crates/yaml-db/src/lib.rs` (Task, TaskComment, enums, EnrichedTask, GroupStats, ProjectProgress)
- `crates/yaml-db/src/entity.rs` (EntityFile trait, atomic write pattern)
- `crates/yaml-db/src/features.rs` (Feature, FeaturesFile)
- `crates/yaml-db/src/disciplines.rs` (Discipline, DisciplinesFile, 10 defaults)
- `crates/yaml-db/src/metadata.rs` (ProjectMetadata, MetadataFile, counters)
- `crates/yaml-db/src/acronym.rs` (acronym format validation)
- `crates/yaml-db/src/migration.rs` (auto-generate acronyms for legacy projects)
- `crates/yaml-db/src/database/mod.rs` (YamlDatabase coordinator, load/save/lock)
- `crates/yaml-db/src/database/tasks.rs` (Task CRUD, dependency validation, enrichment)
- `crates/yaml-db/src/database/features.rs` (Feature CRUD)
- `crates/yaml-db/src/database/disciplines.rs` (Discipline CRUD, defaults init)
- `crates/yaml-db/src/database/comments.rs` (Comment CRUD)
- `crates/yaml-db/src/database/stats.rs` (get_feature_stats, get_discipline_stats, get_project_progress, get_all_tags)

---

# Entities

## Task

Fields (YAML):

- `id: u32` — globally unique across all features/disciplines, assigned as `max(all_task_ids) + 1`
- `feature: String` — foreign key to features, must exist at create/update time
- `discipline: String` — foreign key to disciplines, must exist at create/update time
- `title: String` — required, trimmed, not empty
- `description: Option<String>`
- `status: TaskStatus` — required, always set to `pending` on create
- `priority: Option<Priority>`
- `tags: Vec<String>` — default empty
- `depends_on: Vec<u32>` — validated: all IDs must exist, no self-reference, no cycles (DFS)
- `blocked_by: Option<String>` — human-readable reason, preserved on update (NOT overwritten)
- `created: Option<String>` — auto-set to UTC ISO date on create, preserved on update
- `updated: Option<String>` — auto-set to UTC ISO date on update
- `completed: Option<String>` — preserved on update (NOT overwritten)
- `acceptance_criteria: Vec<String>` — default empty
- `context_files: Vec<String>` — default empty
- `output_artifacts: Vec<String>` — default empty
- `hints: Option<String>`
- `estimated_turns: Option<u32>`
- `provenance: Option<TaskProvenance>` — preserved on update (NOT overwritten)
- `comments: Vec<TaskComment>` — nested array; preserved on update (NOT overwritten). In SQLite, normalized to `task_comments` table.

**Fields preserved on update (NOT overwritten):** status, blocked_by, created, completed, provenance, comments.

Related enums:

- `TaskStatus`: `pending | in_progress | done | blocked | skipped`
- `Priority`: `low | medium | high | critical`
- `TaskProvenance`: `agent | human | system`

## TaskComment

Fields (YAML):

- `author: CommentAuthor` — required
- `agent_task_id: Option<u32>` — required if author=agent, forbidden if author=human
- `body: String` — required, trimmed, not empty
- `created: Option<String>` — auto-set to UTC ISO timestamp (YYYY-MM-DDTHH:MM:SSZ)

Addressing: Currently by **array index** (0-based). In SQLite, will change to **auto-increment ID** (stable, does not shift on delete). This is an IPC breaking change — see Doc 020.

Enum:

- `CommentAuthor`: `human | agent`

## Feature

Fields (YAML):

- `name: String` — primary key, lowercase-hyphenated, unique. Normalized by `normalize_feature_name()` (lowercase, replace whitespace with hyphens, reject `/:\`)
- `display_name: String` — required, trimmed, not empty
- `acronym: String` — required, 4 chars, uppercase alphanumeric, unique among features
- `description: Option<String>`
- `created: Option<String>` — auto-set on create, preserved on update
- `knowledge_paths: Vec<String>` — reserved for RAG, default empty, preserved on update
- `context_files: Vec<String>` — reserved for RAG, default empty, preserved on update

**Delete constraint:** Cannot delete a feature if any task references it.

## Discipline

Fields (YAML):

- `name: String` — primary key, lowercase-hyphenated, unique
- `display_name: String` — required, trimmed, not empty
- `icon: String` — Lucide icon name (e.g., "Monitor")
- `color: String` — hex color (e.g., "#3b82f6")
- `acronym: String` — required, 4 chars, uppercase alphanumeric, unique among disciplines
- `system_prompt: Option<String>` — preserved on update
- `skills: Vec<String>` — default empty, preserved on update
- `conventions: Option<String>` — preserved on update
- `mcp_servers: Vec<McpServerConfig>` — default empty, preserved on update

**Delete constraint:** Cannot delete a discipline if any task references it.

**10 Default Disciplines (seeded on init):**

| name | acronym | icon | color |
|---|---|---|---|
| frontend | FRNT | Monitor | #3b82f6 |
| backend | BACK | Server | #8b5cf6 |
| wiring | WIRE | Cable | #06b6d4 |
| database | DTBS | Database | #10b981 |
| testing | TEST | TestTube2 | #f59e0b |
| infra | INFR | Cloud | #6366f1 |
| security | SECR | Shield | #ef4444 |
| docs | DOCS | BookOpen | #14b8a6 |
| design | DSGN | Palette | #ec4899 |
| api | APIS | Globe | #84cc16 |

## McpServerConfig

Fields (YAML):

- `name: String`
- `command: String`
- `args: Vec<String>` — default empty
- `env: HashMap<String, String>` — default empty

In SQLite: stored as JSON text within the `disciplines.mcp_servers` column.

## ProjectMetadata

Fields (YAML):

- `title: String` — required
- `description: Option<String>`
- `created: Option<String>` — auto-set on init

## MetadataFile (YAML)

Fields:

- `schema_version: String` — "1.0"
- `project: ProjectMetadata`
- `_counters: { feature: { discipline: max_id } }`

Counters are rebuilt from tasks in `metadata.rebuild_counters()`. They track highest task ID per (feature, discipline) pair but are NOT authoritative — just an optimization for ID assignment. In SQLite, replaced by `SELECT COALESCE(MAX(id), 0) + 1 FROM tasks`.

---

# Derived / Computed Structures (Non-YAML)

## EnrichedTask

Returned over IPC (camelCase JSON) by `get_enriched_tasks()`:

- All Task fields (camelCase serialized)
- `feature_display_name` — from Feature join, fallback: raw feature name
- `feature_acronym` — from Feature join, fallback: empty string
- `discipline_display_name` — from Discipline join, fallback: raw discipline name
- `discipline_acronym` — from Discipline join, fallback: empty string
- `discipline_icon` — from Discipline join, fallback: empty string
- `discipline_color` — from Discipline join, fallback: empty string
- `inferred_status: InferredTaskStatus` — computed (not stored)

## InferredTaskStatus

Derived from raw status + dependency graph. Derivation logic:

```
if status == Done       -> Done
if status == Skipped    -> Skipped
if status == InProgress -> InProgress
if status == Blocked    -> ExternallyBlocked
if status == Pending:
  if ALL depends_on tasks have status == Done -> Ready
  else -> WaitingOnDeps
```

Variants: `ready | waiting_on_deps | externally_blocked | in_progress | done | skipped`

## GroupStats

Returned by `get_feature_stats()` and `get_discipline_stats()`:

- `name: String`
- `display_name: String`
- `total: u32`
- `done: u32`
- `pending: u32`
- `in_progress: u32`
- `blocked: u32`
- `skipped: u32`

## ProjectProgress

Returned by `get_project_progress()`:

- `total_tasks: u32`
- `done_tasks: u32`
- `progress_percent: u32` — integer: `(done / total) * 100`, or 0 if no tasks

---

# Behavioral Notes (Current YAML Implementation)

## Validation Rules

- **Feature names** are normalized by `normalize_feature_name()`: lowercase, replace whitespace with hyphens, reject slashes (`/`), colons (`:`), and backslashes (`\`).
- **Acronym format**: exactly 4 characters, uppercase ASCII alphanumeric only.
- **Acronym uniqueness**: enforced independently for Features and Disciplines (a feature and discipline CAN share an acronym, but two features cannot).
- **Task creation requires**: existing feature + existing discipline; title not empty; feature not empty; discipline not empty.
- **Task update preserves**: status, blocked_by, created, completed, provenance, comments (these fields are NOT overwritten by update).
- **Task delete blocked if**: any other task has this task's ID in its `depends_on` list.
- **Feature/Discipline delete blocked if**: any task references it.
- **Dependency validation**: all `depends_on` IDs must reference existing tasks. Self-dependencies rejected. Circular chains detected via DFS.
- **Comment validation**: body not empty (trimmed); agent author requires agent_task_id; human author forbids agent_task_id.

## Concurrency Model

- **Atomic writes**: write to `.yaml.tmp`, then atomic rename to `.yaml`.
- **File locking**: fs2 crate exclusive lock before any mutation. Blocks concurrent writers.
- **Reload on write**: all 4 files reloaded from disk before every mutation (fresh state under lock).
- **Save-all**: all 4 files saved atomically together after every mutation.

## ID Assignment

- Task IDs are globally unique (not scoped to feature/discipline).
- Next ID = `max(all existing task IDs) + 1`.
- This means IDs CAN be reused after deletion if the deleted ID was the max.

---

# Data Flow: How YAML Reaches Claude

Understanding how database data flows into Claude is critical for the SQLite migration.

## 1. Prompt Builder (hot path)
`prompt_builder.rs` → `read_prd_content()` reads 4 YAML files as **raw text** and concatenates them. Claude receives the YAML directly — it's not parsed or reformatted. This means the SQLite export function must produce YAML-formatted text to preserve prompt behavior.

## 2. Stagnation Hash (hot path)
`loop_engine.rs` → `get_progress_hash()` reads 4 YAML files + `progress.txt` + `learnings.txt`, concatenates all 6, and SHA256 hashes the result. Any byte change in any file resets the stagnation counter.

## 3. MCP Tools (NOT wired into loop yet)
`mcp_generator.rs` generates bash scripts that `cat` YAML files and expose them as MCP tools/resources. However, **the loop engine does NOT pass `--mcp-config` to Claude CLI** — it only uses `-p` (inline prompt). MCP tool generation exists but isn't used during loop iterations. This means MCP migration is lower priority.

## 4. Claude CLI Invocation
`claude_client.rs` invokes: `timeout 900s claude --output-format stream-json --max-turns 50 -p "$prompt"` in the project directory. The prompt is the only data channel — Claude doesn't read YAML files directly during the loop.

---

# IPC Commands (from commands.rs)

22 IPC commands total. These define the API surface that must remain stable.

## Project Lifecycle
- `validate_project_path(path)` — checks dir exists, has `.ralph/`, has `.ralph/db/`
- `initialize_ralph_project(path, project_title)` — creates `.ralph/db/` + 4 YAML files + CLAUDE.RALPH.md template
- `set_locked_project(path)` — validates, canonicalizes, stores in app state (one-time)
- `get_locked_project()` — returns locked path or None
- `scan_for_ralph_projects(root_dir?)` — recursive scan (5 levels, max 100, skip node_modules/.git/etc)
- `get_current_dir()` — returns home directory

## Task CRUD
- `create_task(feature, discipline, title, ...)` — normalizes feature name, returns task ID as string
- `update_task(id, feature, discipline, title, ...)` — normalizes feature name
- `delete_task(id)` — fails if other tasks depend on it
- `get_enriched_tasks()` — returns all tasks with joined display fields + inferred status
- `get_feature_stats()` — returns GroupStats per feature
- `get_discipline_stats()` — returns GroupStats per discipline
- `get_project_progress()` — returns total/done/percent
- `get_all_tags()` — returns deduplicated, sorted tags from all tasks

## Feature CRUD
- `get_features()` — returns FeatureData (full shape)
- `get_features_config()` — returns FeatureConfig (reduced shape, no metadata/paths)
- `create_feature(name, displayName, acronym, description?)`
- `update_feature(name, displayName, acronym, description?)`
- `delete_feature(name)` — fails if tasks reference it

## Discipline CRUD
- `get_disciplines_config()` — returns DisciplineConfig (with MCP servers)
- `create_discipline(name, displayName, acronym, icon, color)`
- `update_discipline(name, displayName, acronym, icon, color)`
- `delete_discipline(name)` — fails if tasks reference it

## Comment CRUD
- `add_task_comment(taskId, author, agentTaskId?, body)`
- `update_task_comment(taskId, commentIndex, body)` — **will change to commentId in SQLite**
- `delete_task_comment(taskId, commentIndex)` — **will change to commentId in SQLite**

## Project Info
- `get_project_info()` — returns title, description, created

---

# Planned Schema Expansions (From Docs 017 + 018)

These define the **target SQLite schema** additions. Not part of the current YAML schema. Implement when RAG work begins.

## Feature (Planned Additions from Doc 018)

Add columns to the `features` table:

- `architecture: TEXT` — architectural overview
- `boundaries: TEXT` — feature boundaries description
- `dependencies: TEXT` — JSON array of feature names this depends on

Add separate `feature_learnings` table (not a column — normalized for query efficiency):

### feature_learnings table

- `id: INTEGER PRIMARY KEY AUTOINCREMENT`
- `feature: TEXT NOT NULL` — FK to features
- `text: TEXT NOT NULL`
- `source: TEXT` — `auto | agent | human | opus_reviewed`
- `iteration: INTEGER`
- `created: TEXT`
- `hit_count: INTEGER DEFAULT 0`
- `verified: INTEGER DEFAULT 0` — boolean (SQLite has no bool type)

`current_state` from Doc 018 is computed, not stored. Derive at query time from latest iteration outcome + task statuses.

## Feature Memory (Planned from Doc 017)

`iterations` table for structured memory entries:

- `id: INTEGER PRIMARY KEY AUTOINCREMENT`
- `iteration_number: INTEGER NOT NULL`
- `task_id: INTEGER NOT NULL` — FK to tasks
- `task_title: TEXT`
- `discipline: TEXT`
- `feature: TEXT NOT NULL` — FK to features
- `timestamp: TEXT`
- `outcome: TEXT` — `success | failure | partial`
- `summary: TEXT`
- `errors: TEXT DEFAULT '[]'` — JSON array
- `decisions: TEXT DEFAULT '[]'` — JSON array
- `files_touched: TEXT DEFAULT '[]'` — JSON array of `{path, action}`
- `tokens_used: INTEGER`

These entries are embedded into Qdrant for vector search. **SQLite is the durable store; Qdrant is an index.** On embedding model change (dimension mismatch), re-read from `iterations` table, re-embed, rebuild Qdrant collections. History is never lost. (See Doc 018, F19.)

## Discipline (Planned Execution Context)

No schema changes needed. These fields already exist in the YAML schema and will carry over to the SQLite `disciplines` table:

- `system_prompt: TEXT`
- `skills: TEXT` — JSON array
- `conventions: TEXT`
- `mcp_servers: TEXT` — JSON array of McpServerConfig objects

Work needed: build CRUD UI for these fields (currently unused in frontend).
