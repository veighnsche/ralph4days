# Feature-Scoped RAG System

**Created:** 2026-02-07
**Status:** Implementation Plan
**Depends on:** Doc 015 (Task Model as Prompt Assembly Nexus)
**Reference:** [Kilo Code codebase indexing](https://github.com/Kilo-Org/kilocode) — Qdrant + Ollama patterns adapted for feature-scoped use

## Context

Doc 015 established the "Task Model as Prompt Assembly Nexus" — tasks reference Disciplines (HOW) and Features (WHAT), and Ralph assembles surgical prompts from all three. It added `knowledge_paths` and `context_files` to the Feature schema and described three phases:

**Note:** There is no production data to migrate. All data to date is mock. Do not spend time on data migration logic.

- Phase 1 (Collect): Schema — **done**, fields exist but are unwired
- Phase 2 (Assemble): Prompt builder — **not started**
- Phase 3 (Refine): MCP resources — **not started**

This document extends Doc 015 with a **feature memory** layer: after each iteration, Ralph extracts what happened, embeds it via Ollama, and stores vectors in Qdrant per-feature. Future iterations can *semantically search* this memory for relevant context rather than getting everything dumped into the prompt.

**RAG is optional.** The system degrades gracefully when Qdrant or Ollama are unavailable — Doc 015's static file injection still works. RAG enriches the experience but is never required.

## Motivation: Why Feature-Scoped RAG

The [Cline blog post](https://cline.bot/blog/why-cline-doesnt-index-your-codebase-and-why-thats-a-good-thing) and similar articles argue against RAG for code agents. Their arguments are valid for **whole-codebase code indexing**. Feature-scoped RAG sidesteps every one:

| Anti-RAG Argument | Whole-Codebase RAG | Feature-Scoped RAG |
|---|---|---|
| Code chunks lose meaning when split | Chunking arbitrary code destroys logic flow | We're indexing **feature artifacts** — tasks, decisions, learnings, file maps — not code chunks |
| Index goes stale instantly | Codebase changes every save/commit | Features have **finite lifetimes**; memory is rebuilt/updated per iteration |
| Embeddings miss code semantics | `sort_asc` and `sort_desc` embed identically | Our data is natural language (task descriptions, error messages, decision rationale) — embeddings work well here |
| Whole-codebase is a security risk | Entire IP in a vector DB | Scoped to one feature's context, local Qdrant instance |
| Grep + reasoning beats embeddings for code | True for finding code definitions/usage | Grep can't answer "what did we try last time and why did it fail?" — that requires semantic memory |

**The key insight:** this is not code RAG. It is **feature memory RAG** — structured natural-language artifacts accumulated over iterations.

## What Problem This Solves

Currently, each Haiku iteration is **amnesiac**. Claude gets a wall of raw YAML, picks a task, and works blind. If the previous iteration:

- Tried approach X and it failed → Haiku might try X again
- Touched files A, B, C → Haiku doesn't know these are relevant
- Made architectural decision Y → Haiku might contradict it
- Hit error Z in dependency → Haiku will rediscover it

Feature memory gives Haiku **cross-iteration continuity** within a feature, without bloating the prompt with full history.

## Required Dependencies (Optional — RAG disabled when absent)

### Qdrant (Vector Database)

**Vendored sidecar process (no Docker, no user setup).**

- Qdrant server binary is bundled with the app package.
- Ralph spawns Qdrant on startup if it is not already running.
- REST API at `http://localhost:6333`
- gRPC at `localhost:6334` (used by Rust client for performance)
- Storage persisted to `~/.ralph/qdrant_storage` (survives restarts)
- No authentication needed for local use

### Ollama (Embedding Model)

Local Ollama instance with an embedding model:

```bash
# Install ollama (if not already)
curl -fsSL https://ollama.com/install.sh | sh

# Pull embedding model
ollama pull nomic-embed-text
```

- API at `http://localhost:11434`
- Default model: `nomic-embed-text` (768 dimensions, good for natural language)
- Alternative: `mxbai-embed-large` (1024 dimensions, higher quality)

### Why Only Qdrant + Ollama

No other providers supported. Rationale:
- **Local-only**: All data stays on-machine. No API keys, no cloud, no cost-per-embed.
- **One path**: Ralph's "Single Execution Path Policy" — no provider abstraction layer, no feature flags, no alternate backends.
- **Dev environment match**: Target machine has RTX 3090 — Ollama with GPU acceleration is fast.
- **Kilo Code validated this stack**: Qdrant + Ollama is a proven combination for local code tooling.

## What Goes Into Feature Memory

### Structured Records (per iteration)

Each completed iteration produces a `FeatureMemoryEntry`:

```yaml
# Conceptual schema — actual storage is Qdrant vectors
- iteration: 7
  task_id: 42
  task_title: "Build login form component"
  discipline: frontend
  timestamp: "2026-02-07T14:30:00Z"
  outcome: success | failure | partial
  summary: "Implemented LoginForm.tsx with email/password validation. Had to fix auth middleware response shape first."
  files_touched:
    - path: src/components/auth/LoginForm.tsx
      action: created
    - path: src/middleware/auth.ts
      action: modified
    - path: src/components/auth/LoginForm.test.tsx
      action: created
  errors_encountered:
    - "TypeError: Cannot read property 'user' of undefined in auth middleware"
  decisions:
    - "Used React Hook Form instead of controlled inputs for form state"
    - "Added explicit type guard for auth middleware response"
  tokens_used: 45000
```

### What Gets Embedded and Stored

Each entry becomes **one Qdrant point** with:

- **Vector**: Ollama embedding of a combined text: `"{task_title}\n{summary}\n{errors}\n{decisions}"`
- **Payload** (stored alongside vector, returned with search results):
  - `iteration_number: u32`
  - `task_id: u32`
  - `task_title: String`
  - `discipline: String`
  - `feature: String`
  - `timestamp: String`
  - `outcome: String` (success/failure/partial)
  - `summary: String`
  - `errors: Vec<String>`
  - `decisions: Vec<String>`
  - `files_touched: Vec<{path, action}>`
  - `tokens_used: Option<u32>`

### What Does NOT Go Into Feature Memory

- Raw code content (that's what `context_files` + `fs::read_to_string` is for)
- Full Claude output transcripts (too large, mostly noise)
- Other features' data (strict feature isolation via Qdrant collection-per-feature)

## How Memory Accumulates

### The Extraction Pipeline

After each iteration completes in the loop engine:

```
Claude finishes iteration
        │
        ▼
┌─────────────────────────┐
│  Parse stream-json       │  ← Already happening (claude_client.rs)
│  output for:             │
│  - assistant text        │  ← Currently captured
│  - tool_use events       │  ← NEW: capture file read/write/edit tool calls
│  - result event          │  ← Currently ignored, has cost_usd + summary
└─────────┬───────────────┘
          │
          ▼
┌─────────────────────────┐
│  Extract structured      │  ← NEW: post-iteration extraction
│  data:                   │
│  - files touched         │  (from tool_use: Read, Write, Edit tool names + paths)
│  - summary               │  (from result event or last assistant text)
│  - outcome               │  (task status change in YAML = success, no change = failure)
│  - errors                │  (regex patterns in assistant text: "error", "failed", "TypeError")
└─────────┬───────────────┘
          │
          ▼
┌─────────────────────────┐
│  Determine feature       │  ← From task that was worked on (parse YAML diff or
│                          │     from prompt builder's target task)
└─────────┬───────────────┘
          │
          ▼
┌─────────────────────────┐
│  Check RAG availability  │  ← Is Qdrant reachable? Is Ollama running?
│  (skip if unavailable)   │
└─────────┬───────────────┘
          │ (available)
          ▼
┌─────────────────────────┐
│  Embed via Ollama        │  ← POST http://localhost:11434/api/embed
│  "{title}\n{summary}     │     model: nomic-embed-text
│   \n{errors}\n{decisions}"│     → 768-dim vector
└─────────┬───────────────┘
          │
          ▼
┌─────────────────────────┐
│  Upsert to Qdrant        │  ← Collection: feature-<sha256(feature_name)[:16]>
│  point: {vector, payload} │     Adapted from Kilo Code's ws-<hash> pattern
└─────────────────────────┘
```

### Extracting Files Touched from stream-json

The Claude CLI stream-json format includes tool use events. We need to parse these for file paths:

```json
{"type": "assistant", "message": {"content": [
  {"type": "tool_use", "name": "Read", "input": {"file_path": "/path/to/file.ts"}},
  {"type": "tool_use", "name": "Write", "input": {"file_path": "/path/to/new.ts"}},
  {"type": "tool_use", "name": "Edit", "input": {"file_path": "/path/to/existing.ts"}}
]}}
```

Extract `file_path` from tool_use inputs where name is `Read`/`Write`/`Edit`/`Glob`/`Grep`. Classify as:
- `created` — Write to a path that didn't exist before
- `modified` — Edit or Write to an existing path
- `read` — Read only (reference, not mutation)

### Extracting the Summary

Two sources, in priority order:
1. **Result event** — The `result` stream-json event contains a `result` field with Claude's final summary. Currently ignored by `claude_client.rs`.
2. **Last assistant text** — If no result event, use the last significant assistant text block (>50 chars) as the summary.

### Determining Outcome

Compare task statuses before and after iteration:
- Task moved to `done` → `success`
- Task moved to `in_progress` (was `pending`) → `partial`
- No task status change → `failure`
- Task gained a new comment → `partial` (progress was noted even if not completed)

This reuses the existing stagnation detection mechanism (SHA256 diff) but at task granularity.

### Determining Which Feature Was Worked On

Two approaches, both should be implemented:

1. **From prompt builder** (preferred): When using `build_task_execution_prompt()` (Doc 015 Phase 2), Ralph already knows the target task and its feature. Pass this through.

2. **From YAML diff** (fallback for current dump-everything mode): Compare tasks.yaml before/after iteration, find which task changed status → look up its feature.

## How Memory Is Retrieved

### Query Interface via MCP

Ralph generates an MCP server that exposes semantic search over feature memory:

```
MCP Tools:
  search_feature_memory(query: string, limit?: number, min_score?: number) → MemoryEntry[]
  get_recent_iterations(count?: number) → MemoryEntry[]
  get_feature_files() → FileEntry[]       # all files ever touched for this feature
  get_failed_attempts(task_id?: number) → MemoryEntry[]
```

The MCP server is a small Rust sidecar binary (not a bash script — Qdrant queries are too complex for shell). It:
- Connects to Qdrant REST API at `http://localhost:6333`
- Calls Ollama to embed the search query
- Queries the feature's collection with cosine similarity
- Returns results as JSON-RPC 2.0 responses over stdio

### Query Flow in an Iteration

```
1. Loop engine selects task T in feature F
2. Check RAG availability (Qdrant + Ollama health check)
3. If RAG available:
   a. Prompt builder constructs base prompt (Doc 015 assembly)
   b. MCP generator creates feature-memory MCP sidecar for F
   c. Claude CLI launched with --mcp-config including feature-memory server
   d. Prompt includes: "Use search_feature_memory to check for prior work"
4. If RAG unavailable:
   a. Prompt builder constructs base prompt (Doc 015 assembly) — no memory tools
   b. Log warning: "RAG unavailable — Qdrant/Ollama not reachable"
5. Haiku runs iteration
6. After iteration: extract → embed → store (if RAG available, skip if not)
```

### Why MCP Over Prompt Injection

Doc 015 (Resolved Question #5) already noted: "Feature knowledge will eventually be exposed as MCP resources that Claude can pull on-demand, not injected into the prompt."

Reasons:
- **Token efficiency**: Haiku only retrieves what it needs, not the full history
- **Haiku decides relevance**: The model is better at formulating search queries than Ralph is at pre-selecting context
- **Scales with history**: A feature with 50 iterations doesn't mean 50x prompt size
- **Semantic understanding**: "login form broken" finds entries about "authentication form errors" — keyword search can't do this
- **Consistent pattern**: Already using MCP for ralph-db access in PTY sessions

### Bootstrapping: First Iteration Has No Memory

On the first iteration for a feature, the Qdrant collection is empty. This is fine — the system degrades to Doc 015 behavior (static file injection from `knowledge_paths` + `context_files`). Memory enriches the experience over time but is never required.

## Technology Choice: Qdrant + Ollama

### Why Qdrant

Adapted from [Kilo Code's implementation](https://github.com/Kilo-Org/kilocode) patterns:

| Factor | Decision |
|---|---|
| **Collection naming** | `feature-<sha256(feature_name)[:16]>` per feature (Kilo uses `ws-<hash>` per workspace) |
| **Distance metric** | Cosine (same as Kilo Code) |
| **HNSW config** | `m: 64, ef_construct: 512, on_disk: true` (same as Kilo Code — optimized for quality) |
| **Search params** | `hnsw_ef: 128, exact: false` for fast approximate search |
| **Min score threshold** | `0.4` default (Kilo Code's default — filters irrelevant noise) |
| **Max results** | `20` default (feature memory is smaller than whole-codebase, need fewer results) |
| **Payload fields** | All structured data stored as payload — returned with search results, no second lookup needed |
| **Metadata tracking** | Special metadata point tracks indexing state (Kilo Code pattern) |

**Rust crate**: `qdrant-client` (official Qdrant Rust SDK, uses gRPC for performance)

### Why Ollama

| Factor | Decision |
|---|---|
| **Endpoint** | `http://localhost:11434/api/embed` |
| **Default model** | `nomic-embed-text` (768 dims, fast, good for natural language) |
| **Timeout** | 60s for embeddings, 30s for validation (Kilo Code's values) |
| **Batch size** | 1 per iteration (we embed one combined text per iteration, not batches of code chunks) |
| **Validation** | On startup: check Ollama reachable → check model exists → test embedding capability |
| **Query prefix** | `nomic-embed-text` uses `search_query: ` prefix for queries, `search_document: ` for documents (Kilo Code handles this per-model) |

### Qdrant Collection Schema

```
Collection: feature-<hash>
  Vector: 768 dimensions (nomic-embed-text), Cosine distance

  Point payload schema:
  {
    iteration_number: integer,
    task_id: integer,
    task_title: keyword,
    discipline: keyword,
    feature: keyword,
    timestamp: keyword,
    outcome: keyword,         // "success" | "failure" | "partial"
    summary: text,
    errors: text,             // JSON-serialized Vec<String>
    decisions: text,          // JSON-serialized Vec<String>
    files_touched: text,      // JSON-serialized Vec<{path, action}>
    tokens_used: integer,
    type: keyword             // "iteration" or "metadata" (for tracking)
  }

  Payload indexes (for filtered queries):
    - task_id: integer index (for get_failed_attempts)
    - outcome: keyword index (for filtering by success/failure)
    - type: keyword index (to exclude metadata points from search)
```

## RAG Availability Detection

### Startup Health Check

On Ralph startup (or when loop starts), check both services:

```rust
pub struct RagStatus {
    pub available: bool,
    pub qdrant_ok: bool,
    pub ollama_ok: bool,
    pub ollama_model: Option<String>,    // e.g., "nomic-embed-text"
    pub embedding_dims: Option<u32>,     // e.g., 768
    pub error: Option<String>,           // Human-readable reason if unavailable
}

impl RagHealthCheck {
    /// Check Qdrant: GET http://localhost:6333/healthz
    async fn check_qdrant(&self) -> bool;

    /// Check Ollama: GET http://localhost:11434/api/tags
    /// Then verify embedding model exists and is embedding-capable
    /// (POST /api/embed with test input, check response has embeddings)
    async fn check_ollama(&self) -> (bool, Option<String>, Option<u32>);
}
```

### Behavior When Unavailable

| Scenario | Behavior |
|---|---|
| Both Qdrant + Ollama running | Full RAG: embed, store, search via MCP |
| Qdrant down, Ollama up | RAG disabled. Log warning. Loop runs without memory. |
| Qdrant up, Ollama down | RAG disabled. Log warning. Loop runs without memory. |
| Both down | RAG disabled. No warning beyond initial log. Normal operation. |
| Ollama model not found | RAG disabled. Log: "nomic-embed-text not found. Run: ollama pull nomic-embed-text" |
| Qdrant collection missing | Auto-create on first write. Not an error. |

**RAG status is checked once at loop start**, not per-iteration. If services go down mid-loop, writes fail silently (fire-and-forget). MCP server will return empty results if Qdrant is unreachable.

### Frontend Indicator

Small status badge in the loop control area:
- Green dot: "RAG active" — both services healthy
- Gray dot: "RAG off" — services not detected
- Tooltip shows details: "Qdrant: ok | Ollama: nomic-embed-text (768d)"

## Architecture: Where It Fits

```
                     ┌─────────────────────────────────────────────────┐
                     │               Loop Engine                        │
                     │                                                  │
                     │  0. Check RAG availability (once at loop start)  │
                     │  1. Select task → know feature                   │
                     │  2. Prompt builder assembles context              │
                     │  3. If RAG: MCP generator creates memory sidecar  │
                     │  4. Claude client runs iteration (±MCP)           │
                     │  5. MemoryExtractor parses output  ◄── NEW       │
                     │  6. If RAG: embed + upsert Qdrant  ◄── NEW       │
                     │  7. Stagnation check (existing)                  │
                     └─────────────────────────────────────────────────┘
                                          │
                    ┌─────────────────────┼──────────────────────┐
                    │                     │                      │
              .ralph/db/           Qdrant (Docker)         /tmp/ralph-mcp/
           tasks.yaml            localhost:6333           feature-memory sidecar
           features.yaml         Collections:             mcp-config.json
           disciplines.yaml        feature-<hash1>
           metadata.yaml           feature-<hash2>
                                   ...
                                                     Ollama (localhost:11434)
                                                       nomic-embed-text
```

### New Modules

```
src-tauri/src/
  memory/
    mod.rs            # pub mod extractor; pub mod store; pub mod health; pub mod mcp;
    extractor.rs      # MemoryExtractor — parses stream-json output into MemoryEntry
    store.rs          # MemoryStore — Qdrant + Ollama operations (embed, upsert, search)
    health.rs         # RagHealthCheck — startup validation of Qdrant + Ollama
    mcp.rs            # Generate feature-memory MCP sidecar binary config
```

### New Binary: Feature Memory MCP Sidecar

```
src-tauri/src/bin/
  ralph-feature-memory-mcp.rs    # Standalone MCP server binary
```

This is a small Rust binary that:
- Reads feature name + Qdrant URL + Ollama URL from CLI args
- Implements JSON-RPC 2.0 over stdio (MCP protocol)
- On `search_feature_memory`: calls Ollama to embed query → searches Qdrant → returns results
- On `get_recent_iterations`: scrolls Qdrant collection sorted by timestamp
- On `get_feature_files`: aggregates `files_touched` from all points
- On `get_failed_attempts`: filtered search where `outcome = "failure"`

Ralph generates an MCP config pointing to this binary:

```json
{
  "mcpServers": {
    "feature-memory": {
      "command": "/path/to/ralph-feature-memory-mcp",
      "args": ["--feature", "authentication", "--qdrant", "http://localhost:6333", "--ollama", "http://localhost:11434"]
    }
  }
}
```

### Changes to Existing Modules

| File | Change |
|---|---|
| `claude_client.rs` | Extend `ClaudeOutput` enum with `ToolUse { name, input }` variant. Parse `tool_use` content blocks from assistant events. Capture `result` event summary. |
| `loop_engine.rs` | On loop start: run `RagHealthCheck`. After iteration: call `MemoryExtractor::extract()`, then `MemoryStore::write()` (if RAG available). Pass feature name from task selection. |
| `prompt_builder.rs` | Implement `build_task_execution_prompt()` (Doc 015 Phase 2). If RAG available: add instruction about `search_feature_memory` tool. |
| `mcp_generator.rs` | Add `generate_feature_memory_mcp()` method. Include in MCP config alongside ralph-db server + discipline MCP servers. |
| `commands.rs` | Add `get_rag_status` IPC command for frontend badge. |
| `Cargo.toml` | Add `qdrant-client`, `reqwest` (for Ollama HTTP). |

### Cargo Dependencies

```toml
# src-tauri/Cargo.toml
[dependencies]
qdrant-client = "1.13"     # Official Qdrant Rust client (gRPC)
# reqwest already used? If not:
reqwest = { version = "0.12", features = ["json"] }  # For Ollama HTTP API

# For the MCP sidecar binary
[[bin]]
name = "ralph-feature-memory-mcp"
path = "src/bin/ralph-feature-memory-mcp.rs"
```

## Implementation Phases

### Phase 0: Wire Up Doc 015 Foundation (prerequisite)

Before feature memory can work, the Doc 015 "Assemble" phase must be done. This means:

0a. **Feature/Discipline CRUD for execution context fields**
- `update_feature` accepts `knowledge_paths` and `context_files`
- `update_discipline` accepts `system_prompt`, `skills`, `conventions`, `mcp_servers`
- Frontend forms expose these fields

0b. **`build_task_execution_prompt()`**
- Implement the surgical prompt builder from Doc 015
- Reads `knowledge_paths` files, `context_files` files, assembles per-task prompt
- Loop engine uses this instead of `build_haiku_prompt()` when a specific task is targeted

0c. **Task selection in loop engine**
- Instead of "pick ONE task" in prompt, Ralph selects the task (via `get_ready_tasks()` from Doc 011)
- Enables Ralph to know which feature is being worked on *before* the iteration

### Phase 1: RAG Infrastructure

1a. **Add `qdrant-client` and `reqwest` dependencies**

1b. **Implement `RagHealthCheck`**
- `check_qdrant()` — GET `http://localhost:6333/healthz`
- `check_ollama()` — GET `http://localhost:11434/api/tags`, verify `nomic-embed-text` exists, test embed capability
- Returns `RagStatus` struct
- Called once at loop start, result cached for loop lifetime

1c. **Implement `MemoryStore`**
- `new(feature_name, qdrant_url, ollama_url)` — creates client connections
- `ensure_collection()` — creates Qdrant collection if not exists (768 dims, Cosine, HNSW m:64 ef:512)
- `embed_text(text)` — POST to Ollama `/api/embed`, returns `Vec<f32>`
- `write_entry(entry: MemoryEntry)` — embeds combined text, upserts point to Qdrant
- `search(query, limit, min_score)` — embeds query (with `search_query:` prefix), queries Qdrant
- `get_recent(count)` — scroll collection ordered by timestamp
- `get_files()` — aggregate `files_touched` across all points
- `get_failed_for_task(task_id)` — filtered search where `outcome = "failure"` and `task_id` matches
- `delete_collection()` — cleanup when feature is deleted

1d. **Implement `MemoryExtractor`** (same as before — technology-agnostic)
- Takes collected `Vec<ClaudeOutput>` from an iteration
- Extracts: files touched, summary, errors, decisions, outcome
- Returns `MemoryEntry` struct

1e. **Extend `claude_client.rs`**
- Parse `tool_use` content blocks from assistant events (extract tool name + input)
- Capture `result` event (currently ignored) for iteration summary
- Add `ClaudeOutput::ToolUse { name: String, input: serde_json::Value }` variant
- Add `ClaudeOutput::Result { summary: String, cost_usd: Option<f64> }` variant

1f. **Integrate into loop engine**
- On loop start: run health check, store `RagStatus`
- After each iteration: extract memory entry, if RAG available: embed + store
- On failure to embed/store: log warning, don't break the loop

### Phase 2: Memory Retrieval (MCP sidecar)

2a. **Build `ralph-feature-memory-mcp` binary**
- Standalone Rust binary implementing MCP protocol (JSON-RPC 2.0 over stdio)
- Takes CLI args: `--feature`, `--qdrant`, `--ollama`, `--model`
- Implements tools: `search_feature_memory`, `get_recent_iterations`, `get_feature_files`, `get_failed_attempts`
- Reuses `MemoryStore` from Phase 1 for actual Qdrant/Ollama calls

2b. **Integrate MCP into autonomous loop**
- `claude_client.rs` adds `--mcp-config` to CLI invocation (currently missing)
- Config includes: ralph-db MCP + feature-memory MCP sidecar + discipline MCP servers
- Loop engine passes MCP config path to claude client
- Only includes feature-memory MCP if RAG is available

2c. **Prompt instructions for memory usage**
- Add to task execution prompt (when RAG available): "You have access to `search_feature_memory` — use it to check for prior work, failed approaches, and relevant files before starting."
- Add to Opus review prompt: "Use `get_recent_iterations` to review feature progress across iterations."

2d. **Add `get_rag_status` IPC command** for frontend status badge

### Phase 3: Auto-enrichment

3a. **Auto-populate `context_files` on Feature**
- After N iterations, query Qdrant for all `files_touched` in feature collection
- Files touched 3+ times across iterations → auto-add to feature's `context_files`
- This makes the static file injection (Doc 015) smarter over time

3b. **Auto-generate iteration summaries for stagnation**
- If memory shows same errors repeating across iterations, surface this prominently
- "Warning: This error has occurred in 3 previous iterations" in prompt

### Phase 4: Frontend Visibility (optional, later)

4a. **RAG status badge** in loop controls (green/gray dot)
4b. **Feature detail view** shows memory stats (iteration count, success rate, files touched)
4c. **Timeline view** of feature iterations with outcome badges
4d. **Search UI** — query feature memory from the desktop app (calls `MemoryStore::search` directly via IPC)

## Qdrant Data Lifecycle

### Collection Per Feature

Each feature gets its own Qdrant collection. This provides:
- **Isolation**: Searching "auth" never returns "payments" results
- **Easy deletion**: Drop collection when feature is deleted
- **Independent scaling**: Active features accumulate, finished features can be pruned

### Collection Naming

```
feature-<sha256(feature_name)[:16]>
```

Example: feature named `authentication` → collection `feature-a1b2c3d4e5f6a7b8`

Adapted from Kilo Code's `ws-<hash>` workspace pattern. Hash prevents collection name issues with special characters.

### Point IDs

UUID v5 from: `sha256("{feature_name}-{iteration_number}-{task_id}")` — deterministic, idempotent upserts.

### Pruning Strategy

Not needed initially — feature memory is small (one point per iteration, ~1KB payload each). A feature with 100 iterations = ~100 points. Qdrant handles millions.

If needed later: prune points where `outcome = "success"` and `age > 30 days` — keep failures and decisions longer as they're more valuable for avoiding repeated mistakes.

## Comparison: Before vs After

| Aspect | Without RAG (Doc 015 only) | With RAG (this doc) |
|---|---|---|
| Cross-iteration memory | None — each iteration amnesiac | Semantic search over all prior iterations |
| "What was tried before?" | Haiku doesn't know | `search_feature_memory("login form validation")` |
| "Why did it fail?" | Rediscovers same errors | Errors from prior iterations surfaced |
| File discovery | Manual `context_files` only | Auto-enriched from `files_touched` history |
| Prompt size scaling | Grows with feature complexity | Constant — Haiku pulls on demand via MCP |
| Required infrastructure | None | Qdrant (Docker) + Ollama — **optional** |
| Degradation | N/A — this IS the baseline | Falls back to Doc 015 behavior seamlessly |

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Qdrant Docker not running | RAG disabled gracefully. Log once, run without. |
| Ollama model not pulled | Health check provides actionable error: "Run: ollama pull nomic-embed-text" |
| Ollama embedding latency | One embed per iteration (~100ms on GPU). Not on hot path. Fire-and-forget. |
| Qdrant storage grows | Local Docker volume. Feature collections are tiny. Pruning strategy if needed. |
| MCP sidecar binary distribution | Built as part of `cargo build`. Tauri bundles it as a sidecar. |
| gRPC dependency from qdrant-client | Pulls in tonic + prost. Compile time increase ~30s. One-time cost. |
| Memory extraction is noisy/inaccurate | Start conservative — only extract file paths from tool_use (high confidence). Summaries from result events. |
| Loop engine complexity | RAG is behind availability check. All memory ops are fire-and-forget. Loop never blocks on RAG. |

## Resolved Design Decisions

1. **Qdrant + Ollama only** — No provider abstraction. No OpenAI, no Gemini, no LanceDB. One path. Matches Ralph's Single Execution Path Policy.

2. **RAG is optional** — System works identically without Qdrant/Ollama. RAG enriches but never gates functionality. Health check once at loop start, not per-iteration.

3. **MCP sidecar over bash script** — Qdrant queries need HTTP client + JSON parsing. Too complex for bash. Small Rust binary reuses `MemoryStore` code.

4. **One collection per feature** — Isolation, easy cleanup, no cross-feature leakage. Adapted from Kilo Code's one-collection-per-workspace pattern.

5. **Embed combined text, not individual fields** — One embedding per iteration entry (title + summary + errors + decisions concatenated). Simpler than multi-vector approaches, sufficient for our use case.

6. **nomic-embed-text as default** — 768 dimensions, fast, good for natural language. Kilo Code supports it. Uses query/document prefixes for better retrieval.

7. **Fire-and-forget writes** — Memory store failures never break the loop. Log warning, continue.

8. **Cosine similarity with 0.4 threshold** — Kilo Code's default. Filters out irrelevant noise while keeping semantic matches.

## Dependency on Doc 015 Phases

This plan cannot be implemented in isolation. The dependency chain:

```
Doc 015 Phase 0a (CRUD for execution fields)
    │
    ├──→ Doc 015 Phase 0b (build_task_execution_prompt)
    │        │
    │        └──→ Doc 015 Phase 0c (task selection in loop)
    │                 │
    │                 ├──→ Phase 1 (RAG infrastructure — Qdrant/Ollama/extractor)
    │                 │        │
    │                 │        └──→ Phase 2 (MCP sidecar + retrieval)
    │                 │                 │
    │                 │                 └──→ Phase 3 (auto-enrichment)
    │                 │
    │                 └──→ Phase 2 partial (MCP in autonomous loop — needed regardless)
    │
    └──→ Phase 4 (frontend — independent, can happen anytime)
```

**Phase 0 is the critical path.** Without task selection in the loop engine, Ralph doesn't know which feature is being worked on, and memory can't be scoped.

## Setup Instructions (for users)

### Quick Start

```bash
# 1. Install Ollama + embedding model (one-time)
curl -fsSL https://ollama.com/install.sh | sh
ollama pull nomic-embed-text

# 2. Start Ralph — Qdrant auto-starts as a sidecar
ralph --project /path/to/project
# Console: "RAG: active (Qdrant ok, Ollama nomic-embed-text 768d)"
```

### Verify

```bash
# Check Qdrant (auto-started)
curl http://localhost:6333/healthz
# → "ok"

# Check Ollama
curl http://localhost:11434/api/tags
# → {"models": [{"name": "nomic-embed-text:latest", ...}]}

# Test embedding
curl http://localhost:11434/api/embed -d '{"model": "nomic-embed-text", "input": ["test"]}'
# → {"embeddings": [[0.123, -0.456, ...]]}
```

### Without RAG (default if not set up)

Just run Ralph normally. No Docker, no Ollama needed. Loop works exactly as before. RAG badge shows gray "RAG off".

## Open Questions (for future resolution)

1. **Should Opus reviews query memory differently than Haiku iterations?** Probably — Opus might want `get_recent_iterations(20)` for a broad view, while Haiku wants `search_feature_memory("login form validation")` for specific context.

2. **Should memory include cost tracking?** The `result` event includes `cost_usd`. Could aggregate per-feature cost in Qdrant payload. Useful but not core.

3. **Glob patterns in `context_files`?** Doc 015 parked this. Feature memory's auto-enrichment (Phase 3a) might make this unnecessary.

4. **Memory portability?** Qdrant data is in Docker volume. Not portable by default. Could add export/import via Qdrant snapshots if needed.

5. **Multiple embedding models?** Currently hardcoded to `nomic-embed-text`. Could make configurable later. But "one path" policy says: not now.

6. **Ollama GPU vs CPU?** On target machine (RTX 3090), Ollama auto-detects GPU. On machines without GPU, embedding is slower but still <1s per iteration entry. Acceptable for fire-and-forget.
