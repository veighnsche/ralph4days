# Current YAML DB Schema (As Implemented)

**Created:** 2026-02-07
**Status:** Reference

This document captures the **current YAML database schema** as implemented in code. This is the source of truth for behavior parity when porting to SQLite.

**Note:** There is no production data to migrate. All data to date is mock. Do not spend time on data migration logic.

## Source Files

- `crates/yaml-db/src/lib.rs` (Task, TaskComment, enums, EnrichedTask)
- `crates/yaml-db/src/features.rs` (Feature)
- `crates/yaml-db/src/disciplines.rs` (Discipline, MCP config)
- `crates/yaml-db/src/metadata.rs` (Project metadata, counters)

---

# Entities

## Task

Fields (YAML):

- `id: u32`
- `feature: String`
- `discipline: String`
- `title: String`
- `description: Option<String>`
- `status: TaskStatus`
- `priority: Option<Priority>`
- `tags: Vec<String>`
- `depends_on: Vec<u32>`
- `blocked_by: Option<String>`
- `created: Option<String>`
- `updated: Option<String>`
- `completed: Option<String>`
- `acceptance_criteria: Vec<String>`
- `context_files: Vec<String>`
- `output_artifacts: Vec<String>`
- `hints: Option<String>`
- `estimated_turns: Option<u32>`
- `provenance: Option<TaskProvenance>`
- `comments: Vec<TaskComment>`

Related enums:

- `TaskStatus`: `pending | in_progress | done | blocked | skipped`
- `Priority`: `low | medium | high | critical`
- `TaskProvenance`: `agent | human | system`

## TaskComment

Fields (YAML):

- `author: CommentAuthor`
- `agent_task_id: Option<u32>`
- `body: String`
- `created: Option<String>`

Enum:

- `CommentAuthor`: `human | agent`

## Feature

Fields (YAML):

- `name: String`
- `display_name: String`
- `acronym: String`
- `description: Option<String>`
- `created: Option<String>`
- `knowledge_paths: Vec<String>`
- `context_files: Vec<String>`

## Discipline

Fields (YAML):

- `name: String`
- `display_name: String`
- `icon: String`
- `color: String`
- `acronym: String`
- `system_prompt: Option<String>`
- `skills: Vec<String>`
- `conventions: Option<String>`
- `mcp_servers: Vec<McpServerConfig>`

## McpServerConfig

Fields (YAML):

- `name: String`
- `command: String`
- `args: Vec<String>`
- `env: HashMap<String, String>`

## ProjectMetadata

Fields (YAML):

- `title: String`
- `description: Option<String>`
- `created: Option<String>`

## MetadataFile (YAML)

Fields:

- `schema_version: String`
- `project: ProjectMetadata`
- `_counters: { feature: { discipline: max_id } }`

Counters are rebuilt from tasks in `metadata.rebuild_counters()`.

---

# Derived / Computed Structures (Non-YAML)

## EnrichedTask

Returned over IPC (camelCase JSON) by `get_enriched_tasks()`:

- Task fields plus pre-joined display data from Feature and Discipline
- `feature_display_name`
- `feature_acronym`
- `discipline_display_name`
- `discipline_acronym`
- `discipline_icon`
- `discipline_color`
- `inferred_status: InferredTaskStatus`

## InferredTaskStatus

Derived from raw status + dependency graph:

- `ready`
- `waiting_on_deps`
- `externally_blocked`
- `in_progress`
- `done`
- `skipped`

---

# Behavioral Notes (Current YAML Implementation)

- **Feature names** are normalized to lowercase hyphenated strings.
- **Acronym uniqueness** enforced for both Features and Disciplines.
- **Comment validation**:
  - Body cannot be empty
  - Agent comment requires `agent_task_id`
  - Human comment must NOT include `agent_task_id`
- **Atomic writes** (temp + rename) and **file locking** (fs2) used in YAML DB.
- **Counters** are rebuilt from tasks and are not authoritative.

---

# Planned Schema Expansions (From Docs 017 + 018)

These were originally described as YAML changes, but they now define the **target SQLite schema** additions. They are included here for completeness.

## Feature (Planned Additions from Doc 018)

Add the following fields to the Feature entity:

- `architecture: Option<String>`
- `boundaries: Option<String>`
- `learnings: Vec<FeatureLearning>` (support simple strings or rich objects)
- `dependencies: Vec<String>`
- `current_state: Option<String>` (computed, not stored in YAML; may be computed in SQLite too)

### FeatureLearning (Planned)

Either a simple string or a rich object with provenance:

- `text: String`
- `source: LearningSource` (`auto | agent | human | opus_reviewed`)
- `iteration: Option<u32>`
- `created: Option<String>`
- `hit_count: u32`
- `verified: bool`

## Feature Memory (Planned from Doc 017)

After each iteration, store a structured memory entry:

- `iteration_number: u32`
- `task_id: u32`
- `task_title: String`
- `discipline: String`
- `feature: String`
- `timestamp: String`
- `outcome: String` (`success | failure | partial`)
- `summary: String`
- `errors: Vec<String>`
- `decisions: Vec<String>`
- `files_touched: Vec<{path, action}>`
- `tokens_used: Option<u32>`

These entries are used for embedding + search and should map to an `iterations` table in SQLite and to vectors in Qdrant.

## Discipline (Planned Execution Context)

Ensure full CRUD on these fields (already in YAML schema, but currently underused):

- `system_prompt: Option<String>`
- `skills: Vec<String>`
- `conventions: Option<String>`
- `mcp_servers: Vec<McpServerConfig>`
