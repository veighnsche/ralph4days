# Feature: Local DB Architecture (SQLite + Vendored Qdrant)

**Created:** 2026-02-07
**Status:** Design Proposal
**Depends on:** Doc 017 (Feature-Scoped RAG), Doc 018 (Feature Entity Redesign)

## Summary

Ralph will use two local data stores:

**Note:** There is no production data to migrate. All data to date is mock. Do not spend time on data migration logic.

1. **SQLite** (embedded via `rusqlite` with `bundled` feature) for all structured data: tasks, features, disciplines, metadata, comments, iteration history.
2. **Qdrant** (vendored via Tauri sidecar) for vector search over feature memory and feature snapshots.

Embeddings remain **external** (Ollama or other provider), but Qdrant and SQLite are fully local and require **no user setup**. Qdrant runs as a **Tauri sidecar process**.

## Goals

- Eliminate YAML as the primary database for runtime state.
- Avoid requiring Docker or manual user setup.
- Keep vector search performant at 4096 dimensions.
- Preserve the current architecture: task-centric loop + feature-scoped memory.

## Non-Goals

- Embedding model is not bundled yet (Ollama remains external for now).
- No cloud services.
- No multi-node, multi-user database.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                     Ralph Desktop App                   │
│                                                         │
│  ┌──────────────┐     ┌────────────────────────────┐     │
│  │   SQLite     │     │   Qdrant (Tauri sidecar)   │     │
│  │  (rusqlite)  │     │   vendored binary           │     │
│  └──────┬───────┘     └──────────────┬─────────────┘     │
│         │                            │                   │
│         │                            │                   │
│  Structured data                 Vector search           │
│  (tasks, features,               (feature memory,        │
│  comments, metadata)             feature snapshots)      │
└─────────┴──────────────────────────┴─────────────────────┘

Embedding generation is external (e.g., Ollama). Ralph only
stores embeddings and queries vectors from Qdrant.
```

## Why Two Databases

### SQLite for Structured Data
- Strong consistency and ACID transactions
- Natural fit for relational data (tasks, comments, foreign keys)
- Schema migration via `rusqlite_migration` + `PRAGMA user_version`
- Zero runtime setup (compiled from source via `bundled` feature)

### Qdrant for Vectors
- Handles 4096-dim embeddings reliably
- Efficient ANN search with filters
- Mature, stable, battle-tested

### Why Not a Single DB?
A single embedded DB that does relational + ANN at high dimension is possible but:
- More engineering effort
- Slower ANN at scale
- Less reliable for 4096-dim vectors

Two-store architecture minimizes risk and keeps performance predictable.

## SQLite Strategy

### Crate: `rusqlite` + `sea-query` + `rusqlite_migration`
**Not** `sqlx`, Diesel, or SeaORM. Rationale:
- `rusqlite` is synchronous — simpler for a single-user desktop app, no async overhead
- `bundled` feature compiles SQLite from C source — no system dependency on `libsqlite3`
- `sea-query` provides a typed query builder (no raw SQL strings) without ORM weight
- `sea-query-rusqlite` bridges sea-query output to rusqlite parameters
- `rusqlite_migration` handles schema versioning via `PRAGMA user_version`
- Full ORMs (Diesel, SeaORM) are overkill for 5 tables with existing Rust structs

### Database Location
Stored **per-project** at `<project>/.ralph/db/ralph.db` — same directory as current YAML files. Each project has its own database.

### Connection Management (Tauri Pattern)
```rust
// In Tauri managed state
pub struct AppState {
    pub db: std::sync::Mutex<Option<rusqlite::Connection>>,
}
```
- Open connection once in `set_locked_project()` (after project is validated and locked)
- Reuse across all IPC commands via `app.state::<AppState>()`
- Close on app exit (Rust Drop handles this)
- NOT a connection pool — single user, single writer

### PRAGMAs (Set on Every Connection Open)
```sql
PRAGMA journal_mode = WAL;       -- concurrent reads during writes
PRAGMA synchronous = NORMAL;     -- safe for app crashes, faster than FULL
PRAGMA foreign_keys = ON;        -- enforce referential integrity
```
On connection close (graceful shutdown):
```sql
PRAGMA analysis_limit = 400;
PRAGMA optimize;                 -- update query planner statistics
```
Sources: [SQLite Pragma Cheatsheet](https://cj.rs/blog/sqlite-pragma-cheatsheet-for-performance-and-consistency/), [SQLite WAL docs](https://sqlite.org/wal.html)

### Schema Versioning
Use `rusqlite_migration` crate:
- Migrations defined as SQL strings in Rust code
- Tracks version via `PRAGMA user_version` (lightweight integer, no migration table)
- Run `migrations.to_latest(&mut conn)` on every connection open
- Future schema changes are just new migration entries

```rust
use rusqlite_migration::{Migrations, M};

let migrations = Migrations::new(vec![
    M::up(include_str!("migrations/001_initial.sql")),
    // Future: M::up(include_str!("migrations/002_rag_fields.sql")),
]);
migrations.to_latest(&mut conn)?;
```

## Qdrant Strategy

### Tauri Sidecar (NOT manual process spawn)
Tauri 2 has native sidecar support. Use it instead of raw `Command::new()`:

**tauri.conf.json:**
```json
{
  "bundle": {
    "externalBin": ["binaries/qdrant"]
  }
}
```

**Binary naming per platform:**
- `src-tauri/binaries/qdrant-x86_64-unknown-linux-gnu`
- `src-tauri/binaries/qdrant-x86_64-apple-darwin`
- `src-tauri/binaries/qdrant-aarch64-apple-darwin`
- `src-tauri/binaries/qdrant-x86_64-pc-windows-msvc.exe`

**Permissions** (`src-tauri/capabilities/default.json`):
```json
{
  "identifier": "shell:allow-spawn",
  "allow": [{
    "name": "binaries/qdrant",
    "sidecar": true,
    "args": [
      "--storage-path", { "validator": ".*" },
      "--http-port", "6333",
      "--grpc-port", "6334"
    ]
  }]
}
```

**Spawn from Rust:**
```rust
let (rx, child) = app.shell()
    .sidecar("binaries/qdrant")
    .args(["--storage-path", &qdrant_storage_path, "--http-port", "6333", "--grpc-port", "6334"])
    .spawn()?;
```

### Storage Location
`~/.ralph/qdrant_storage` — global (not per-project), since Qdrant runs as a single shared process.

### Collection Naming (Multi-Project Isolation)
Qdrant storage is global but collection names must be **project-scoped**. Two projects can have a feature named "authentication" — their vectors must not collide. Collection naming format:

```
proj-<sha256(project_path)[:8]>-feature-<sha256(feature_name)[:8]>
```

This guarantees isolation even with identical feature names across projects. See Doc 018, question F18 for full analysis.

### Lifecycle
- **Start:** On app startup, check `http://127.0.0.1:6333/healthz`. If not running, spawn sidecar.
- **Stop:** Kill child process on app exit. Tauri's sidecar API provides the process handle.
- **Health check failure:** Set `RagStatus.available = false`, continue without RAG.
- **Pidfile:** Use `~/.ralph/qdrant_storage/qdrant.pid` to detect orphaned processes.

### Port Conflict Handling
Ports 6333/6334 are hardcoded. If health check returns OK but the process isn't ours (another Qdrant instance):
- Check if PID matches our spawned child
- If mismatch, log warning: "Qdrant already running (external instance). Using existing."
- If port is occupied by non-Qdrant process, log error and mark RAG unavailable.

## Data Responsibilities

### SQLite (Primary Store — per project)
- Tasks (with context_files, hints, output_artifacts)
- Features (with knowledge_paths, context_files, planned RAG fields from Doc 018)
- Disciplines (with system_prompt, skills, conventions, mcp_servers)
- Project metadata (title, description, schema_version)
- Task comments (normalized to separate table, addressed by auto-ID not array index)
- Iteration history (from Doc 017: iteration_number, outcome, summary, errors, decisions)

### Qdrant (Vector Store — global)
- Feature memory (iteration embeddings)
- Feature snapshots (feature-level embeddings)

## Vector Scale Reality Check

Based on current loop design:
- One vector per iteration
- One feature snapshot vector per feature

Typical scale for a large project: **1k-10k vectors**, not millions.

4096-dim embeddings remain feasible in Qdrant. SQLite is not relied upon for vector search at all.

## API/Module Impact

### New Modules (src-tauri)
- `db/` module — SQLite connection, CRUD, migrations, text export
- `qdrant_sidecar.rs` — Tauri sidecar lifecycle (spawn, health, stop)
- `rag/` module — vector operations (wraps `qdrant-client` crate)

### Changed Modules
- `commands.rs` — Replace `YamlDatabase::from_path()` with `app.state::<AppState>()` DB access
- `prompt_builder.rs` — Replace `read_prd_content()` YAML reads with `db.export_prd_yaml()` function
- `loop_engine.rs` — Replace `get_progress_hash()` to hash SQLite export + progress.txt + learnings.txt
- `mcp_generator.rs` — Replace YAML file reads (MCP not wired into loop yet; lower priority)

### Frontend (React)
No direct database access. IPC return shapes (JSON) remain identical. One breaking change: `TaskComment` adds `id` field, comment CRUD uses `commentId` instead of `commentIndex`.

---

# Current Code Touchpoints

These are the concrete files that currently depend on YAML and must be updated:

## Backend (Rust / Tauri)

| File | Current YAML Usage | SQLite Replacement |
|---|---|---|
| `commands.rs` | `YamlDatabase::from_path()` in all CRUD commands; creates YAML files in `initialize_ralph_project` | `app.state::<AppState>().db` connection; create `ralph.db` + seed defaults |
| `prompt_builder.rs` | `read_prd_content()` reads 4 YAML files, concatenates as text | `db.export_prd_yaml()` — generate YAML-formatted text from SQLite |
| `loop_engine.rs` | `get_progress_hash()` reads 4 YAML + 2 text files, hashes | Hash `db.export_prd_yaml()` + progress.txt + learnings.txt |
| `prd.rs` | Legacy PRD parser (unused in hot path) | Delete entirely |
| `mcp_generator.rs` | Bash scripts that `cat` YAML files | Lower priority — MCP not wired into loop yet. When needed: generate tools that call `sqlite3` or read export files |

## Frontend (React)
No direct YAML access. IPC return shapes must remain stable (camelCase JSON).

## Tests
Replace `crates/yaml-db/tests/*` and YAML snapshot fixtures with SQLite tests and JSON export snapshots.

---

# Migration Plan (YAML -> SQLite)

Since all data is mock, no phased migration or dual-write needed. Cut over directly:

1. **Phase 1:** Implement SQLite DB module with schema, seed defaults, implement all CRUD. Port all yaml-db tests.
2. **Phase 2:** Replace all IPC handlers in commands.rs. Update prompt builder and stagnation hash.
3. **Phase 3:** Remove yaml-db crate from workspace. Delete prd.rs. Update project init and validation.

See Doc 020 for detailed implementation steps.

## Risks

- **Bundled Qdrant binary size**: ~50-100MB per platform, increases package size.
- **OS compatibility**: Need correct Qdrant binary per target triple.
- **Sidecar lifecycle**: Must kill on app exit to avoid orphan processes.
- **Qdrant port conflict**: Ports 6333/6334 hardcoded. Health-check before spawn; clear error if port occupied by non-Qdrant process.

## Decision Summary

| Decision | Choice | Rationale |
|---|---|---|
| SQLite crate | `rusqlite` with `bundled` | Synchronous, direct, compiles SQLite from source |
| Query builder | `sea-query` + `sea-query-rusqlite` | Typed SQL builders, no raw strings, no ORM overhead |
| Schema versioning | `rusqlite_migration` | Lightweight, uses PRAGMA user_version |
| Connection pattern | `Mutex<Connection>` in Tauri state | Single user, single writer |
| Journal mode | WAL | Concurrent reads during writes |
| Qdrant deployment | Tauri sidecar (`externalBin`) | Native Tauri support, proper lifecycle |
| Docker requirement | No | Zero user setup |
| Embeddings | External (Ollama) | Not bundled yet |
| Prompt format | YAML-formatted text from SQLite | Claude reads YAML; minimal prompt disruption |
