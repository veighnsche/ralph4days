# Feature: Local DB Architecture (SQLite + Vendored Qdrant)

**Created:** 2026-02-07
**Status:** Design Proposal
**Depends on:** Doc 017 (Feature-Scoped RAG), Doc 018 (Feature Entity Redesign)

## Summary

Ralph will use two local data stores:

**Note:** There is no production data to migrate. All data to date is mock. Do not spend time on data migration logic.

1. **SQLite** (embedded in the app) for all structured data: tasks, features, disciplines, metadata, comments, iteration history.
2. **Qdrant** (vendored server binary) for vector search over feature memory and feature snapshots.

Embeddings remain **external** (Ollama or other provider), but Qdrant and SQLite are fully local and require **no user setup**. Qdrant runs as a **sidecar process** managed by Ralph.

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
│  │   SQLite     │     │   Qdrant (sidecar process) │     │
│  │  (embedded)  │     │   vendored binary           │     │
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
- Natural fit for relational data (tasks, comments, counters)
- Easy migrations
- Zero runtime setup (compiled in)

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

## Vendor Strategy

### SQLite
- Use `rusqlite` or `sqlx` (embedded, no external binaries)
- Stored at `~/.ralph/db/ralph.db`

### Qdrant
- Vendor the Qdrant server binary in the app bundle
- Store data in `~/.ralph/qdrant_storage`
- On app startup, check health at `http://127.0.0.1:6333/healthz`
- If not running, spawn the bundled Qdrant binary
- If start fails, mark RAG unavailable and continue

No Docker, no user instructions.

## Qdrant Sidecar Lifecycle

- **Start:** On app startup or first RAG usage
- **Stop:** Optional; can leave running or terminate on app exit
- **Health checks:** Used by `RagHealthCheck`
- **Data persistence:** Local disk under `~/.ralph/qdrant_storage`

## Data Responsibilities

### SQLite (Primary Store)
- Tasks
- Features
- Disciplines
- Metadata counters
- Task comments
- Iteration history
- Execution context (context_files, hints, etc.)

### Qdrant (Vector Store)
- Feature memory (iteration embeddings)
- Feature snapshots (feature-level embeddings)

## Vector Scale Reality Check

Based on current loop design:
- One vector per iteration
- One feature snapshot vector per feature

Typical scale for a large project: **1k–10k vectors**, not millions.

4096-dim embeddings remain feasible in Qdrant. SQLite is not relied upon
for vector search at all.

## API/Module Impact

### New Modules (src-tauri)
- `db/` module for SQLite access
- `qdrant_sidecar.rs` for lifecycle management
- `rag/` module for vector operations (wraps Qdrant client)

### Loop Engine Changes
- Uses SQLite for task/feature retrieval and updates
- RAG availability tied to Qdrant health

### Prompt Builder Changes
- Reads from SQLite instead of YAML

---

# Current Code Touchpoints (Tailored)

These are the concrete files that currently depend on YAML and must be updated:

## Backend (Rust / Tauri)

- `src-tauri/src/commands.rs` uses `YamlDatabase::from_path(...)` in all CRUD and query IPC commands and creates YAML files in `initialize_ralph_project`. Replace with SQLite-backed db handle and SQLite seeding.
- `src-tauri/src/prompt_builder.rs` `read_prd_content()` reads YAML and concatenates text. Replace with SQLite export (text/JSON).
- `src-tauri/src/loop_engine.rs` `get_progress_hash()` hashes YAML + progress/learnings. Replace YAML inputs with SQLite export text or a snapshot hash.
- `src-tauri/src/prd.rs` is a legacy YAML PRD parser. Deprecate or repurpose to read SQLite exports.
- `src-tauri/src/mcp_generator.rs` exposes MCP tools that read YAML from `.ralph/db`. Replace with tools that query SQLite.

## Frontend (React)

No direct YAML access, but IPC return shapes must remain stable. Keep JSON shapes identical for `get_features`, `get_enriched_tasks`, `get_feature_stats`, `get_project_progress`.

## Tests

Replace `crates/yaml-db/tests/*` and YAML snapshot fixtures with SQLite tests and JSON export snapshots.

---

# Packaging Notes (Tailored)

### Vendored Qdrant Binary
- Add Qdrant binary to the app bundle for each target OS/arch.
- Spawn from a known path (e.g., `resources/qdrant/qdrant`).
- Use a lockfile or pidfile in `~/.ralph/qdrant_storage` to avoid double-start.

### Sidecar Startup Flow (Suggested)
1. On app boot, call `QdrantSidecar::ensure_running()`.
2. If `healthz` fails, spawn Qdrant with `--storage-path ~/.ralph/qdrant_storage --http-port 6333 --grpc-port 6334`.
3. Poll `healthz` up to N seconds.
4. On failure, set `RagStatus.available = false` and continue.


## Migration Plan (YAML → SQLite)

1. **Phase 0:** Introduce SQLite alongside YAML (read-only mirror).
2. **Phase 1:** Write through to SQLite, keep YAML for compatibility.
3. **Phase 2:** Switch app reads to SQLite.
4. **Phase 3:** Deprecate YAML writes (optional export for debugging).

## Risks

- **Bundled Qdrant binary size**: increases package size.
- **OS compatibility**: need correct Qdrant binary per target.
- **Sidecar lifecycle**: must be robust on crash/restart.

## Decision Summary

- **Yes** to SQLite as embedded structured DB.
- **Yes** to vendored Qdrant as vector sidecar.
- **No** to Docker requirement.
- **Embeddings external** for now.
