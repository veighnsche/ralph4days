# Deprecate yaml-db in Favor of SQLite

**Created:** 2026-02-07
**Status:** Implementation Plan
**Depends on:** Doc 019 (Local DB Architecture), Doc 018 (Feature Redesign)

## Goal

Replace the YAML-based database (`crates/yaml-db`) with a SQLite-backed store while preserving behavior and API shape. There is no production data to migrate (everything is mock), so the plan focuses on **logic/behavior migration**, not data migration.

## Guiding Principles

- **Behavior parity first.** Match logic, validation, and IPC shapes exactly.
- **No data migration.** Replace storage and mocks; no import paths required.
- **Keep API surface stable** (IPC and frontend unaffected, except comment addressing change).
- **Remove YAML everywhere** (init, snapshots, tests) once SQLite is in place.

---

# Phases

## Phase 1 — SQLite Crate + Schema + CRUD

**Goal:** A working `sqlite-db` crate with identical behavior to `yaml-db`.

1. Create `crates/sqlite-db/` with dependencies: `rusqlite` (bundled), `sea-query` (with sqlite backend), `sea-query-rusqlite`, `rusqlite_migration`, `serde`, `serde_json`, `chrono`, `thiserror`.
2. Define initial migration in `src/migrations/001_initial.sql` (see "Complete SQLite Schema" below).
3. Implement connection open: set PRAGMAs (WAL, synchronous=NORMAL, foreign_keys=ON), run migrations.
4. Implement all CRUD operations mirroring yaml-db method signatures.
5. Seed 10 default disciplines on first init.
6. Port all 9 yaml-db test suites (100+ test cases) to sqlite-db.

**Deliverable:** `crates/sqlite-db` passes all ported tests with identical behavior.

## Phase 2 — Wire Into Tauri

**Goal:** All IPC commands use SQLite.

1. Add `sqlite-db` dependency to `src-tauri/Cargo.toml`.
2. Add `AppState { db: Mutex<Option<Connection>> }` to Tauri managed state.
3. Open SQLite connection in `set_locked_project()` (after path validation).
4. Replace all `YamlDatabase::from_path()` calls in `commands.rs` with `app.state::<AppState>()`.
5. Update `initialize_ralph_project()` to create `ralph.db` + seed defaults (no YAML files).
6. Update `validate_project_path()` to check for `.ralph/db/ralph.db`.
7. Update comment IPC: `commentIndex` -> `commentId` (see "Comment Addressing Change").
8. Update `prompt_builder.rs`: replace `read_prd_content()` with `db.export_prd_yaml()`.
9. Update `loop_engine.rs`: replace `get_progress_hash()` to hash `db.export_prd_yaml()` + progress.txt + learnings.txt.

**Deliverable:** App runs entirely on SQLite. All frontend behavior unchanged.

## Phase 3 — Remove YAML

**Goal:** Clean codebase with no YAML artifacts.

1. Remove `yaml-db` from `[workspace.members]` and `src-tauri/Cargo.toml`.
2. Delete `crates/yaml-db/` directory.
3. Delete `src-tauri/src/prd.rs` (legacy PRD parser, unused in hot path).
4. Remove `serde_yaml` dependency if no longer used.
5. Update `mcp_generator.rs` (low priority — MCP not wired into loop yet; see "MCP Tool Migration").
6. Update mock fixtures (`fixtures/`) to use `ralph.db` instead of YAML files.
7. Run full test suite, verify clean.

**Deliverable:** No YAML code remains in runtime paths.

---

# Complete SQLite Schema

## Initial Migration (001_initial.sql)

```sql
-- Core tables for Ralph project database
-- Matches yaml-db behavior exactly (see Doc 021 for reference)

CREATE TABLE metadata (
  id INTEGER PRIMARY KEY CHECK (id = 1),  -- singleton row
  schema_version TEXT NOT NULL DEFAULT '1.0',
  project_title TEXT NOT NULL,
  project_description TEXT,
  project_created TEXT
) STRICT;

CREATE TABLE features (
  name TEXT PRIMARY KEY,             -- lowercase-hyphenated, unique by PK
  display_name TEXT NOT NULL,
  acronym TEXT NOT NULL UNIQUE,      -- 4 chars, uppercase alphanumeric (validated in app)
  description TEXT,
  created TEXT,
  knowledge_paths TEXT DEFAULT '[]', -- JSON array
  context_files TEXT DEFAULT '[]'    -- JSON array
) STRICT;

CREATE TABLE disciplines (
  name TEXT PRIMARY KEY,             -- lowercase-hyphenated, unique by PK
  display_name TEXT NOT NULL,
  acronym TEXT NOT NULL UNIQUE,      -- 4 chars, uppercase alphanumeric (validated in app)
  icon TEXT NOT NULL,                -- Lucide icon name
  color TEXT NOT NULL,               -- hex color
  system_prompt TEXT,
  skills TEXT DEFAULT '[]',          -- JSON array
  conventions TEXT,
  mcp_servers TEXT DEFAULT '[]'      -- JSON array of {name, command, args, env}
) STRICT;

CREATE TABLE tasks (
  id INTEGER PRIMARY KEY,            -- NOT AUTOINCREMENT; use MAX(id)+1 to match yaml-db
  feature TEXT NOT NULL REFERENCES features(name) ON DELETE RESTRICT,
  discipline TEXT NOT NULL REFERENCES disciplines(name) ON DELETE RESTRICT,
  title TEXT NOT NULL,
  description TEXT,
  status TEXT NOT NULL DEFAULT 'pending',  -- pending|in_progress|done|blocked|skipped
  priority TEXT,                           -- low|medium|high|critical
  tags TEXT DEFAULT '[]',                  -- JSON array
  depends_on TEXT DEFAULT '[]',            -- JSON array of task IDs
  blocked_by TEXT,                         -- human-readable reason string
  created TEXT,
  updated TEXT,
  completed TEXT,
  acceptance_criteria TEXT DEFAULT '[]',   -- JSON array
  context_files TEXT DEFAULT '[]',         -- JSON array
  output_artifacts TEXT DEFAULT '[]',      -- JSON array
  hints TEXT,
  estimated_turns INTEGER,
  provenance TEXT                          -- agent|human|system
) STRICT;

CREATE TABLE task_comments (
  id INTEGER PRIMARY KEY,             -- AUTOINCREMENT implied by INTEGER PRIMARY KEY + no explicit value
  task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  author TEXT NOT NULL,               -- human|agent
  agent_task_id INTEGER,
  body TEXT NOT NULL,
  created TEXT
) STRICT;

-- Indexes for common query patterns
CREATE INDEX idx_tasks_feature ON tasks(feature);
CREATE INDEX idx_tasks_discipline ON tasks(discipline);
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_comments_task ON task_comments(task_id);
```

Note: `STRICT` mode (SQLite 3.37+, 2021) prevents type coercion bugs — a TEXT column rejects integers, an INTEGER column rejects strings.

## Planned Migration (002_rag_fields.sql — implement when RAG work begins)

```sql
-- Feature memory: iteration history for RAG embedding (Doc 017)
-- This is the DURABLE STORE for iteration history. Qdrant is an INDEX over this data.
-- On embedding model change: re-read from this table, re-embed, rebuild Qdrant collections.
-- (See Doc 018, question F19: embedding model change invalidation)
CREATE TABLE iterations (
  id INTEGER PRIMARY KEY,
  iteration_number INTEGER NOT NULL,
  task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  task_title TEXT,
  discipline TEXT,
  feature TEXT NOT NULL REFERENCES features(name) ON DELETE CASCADE,
  timestamp TEXT,
  outcome TEXT,             -- success|failure|partial
  summary TEXT,
  errors TEXT DEFAULT '[]', -- JSON array
  decisions TEXT DEFAULT '[]', -- JSON array
  files_touched TEXT DEFAULT '[]', -- JSON array of {path, action}
  tokens_used INTEGER
) STRICT;

-- Feature learnings (Doc 018 enriched Feature entity)
CREATE TABLE feature_learnings (
  id INTEGER PRIMARY KEY,
  feature TEXT NOT NULL REFERENCES features(name) ON DELETE CASCADE,
  text TEXT NOT NULL,
  source TEXT DEFAULT 'auto',   -- auto|agent|human|opus_reviewed
  iteration INTEGER,
  created TEXT,
  hit_count INTEGER DEFAULT 0,
  verified INTEGER DEFAULT 0    -- boolean (0/1)
) STRICT;

-- Feature entity expansion (Doc 018)
ALTER TABLE features ADD COLUMN architecture TEXT;
ALTER TABLE features ADD COLUMN boundaries TEXT;
ALTER TABLE features ADD COLUMN dependencies TEXT DEFAULT '[]';  -- JSON array of feature names
```

---

# Schema Design Decisions

## Task ID: MAX(id)+1, NOT AUTOINCREMENT
Current yaml-db behavior: `max(all_task_ids) + 1`. This means IDs CAN be reused after deletion (e.g., tasks [1,2,3,4,5] -> delete 5 -> next task gets ID 5). SQLite's `INTEGER PRIMARY KEY` (without AUTOINCREMENT) uses the same algorithm: `MAX(rowid)+1`. Perfect behavioral match. `AUTOINCREMENT` would prevent reuse, changing behavior.

Implementation:
```rust
let next_id: u32 = conn.query_row(
    "SELECT COALESCE(MAX(id), 0) + 1 FROM tasks", [], |row| row.get(0)
)?;
```

## Comments: Table with Auto-ID, NOT Array Index
YAML stores comments as `Vec<TaskComment>` inside each task, addressed by 0-based array index. Deleting index 1 from [a, b, c] shifts c to index 1. SQLite normalizes to `task_comments` table with stable IDs. Deleting ID 2 doesn't change other IDs.

**IPC breaking change required:**
- `update_task_comment(taskId, commentIndex, body)` -> `update_task_comment(taskId, commentId, body)`
- `delete_task_comment(taskId, commentIndex)` -> `delete_task_comment(taskId, commentId)`
- Frontend: add `id: number` to `TaskComment` type
- This is a clean break since all data is mock

## Acronym Format: Application-Level Validation
SQLite UNIQUE constraint handles uniqueness. The 4-char uppercase alphanumeric format check stays in Rust (same `validate_acronym_format()` function). App-level validation produces better error messages than SQL CHECK constraints.

## Cycle Detection: Application-Level (DFS in Rust)
`depends_on` is a JSON array of integers in a TEXT column. Graph traversal can't be done in SQL. Parse JSON in Rust, build adjacency list, run DFS for cycle detection. Same `validate_dependencies_with_cycles()` function.

## Feature Name Normalization: Application-Level
`normalize_feature_name()` (lowercase, hyphenate whitespace, reject `/:\`) runs before any feature write. Same function, same location in `commands.rs`.

## Enriched Tasks: SQL JOIN with Fallback
Replace Rust iteration-and-lookup with SQL LEFT JOIN:
```sql
SELECT t.*,
       COALESCE(f.display_name, t.feature) as feature_display_name,
       COALESCE(f.acronym, '') as feature_acronym,
       COALESCE(d.display_name, t.discipline) as discipline_display_name,
       COALESCE(d.acronym, '') as discipline_acronym,
       COALESCE(d.icon, '') as discipline_icon,
       COALESCE(d.color, '') as discipline_color
FROM tasks t
LEFT JOIN features f ON t.feature = f.name
LEFT JOIN disciplines d ON t.discipline = d.name
ORDER BY t.id
```
`COALESCE` handles missing feature/discipline (fallback: raw name, empty strings). `InferredTaskStatus` is still computed in Rust from query results + dependency graph — it requires cross-row logic.

---

# Behavior Mapping (YAML -> SQLite)

These behaviors MUST be preserved exactly:

| Behavior | YAML Implementation | SQLite Implementation |
|---|---|---|
| Atomic writes | temp file + rename; all 4 files saved together | Single transaction (BEGIN/COMMIT/ROLLBACK) |
| Concurrent access | fs2 exclusive file lock | WAL mode + `Mutex<Connection>` in Tauri state |
| Reload before write | Re-read all 4 files under lock | Not needed — SQLite provides read consistency |
| Feature name uniqueness | Vec scan before insert | PRIMARY KEY constraint (error on INSERT conflict) |
| Discipline name uniqueness | Vec scan before insert | PRIMARY KEY constraint |
| Acronym uniqueness (features) | Vec scan across features | UNIQUE constraint on `features.acronym` |
| Acronym uniqueness (disciplines) | Vec scan across disciplines | UNIQUE constraint on `disciplines.acronym` |
| Delete feature with tasks | Error if any task has `feature=X` | `FOREIGN KEY ON DELETE RESTRICT` (automatic) |
| Delete discipline with tasks | Error if any task has `discipline=X` | `FOREIGN KEY ON DELETE RESTRICT` (automatic) |
| Delete task with dependents | Error if any task's `depends_on` contains this ID | App-level: query all tasks, parse JSON `depends_on` arrays |
| Cycle detection | DFS traversal in Rust | Same DFS in Rust; parse `depends_on` JSON |
| Task ID assignment | `max(all_ids) + 1`, rebuild counters | `SELECT COALESCE(MAX(id), 0) + 1 FROM tasks` |
| Counter rebuild after write | Scan all tasks, update metadata counters | Not needed — `MAX(id)` query replaces counters |
| Inferred status | Computed on each `get_enriched_tasks()` call | Same — computed in Rust after SQL query |
| Comment validation | Body not empty; agent requires agent_task_id; human forbids it | Same app-level validation before INSERT |
| Timestamp auto-set | UTC ISO format, set in Rust before YAML write | Same — set in Rust before INSERT/UPDATE |
| Status preservation on update | `update_task` does NOT overwrite status, blocked_by, created, completed, provenance, comments | `UPDATE tasks SET title=?, description=?, ... WHERE id=?` — only update mutable columns |
| Feature update preserves | created, knowledge_paths, context_files | `UPDATE features SET display_name=?, acronym=?, description=? WHERE name=?` |
| Discipline update preserves | system_prompt, skills, conventions, mcp_servers | `UPDATE disciplines SET display_name=?, acronym=?, icon=?, color=? WHERE name=?` |

---

# Stagnation Hash (get_progress_hash)

Current behavior hashes 6 files: tasks.yaml + features.yaml + disciplines.yaml + metadata.yaml + progress.txt + learnings.txt.

SQLite replacement:
```rust
fn get_progress_hash(conn: &Connection, project_path: &Path) -> String {
    // 1. Deterministic text export from SQLite
    let db_export = conn.export_prd_yaml();  // sorted, deterministic

    // 2. Read plain text files (NOT in SQLite — remain as files)
    let progress = fs::read_to_string(project_path.join(".ralph/progress.txt")).unwrap_or_default();
    let learnings = fs::read_to_string(project_path.join(".ralph/learnings.txt")).unwrap_or_default();

    // 3. Concatenate and hash
    let combined = format!("{}{}{}", db_export, progress, learnings);
    sha256_hex(&combined)
}
```

The `export_prd_yaml()` function must produce **byte-identical output** for the same database state. Sort all rows by primary key, use consistent formatting for JSON array columns, emit fields in a fixed order.

---

# Prompt Builder: export_prd_yaml()

The prompt builder currently concatenates 4 raw YAML files for Claude context. Replace with a single function that generates YAML-formatted text from SQLite:

```rust
impl SqliteDb {
    /// Export database contents as YAML-formatted text for prompt builder.
    /// Output is deterministic: same DB state always produces identical text.
    pub fn export_prd_yaml(&self) -> Result<String, Error> {
        let mut output = String::new();

        // Section 1: metadata.yaml equivalent
        let meta = self.get_project_metadata()?;
        output += &format!("schema_version: \"1.0\"\nproject:\n  title: \"{}\"\n", meta.title);
        if let Some(desc) = &meta.description { output += &format!("  description: \"{}\"\n", desc); }
        if let Some(created) = &meta.created { output += &format!("  created: \"{}\"\n", created); }
        output += "\n";

        // Section 2: features.yaml equivalent (sorted by name)
        // Section 3: disciplines.yaml equivalent (sorted by name)
        // Section 4: tasks.yaml equivalent (sorted by id, with nested comments)

        Ok(output)
    }
}
```

Format as YAML (not JSON) because Claude currently reads raw YAML — changing format would alter prompt behavior.

---

# MCP Tool Migration

**Current state:** MCP generates bash scripts that `cat tasks.yaml`, etc. However, **MCP is NOT wired into the loop yet** — Claude CLI is invoked with `-p` (inline prompt), not `--mcp-config`. MCP tool generation exists but isn't used during loop iterations.

**Priority:** Low. Address when MCP is wired into the loop (separate feature work).

**When needed, two options:**

1. **Export YAML on write** — After every SQLite write, regenerate `.ralph/db/tasks.yaml` etc. from SQLite. MCP bash scripts continue to `cat` these files unchanged. Simple but creates file sync obligation.

2. **sqlite3 CLI in bash tools** — `sqlite3 .ralph/db/ralph.db "SELECT ..."`. Available on macOS and Linux by default. **NOT available on Windows** without separate install. Only viable if Windows isn't a target.

Option 1 is safer for cross-platform. Option 2 is cleaner but platform-limited.

---

# Testing Strategy

## Approach: Port yaml-db tests to sqlite-db

The 9 existing yaml-db test suites (100+ cases) define the behavior contract. Port each test to sqlite-db, verifying identical outcomes:

| yaml-db test suite | sqlite-db equivalent |
|---|---|
| `crud_operations.rs` | Same CRUD workflows, same assertions |
| `duplicate_validation_tests.rs` | Uniqueness enforced by SQL constraints + app validation |
| `acronym_tests.rs` | Same format validation (app-level) |
| `bug_fixes.rs` | Same edge cases (self-deps, circular deps) |
| `transformation_tests.rs` | Same inferred_status derivation |
| `golden_tests.rs` | Snapshot `export_snapshot_json()` instead of YAML files |
| `edge_cases.rs` | Same boundary conditions |
| `migrate_snapshots.rs` | Not needed (no YAML migration path) |
| `generate_snapshots.rs` | Replace with JSON export snapshot utilities |

## Snapshot Strategy
SQLite is binary — can't snapshot the `.db` file. Instead:
1. `export_snapshot_json()` — deterministic JSON with stable ordering (sort by PK)
2. Snapshot this JSON in tests using `insta` crate
3. Direct SQL assertions for critical invariants (FK violations, unique constraints, etc.)

---

# Concrete File-by-File Changes

## New Files

| File | Purpose |
|---|---|
| `crates/sqlite-db/Cargo.toml` | New crate: rusqlite, rusqlite_migration, serde, serde_json, chrono, thiserror |
| `crates/sqlite-db/src/lib.rs` | Public API: SqliteDb struct, open, CRUD methods |
| `crates/sqlite-db/src/migrations/001_initial.sql` | Initial schema DDL |
| `crates/sqlite-db/src/tasks.rs` | Task CRUD + dependency validation + enrichment |
| `crates/sqlite-db/src/features.rs` | Feature CRUD |
| `crates/sqlite-db/src/disciplines.rs` | Discipline CRUD + defaults |
| `crates/sqlite-db/src/comments.rs` | Comment CRUD |
| `crates/sqlite-db/src/stats.rs` | Stats aggregation queries |
| `crates/sqlite-db/src/export.rs` | export_prd_yaml(), export_snapshot_json() |
| `crates/sqlite-db/tests/*.rs` | Ported test suites |

## Modified Files

| File | Changes |
|---|---|
| `Cargo.toml` (workspace) | Add `crates/sqlite-db` to members |
| `src-tauri/Cargo.toml` | Replace `yaml-db` with `sqlite-db` |
| `src-tauri/src/commands.rs` | Replace `YamlDatabase::from_path()` with `app.state::<AppState>()`. Update `initialize_ralph_project` to create ralph.db. Update `validate_project_path` to check for ralph.db. Change comment IPC from index to ID. |
| `src-tauri/src/prompt_builder.rs` | Replace `read_prd_content()` (reads 4 YAML files) with `db.export_prd_yaml()` |
| `src-tauri/src/loop_engine.rs` | Replace `get_progress_hash()`: hash `db.export_prd_yaml()` + progress.txt + learnings.txt |
| `src-tauri/src/main.rs` | Add `AppState` to managed state |
| Frontend `TaskComment` type | Add `id: number` field |
| Frontend comment CRUD calls | `commentIndex` -> `commentId` |

## Deleted Files

| File | Reason |
|---|---|
| `crates/yaml-db/` | Replaced entirely by sqlite-db |
| `src-tauri/src/prd.rs` | Legacy PRD parser, unused in hot path |

---

# Validation Checklist

## CRUD Operations
- [ ] Create task with valid feature + discipline succeeds
- [ ] Create task with nonexistent feature fails
- [ ] Create task with nonexistent discipline fails
- [ ] Create task with empty title fails
- [ ] Update task preserves: status, blocked_by, created, completed, provenance, comments
- [ ] Delete task with dependents fails
- [ ] Delete task without dependents succeeds
- [ ] Create feature with duplicate name fails (PRIMARY KEY)
- [ ] Create feature with duplicate acronym fails (UNIQUE)
- [ ] Update feature preserves: created, knowledge_paths, context_files
- [ ] Delete feature with existing tasks fails (RESTRICT)
- [ ] Create discipline with duplicate name fails
- [ ] Create discipline with duplicate acronym fails
- [ ] Update discipline preserves: system_prompt, skills, conventions, mcp_servers
- [ ] Delete discipline with existing tasks fails (RESTRICT)

## Comments
- [ ] Add comment with empty body fails
- [ ] Add agent comment without agent_task_id fails
- [ ] Add human comment with agent_task_id fails
- [ ] Update comment by ID works
- [ ] Delete comment by ID works
- [ ] Delete task cascades to comments

## Validation
- [ ] Acronym format: 4 chars, uppercase alphanumeric
- [ ] Feature name normalization: lowercase, hyphenated, rejects /:\
- [ ] Dependency validation: all depends_on IDs exist
- [ ] Self-dependency rejected
- [ ] Circular dependency chains rejected

## Computed Data
- [ ] InferredTaskStatus: Ready, WaitingOnDeps, ExternallyBlocked, InProgress, Done, Skipped
- [ ] EnrichedTask: correct JOIN with fallback values for missing feature/discipline
- [ ] Feature stats: correct counts per feature
- [ ] Discipline stats: correct counts per discipline
- [ ] Project progress: correct total/done/percent
- [ ] get_all_tags: deduplicated, sorted

## Integration
- [ ] Stagnation hash detects changes correctly
- [ ] Prompt builder produces valid YAML-formatted context
- [ ] WAL mode enabled (concurrent reads during writes)
- [ ] PRAGMA foreign_keys = ON (enforced)
- [ ] Connection reused across IPC calls (Mutex<Connection>)
- [ ] Graceful shutdown: PRAGMA optimize runs on close
- [ ] Loop runs multiple iterations without errors

---

# Rollback Strategy

Not needed. All data is mock. If SQLite implementation has issues, fix them directly. No YAML fallback path — that would violate the single execution path policy and double the code surface for zero benefit.

---

# Cargo Dependencies (New)

```toml
# crates/sqlite-db/Cargo.toml
[package]
name = "sqlite-db"
version = "0.1.0"
edition = "2021"

[dependencies]
rusqlite = { version = "0.31", features = ["bundled"] }
rusqlite_migration = "1"
sea-query = { version = "0.32", features = ["backend-sqlite"] }
sea-query-rusqlite = "0.7"
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
tempfile = "3"
insta = { version = "1.40", features = ["json"] }
```

---

# Estimated Timeline

- Phase 1 (sqlite-db crate + all CRUD + tests): 2-3 days
- Phase 2 (wire into Tauri, update all IPC): 1-2 days
- Phase 3 (remove yaml-db, cleanup): 1 day

Total: ~4-6 days.
