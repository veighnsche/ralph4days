# Deprecate yaml-db in Favor of SQLite

**Created:** 2026-02-07
**Status:** Implementation Plan
**Depends on:** Doc 019 (Local DB Architecture), Doc 018 (Feature Redesign)

## Goal

Replace the YAML-based database (`crates/yaml-db`) with a SQLite-backed store while preserving behavior and API shape. There is no production data to migrate (everything is mock), so the plan focuses on **logic/behavior migration**, not data migration.

## Guiding Principles

- **Behavior parity first.** Match logic, validation, and IPC shapes exactly.
- **No data migration.** Replace storage and mocks; no import paths required.
- **Keep API surface stable** (IPC and frontend unaffected).
- **Remove YAML everywhere** (init, snapshots, tests) once SQLite is in place.

## Phase 0 — Prep and Schema Design

1. **Define SQLite schema** aligned to current YAML structs and behavior.
2. **Decide SQLite access layer** (recommended: `rusqlite` with `bundled`).

### Required Tables (minimum)

- `features`
- `disciplines`
- `tasks`
- `task_comments`
- `metadata`
- `iterations` (feature memory entries)
- `feature_snapshots` (optional, if storing snapshot text)

## Phase 1 — Implement SQLite DB + Seed Defaults

1. **Introduce new crate or module**: `crates/sqlite-db` or `src-tauri/src/db/`.
2. **Create SQLite file at** `~/.ralph/db/ralph.db`.
3. **Seed defaults** (disciplines + metadata) in SQLite on initialization.
4. **No YAML mirror.** YAML files are not created or read.

Deliverable: SQLite exists and contains default disciplines and project metadata.

## Phase 2 — Behavior Parity (Reads + Writes)

1. **Replace all IPC read paths** to use SQLite:
   - `get_features`, `get_disciplines_config`, `get_features_config`
   - `get_enriched_tasks`, `get_feature_stats`, `get_project_progress`
   - `get_project_info`, `get_all_tags`
2. **Replace all IPC write paths** to use SQLite:
   - `create_task`, `update_task`, `delete_task`
   - `create_feature`, `update_feature`, `delete_feature`
   - `create_discipline`, `update_discipline`, `delete_discipline`
   - `add_task_comment`, `update_task_comment`, `delete_task_comment`
3. **Re-implement validations** identical to YAML behavior (see “Behavior Mapping”).

Deliverable: All behavior flows through SQLite with identical logic/validation.

## Phase 3 — Remove YAML Everywhere

1. **Project initialization**: update `initialize_ralph_project` in `src-tauri/src/commands.rs` to create `ralph.db` only (no YAML files).
2. **Prompt builder**: replace YAML read with SQLite export (see “SQLite → Text Export”).
3. **MCP generator**: replace YAML-backed MCP tools with SQLite-backed tools.
4. **Tests**: remove YAML snapshot fixtures; replace with JSON export snapshots + SQL assertions.
5. **Delete yaml-db crate from runtime** (keep only for reference if desired).

Deliverable: YAML is fully removed from codepaths, tests, and init flow.

## Phase 4 — Cleanups

1. Remove YAML-specific helpers and tests (`src-tauri/src/prd.rs`, yaml-db tests).
2. Update `validate_project_path()` to check for `.ralph/db/ralph.db`.
3. Update `get_progress_hash()` in `src-tauri/src/loop_engine.rs` to hash SQLite export text.

Deliverable: All YAML-era utilities are removed or repurposed.

---

# Detailed Implementation Steps

## Step 1 — Define Schema

Map current structs to SQL tables.

### Example: tasks

```
CREATE TABLE tasks (
  id INTEGER PRIMARY KEY,
  feature TEXT NOT NULL,
  discipline TEXT NOT NULL,
  title TEXT NOT NULL,
  description TEXT,
  status TEXT NOT NULL,
  priority TEXT,
  tags TEXT, -- JSON array
  depends_on TEXT, -- JSON array
  blocked_by TEXT,
  created TEXT,
  updated TEXT,
  completed TEXT,
  acceptance_criteria TEXT, -- JSON array
  context_files TEXT, -- JSON array
  output_artifacts TEXT, -- JSON array
  hints TEXT,
  estimated_turns INTEGER,
  provenance TEXT
);

CREATE TABLE task_comments (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  task_id INTEGER NOT NULL,
  author TEXT NOT NULL,
  agent_task_id INTEGER,
  body TEXT NOT NULL,
  created TEXT,
  FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE
);
```

## Step 2 — Build sqlite-db Layer

- Implement CRUD functions mirroring `yaml-db` behavior.
- Keep method names aligned so callers can be switched easily.
- Target all IPC commands in `src-tauri/src/commands.rs`.

## Step 3 — Initialization Logic

On project initialization:
1. Create `.ralph/` and `.ralph/db/`.
2. Create `ralph.db` with schema.
3. Insert default disciplines.
4. Insert project metadata (title/description/created).

## Step 4 — Replace Read Paths

Update IPC handlers to read from SQLite:
- `get_features`, `get_disciplines_config`, `get_features_config`
- `get_enriched_tasks`, `get_feature_stats`, `get_project_progress`
- `get_project_info`, `get_all_tags`

## Step 5 — Replace Write Paths

Update write paths to persist to SQLite:
- `create_task`, `update_task`, `delete_task`
- `create_feature`, `update_feature`, `delete_feature`
- `create_discipline`, `update_discipline`, `delete_discipline`
- `add_task_comment`, `update_task_comment`, `delete_task_comment`

## Step 6 — Remove YAML From Hot Path

Remove YAML parsing from loop/prompt builder and MCP tooling:
- Update `get_progress_hash()` in `src-tauri/src/loop_engine.rs` to hash SQLite export text.
- Update `validate_project_path()` in `src-tauri/src/commands.rs` to require `ralph.db`.
- Update `mcp_generator.rs` to use SQLite-backed MCP tools.

---

# SQLite → Text Export (Required)

Some prompts currently rely on raw YAML dumps. Replace with a stable text export that is generated from SQLite.

Options:
1. **On-demand export**: `ralph_db.export_prd_text()` produces the same block the prompt builder expects.
2. **Cached export file**: `~/.ralph/db/prd.txt` updated on every write and read by the prompt builder and external tools.

This keeps the prompt shape stable while YAML is removed.

---

# Testing Strategy (Required): Hybrid Snapshots + SQL Assertions

SQLite is a binary file, so tests should snapshot **deterministic text exports**, not the database file itself.

**Recommended hybrid approach:**

1. **Snapshot JSON export (big-picture)**: add `export_snapshot_json()` that outputs deterministic JSON with stable ordering (sort by IDs, names, created timestamps), and snapshot this JSON in tests.
2. **SQL assertions (critical invariants)**: use direct queries to validate constraints and edge cases like unique acronyms, dependency validation, and comment persistence.

This retains readability and keeps golden tests stable without relying on YAML.

---

# Concrete File-by-File Changes (Tailored)

## Backend

- `src-tauri/src/commands.rs` replace `YamlDatabase::from_path(...)`, update `initialize_ralph_project` to create `ralph.db` and seed defaults, and update `validate_project_path` to check for `.ralph/db/ralph.db`.
- `src-tauri/src/prompt_builder.rs` replace `read_prd_content()` with SQLite export.
- `src-tauri/src/loop_engine.rs` replace YAML-based hash inputs with SQLite export text.
- `src-tauri/src/prd.rs` deprecate YAML PRD parsing or rewrite to read SQLite export.
- `src-tauri/src/mcp_generator.rs` replace YAML-backed MCP tools with SQLite-backed MCP tools.

## Tests

- `crates/yaml-db/tests/*` replace with `sqlite-db` tests and JSON export snapshots.

---

# Validation Checklist

- Create task / feature / discipline works
- Update task preserves comments
- Comment CRUD works
- Dependency validation still enforced
- Inferred status logic still matches
- Enriched task output unchanged
- Loop can run multiple iterations without mismatch

---

# Rollback Strategy

If SQLite fails in production:

1. Detect failure (health check or read error)
2. Fall back to YAML read path (only during Phase 2/3)
3. Log warning with remediation instructions

After Phase 5, rollback is no longer YAML-based.

---

# Estimated Timeline

- Phase 0–1: 1–2 days
- Phase 2: 1 day
- Phase 3: 1–2 days
- Phase 4–5: 1 day

Total: ~4–6 days depending on test coverage.
