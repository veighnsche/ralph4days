# RAG Integration Guide

**Created:** 2026-02-07
**Status:** Implementation Guide (for future agents)
**Depends on:** Doc 017 (Feature-Scoped RAG System), Doc 018 (Feature Entity Redesign)
**Crate docs:** `crates/ralph-rag/README.md` (type reference, architecture diagram, failure classes)

## Current State

The `ralph-rag` crate defines every type in the memory pipeline but **nothing is wired into the Tauri backend**. This document tells you exactly how to connect them.

### What Exists (ralph-rag crate)

| Module | Status | What It Has |
|--------|--------|------------|
| `model.rs` | Complete | `IterationRecord`, `ErrorEntry`, `DecisionEntry`, `FileTouched`, `IterationOutcome`, `ModelTier` |
| `extraction.rs` | Complete | `ExtractionResult`, `RawIterationOutput`, `ToolUseEvent`, `ResultEvent`, error/decision patterns, file exclusion lists |
| `journal.rs` | Complete | `JournalEntry`, `journal_path()`, `read_journal()`, `count_entries()`, `list_features_with_history()` |
| `qdrant_schema.rs` | Complete | `MemoryPayload`, `CollectionConfig`, `collection_name()`, `expected_collections()`, payload indexes |
| `learning.rs` | Complete | `FeatureLearning`, `LearningSource`, `check_deduplication()`, `sanitize_learning_text()`, `select_for_pruning()` |
| `config.rs` | Complete | `RagConfig` (all tunables), `RagStatus` (health check result) |

### What Does NOT Exist Yet

| Component | Where It Goes | What It Does |
|-----------|--------------|-------------|
| `ralph-rag` as a dependency | `src-tauri/Cargo.toml` | Backend can't use the types without this |
| `MemoryExtractor` | `src-tauri/src/memory/extractor.rs` | Parses `ClaudeOutput` stream into `RawIterationOutput` then `ExtractionResult` |
| `MemoryStore` | `src-tauri/src/memory/store.rs` | Qdrant + Ollama operations (embed, upsert, search) |
| `RagHealthCheck` | `src-tauri/src/memory/health.rs` | Startup check for Qdrant + Ollama availability |
| Feature-memory MCP sidecar | `src-tauri/src/bin/ralph-feature-memory-mcp.rs` | Standalone binary exposing `search_feature_memory` etc. to Claude via MCP |
| `ClaudeOutput` extensions | `src-tauri/src/claude_client.rs` | Capture `tool_use` events and `result` events (currently ignored) |
| Loop engine integration | `src-tauri/src/loop_engine.rs` | Call extractor after each iteration, write to journal + Qdrant |
| Prompt builder integration | `src-tauri/src/prompt_builder.rs` | Inject top learnings, add RAG search instructions when available |
| IPC command | `src-tauri/src/commands.rs` | `get_rag_status` for frontend badge |

## Dependency Chain

Wire things up in this order. Each phase is independently shippable.

```
Phase 1: Journal-only (no Qdrant, no Ollama)
  1a. Add ralph-rag dependency to src-tauri/Cargo.toml
  1b. Extend ClaudeOutput to capture tool_use + result events
  1c. Build MemoryExtractor (stream -> RawIterationOutput -> ExtractionResult)
  1d. Write ExtractionResult -> JournalEntry -> .ralph/db/memory/{feature}.jsonl
  1e. Wire into loop_engine: extract after each iteration, append to journal

Phase 2: Qdrant + Ollama (semantic search)
  2a. Add qdrant-client + reqwest to src-tauri/Cargo.toml
  2b. Build RagHealthCheck (check Qdrant + Ollama at loop start)
  2c. Build MemoryStore (embed via Ollama, upsert to Qdrant)
  2d. Wire into loop_engine: after journal write, also upsert to Qdrant
  2e. Add get_rag_status IPC command

Phase 3: MCP retrieval (agents can search memory)
  3a. Build ralph-feature-memory-mcp binary
  3b. Update mcp_generator.rs to include feature-memory server
  3c. Update prompt_builder.rs to add search instructions when RAG available

Phase 4: Learnings integration
  4a. Wire FeatureLearning into prompt_builder (inject top learnings by priority)
  4b. Auto-extract learnings from failed iterations
  4c. Wire check_deduplication into learning write path
  4d. Wire select_for_pruning into learning cap enforcement
```

## Phase 1: Journal-Only (Start Here)

This phase gives you iteration history on disk with zero external dependencies. No Qdrant, no Ollama, no network calls. Just JSONL files.

### 1a. Add ralph-rag dependency

```toml
# src-tauri/Cargo.toml [dependencies]
ralph-rag = { path = "../crates/ralph-rag" }
```

### 1b. Extend ClaudeOutput in claude_client.rs

Currently `ClaudeOutput` has: `Text(String)`, `RateLimited`, `Complete`, `Error(String)`.

The stream-json parser in `process_stream()` ignores `tool_use` content blocks and `result` events. You need to capture them.

Add variants:

```rust
pub enum ClaudeOutput {
    Text(String),
    ToolUse { name: String, input: serde_json::Value },  // NEW
    Result { summary: Option<String>, duration_ms: Option<u64>, cost_usd: Option<f64> },  // NEW
    RateLimited,
    Complete,
    Error(String),
}
```

In `process_stream()`, the assistant event handler currently extracts only `text` content blocks. Add handling for `tool_use` blocks:

```rust
// In the assistant event content block loop:
if block_type == "tool_use" {
    let name = block.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let input = block.get("input").cloned().unwrap_or(serde_json::Value::Null);
    // Send ClaudeOutput::ToolUse { name, input } through the channel
}
```

For the `result` event (currently in the `"result" | "system" => {}` arm):

```rust
"result" => {
    let summary = event.get("result").and_then(|v| v.as_str()).map(String::from);
    let duration_ms = event.get("duration_ms").and_then(|v| v.as_u64());
    let cost_usd = event.get("cost_usd").and_then(|v| v.as_f64());
    // Send ClaudeOutput::Result { summary, duration_ms, cost_usd }
}
```

### 1c. Build MemoryExtractor

Create `src-tauri/src/memory/extractor.rs`. This consumes the collected `Vec<ClaudeOutput>` from an iteration and produces an `ExtractionResult`.

```rust
use ralph_rag::extraction::*;
use ralph_rag::model::*;

pub struct MemoryExtractor;

impl MemoryExtractor {
    /// Call this after an iteration completes.
    /// Takes the collected outputs and the model tier.
    pub fn extract(
        outputs: &[ClaudeOutput],
        model_tier: ModelTier,
    ) -> ExtractionResult {
        let mut raw = RawIterationOutput::default();

        for output in outputs {
            match output {
                ClaudeOutput::Text(text) => {
                    raw.assistant_text.push(text.clone());
                }
                ClaudeOutput::ToolUse { name, input } => {
                    raw.tool_uses.push(ToolUseEvent {
                        name: name.clone(),
                        input: input.clone(),
                    });
                }
                ClaudeOutput::Result { summary, duration_ms, cost_usd } => {
                    raw.result = Some(ResultEvent {
                        subtype: "success".into(),
                        result_text: summary.clone(),
                        duration_ms: *duration_ms,
                        cost_usd: *cost_usd,
                    });
                }
                ClaudeOutput::RateLimited => {
                    raw.rate_limited = true;
                }
                _ => {}
            }
        }

        // Extract files touched from tool_use events
        let files_touched: Vec<FileTouched> = raw.tool_uses.iter()
            .filter_map(|tu| {
                let path = tu.file_path()?;
                let action = tu.file_action()?;
                // Skip infrastructure files (F33)
                if should_exclude_from_auto_accumulation(&path) {
                    return None;
                }
                Some(FileTouched { path, action })
            })
            .collect();

        // Extract errors from assistant text using patterns from extraction.rs
        let errors = extract_errors(&raw.assistant_text);

        // Extract decisions from assistant text
        let decisions = extract_decisions(&raw.assistant_text);

        // Determine summary (result event preferred, fallback to last text)
        let summary = raw.result.as_ref()
            .and_then(|r| r.result_text.clone())
            .unwrap_or_else(|| {
                raw.assistant_text.last()
                    .cloned()
                    .unwrap_or_default()
                    .chars().take(2000).collect()
            });

        // Determine outcome (caller should refine based on task status change)
        let outcome = if raw.rate_limited {
            IterationOutcome::RateLimited
        } else if raw.timed_out {
            IterationOutcome::Timeout
        } else if !errors.is_empty() {
            IterationOutcome::Failure
        } else {
            IterationOutcome::Partial // Caller upgrades to Success if task completed
        };

        ExtractionResult {
            summary,
            outcome,
            errors,
            decisions,
            files_touched,
            tokens_used: None, // From result event if available
            duration_ms: raw.result.as_ref().and_then(|r| r.duration_ms),
            model_tier,
        }
    }
}
```

The `extract_errors()` and `extract_decisions()` helper functions use the patterns defined in `ralph_rag::extraction::ERROR_PATTERNS` and `DECISION_PATTERNS`. Iterate over assistant text lines, match against patterns, and build `ErrorEntry`/`DecisionEntry` vectors.

### 1d. Write to JSONL journal

After extraction, convert to `IterationRecord` and append to the journal:

```rust
use ralph_rag::journal::{JournalEntry, journal_path, memory_dir};

fn write_to_journal(
    project_path: &Path,
    extraction: ExtractionResult,
    iteration_number: u32,
    task_id: u32,
    task_title: String,
    feature: String,
    discipline: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // Convert ExtractionResult -> IterationRecord
    let record = extraction.into_record(
        iteration_number, task_id, task_title, feature, discipline,
    );

    // Wrap in versioned journal entry
    let entry = JournalEntry::new(record);
    let json_line = entry.to_json_line()?;

    // Ensure directory exists
    let dir = memory_dir(project_path);
    std::fs::create_dir_all(&dir)?;

    // Append to feature journal (atomic at OS level for small writes)
    let path = journal_path(project_path, &entry.record.feature);
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    use std::io::Write;
    writeln!(file, "{}", json_line)?;

    Ok(())
}
```

### 1e. Wire into loop_engine.rs

In the main loop, after each iteration completes (after stagnation check per F28):

```rust
// After stagnation hash check (IMPORTANT: extraction MUST happen AFTER, see F28)
// The extraction may write to features.yaml (auto-accumulating context_files)
// which would corrupt stagnation detection if done before.

let extraction = MemoryExtractor::extract(&collected_outputs, ModelTier::Haiku);

// Determine final outcome from task status change
let final_outcome = if task_status_changed_to_done {
    IterationOutcome::Success
} else if extraction.outcome == IterationOutcome::Failure {
    IterationOutcome::Failure
} else {
    extraction.outcome
};

// Fire-and-forget journal write (never break the loop)
if let Err(e) = write_to_journal(
    &project_path,
    extraction,
    iteration_number,
    current_task_id,
    current_task_title.clone(),
    current_feature.clone(),
    current_discipline.clone(),
) {
    eprintln!("Warning: failed to write journal entry: {}", e);
}
```

**Important:** You need to collect `ClaudeOutput` events during the iteration. Currently the loop engine consumes them for text display and rate limit detection. You'll need to also buffer them into a `Vec<ClaudeOutput>` for the extractor.

### What Phase 1 gives you

After Phase 1, every iteration produces a JSONL file at `.ralph/db/memory/{feature}.jsonl`. You can:

- `cat .ralph/db/memory/authentication.jsonl | jq .` to see iteration history
- Use `ralph_rag::journal::read_journal()` to load all entries for a feature
- Use `ralph_rag::journal::list_features_with_history()` to see which features have memory
- Git-track the journal files for history

No Qdrant or Ollama needed. The journal is the source of truth regardless.

## Phase 2: Qdrant + Ollama

Phase 2 adds semantic search. This requires Qdrant (vector database) and Ollama (local embeddings).

### 2a. Dependencies

```toml
# src-tauri/Cargo.toml [dependencies]
qdrant-client = "1.13"
reqwest = { version = "0.12", features = ["json"] }
```

### 2b. RagHealthCheck

Create `src-tauri/src/memory/health.rs`:

```rust
use ralph_rag::config::{RagConfig, RagStatus};

pub async fn check_rag(config: &RagConfig) -> RagStatus {
    let qdrant_ok = check_qdrant(&config.qdrant_url).await;
    let (ollama_ok, model, dims) = check_ollama(
        &config.ollama_url,
        &config.embedding_model,
    ).await;

    if qdrant_ok && ollama_ok {
        RagStatus::available(model.unwrap(), dims.unwrap())
    } else if !qdrant_ok && !ollama_ok {
        RagStatus::disabled()
    } else {
        let mut reasons = vec![];
        if !qdrant_ok { reasons.push("Qdrant unreachable"); }
        if !ollama_ok { reasons.push("Ollama unreachable or model missing"); }
        RagStatus::unavailable(qdrant_ok, ollama_ok, reasons.join(", "))
    }
}

async fn check_qdrant(url: &str) -> bool {
    // GET {url}/healthz -> "ok"
    reqwest::get(format!("{}/healthz", url))
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

async fn check_ollama(
    url: &str,
    model: &str,
) -> (bool, Option<String>, Option<u32>) {
    // GET {url}/api/tags -> check model exists
    // POST {url}/api/embed -> test embedding, get dimensions
    // Return (ok, model_name, dims)
    todo!()
}
```

Call `check_rag()` once at loop start. Store the `RagStatus`. Pass it through the loop — all RAG operations check `status.available` before doing anything.

### 2c. MemoryStore

Create `src-tauri/src/memory/store.rs`. This handles:

1. **Embedding** — POST to Ollama `/api/embed` with the record's `embedding_text()`
2. **Upserting** — Write the vector + `MemoryPayload` to Qdrant
3. **Searching** — Embed query text, search Qdrant collection, return results

Key types from ralph-rag you'll use:

```rust
use ralph_rag::qdrant_schema::{collection_name, CollectionConfig, MemoryPayload};
use ralph_rag::model::IterationRecord;
use ralph_rag::config::RagConfig;
```

The `MemoryPayload::from_record()` method builds the complete Qdrant payload. The `IterationRecord::embedding_text()` method builds the text to embed. The `IterationRecord::point_id()` method generates the deterministic Qdrant point ID.

```rust
// After journal write (fire-and-forget):
if rag_status.available {
    let embedding_text = record.embedding_text();
    let point_id = record.point_id(&project_path_str);
    let payload = MemoryPayload::from_record(&record, &embedding_text, &config.embedding_model);
    let collection = collection_name(&project_path_str, &record.feature);

    // embed_text() -> Vec<f32> via Ollama
    // upsert_point(collection, point_id, vector, payload) -> via Qdrant client
    // Both are fire-and-forget — log errors, never break the loop
}
```

### 2d. Wire into loop_engine

After the journal write in Phase 1, add the Qdrant upsert:

```rust
if rag_status.available {
    if let Err(e) = memory_store.write_entry(&record, &project_path_str).await {
        eprintln!("Warning: failed to write to Qdrant: {}", e);
        // Never break the loop for RAG failures
    }
}
```

### 2e. IPC command

```rust
#[tauri::command]
pub fn get_rag_status(state: tauri::State<AppState>) -> RagStatus {
    state.rag_status.clone()
}
```

## Phase 3: MCP Retrieval

This is where agents get to search memory during iterations.

### 3a. Feature-memory MCP sidecar

Create `src-tauri/src/bin/ralph-feature-memory-mcp.rs`. This is a standalone Rust binary that:

- Takes CLI args: `--feature`, `--project-path`, `--qdrant-url`, `--ollama-url`, `--model`
- Implements JSON-RPC 2.0 over stdio (MCP protocol)
- Exposes these tools:

| Tool | Description | Implementation |
|------|------------|----------------|
| `search_feature_memory` | Semantic search over iteration history | Embed query via Ollama, search Qdrant, return top results |
| `get_recent_iterations` | Last N iterations for this feature | Scroll Qdrant collection by timestamp |
| `get_feature_files` | All files ever touched for this feature | Aggregate `files_touched` from all points |
| `get_failed_attempts` | Failed iterations, optionally filtered by task | Filtered Qdrant query where `outcome = "failure"` |

The sidecar reuses `ralph_rag::qdrant_schema::collection_name()` for the collection, `ralph_rag::config::RagConfig` for endpoints, and `ralph_rag::qdrant_schema::MemoryPayload` for deserializing results.

### 3b. Update mcp_generator.rs

Currently `mcp_generator.rs` generates bash MCP scripts. Add the feature-memory sidecar to the MCP config:

```json
{
  "mcpServers": {
    "ralph-db": { "command": "/tmp/ralph-mcp-.../ralph-db.sh" },
    "feature-memory": {
      "command": "/path/to/ralph-feature-memory-mcp",
      "args": [
        "--feature", "authentication",
        "--project-path", "/home/user/myproject",
        "--qdrant-url", "http://localhost:6333",
        "--ollama-url", "http://localhost:11434",
        "--model", "nomic-embed-text"
      ]
    }
  }
}
```

Only include `feature-memory` when `rag_status.available == true`.

### 3c. Update prompt_builder.rs

When RAG is available, add to the task execution prompt:

```
You have access to feature memory via MCP tools:
- search_feature_memory("your query") — find relevant prior work, errors, decisions
- get_failed_attempts() — see what failed before for this feature
- get_feature_files() — see which files are relevant to this feature

Use search_feature_memory BEFORE starting work to check for prior approaches and known issues.
```

## Phase 4: Learnings Integration

Learnings are the highest-value RAG content. They're distilled knowledge, not raw iteration data.

### How learnings flow

```
Iteration fails with TypeError
        |
        v
MemoryExtractor produces ExtractionResult with ErrorEntry
        |
        v
Auto-extract learning: "Auth middleware expects User object, not userId string"
        |
        v
check_deduplication() against existing learnings
        |
        +-- Unique -> add as new FeatureLearning (source: Auto)
        +-- Duplicate -> increment existing hit_count
        +-- Conflict -> flag for review (negation detected)
        |
        v
prompt_builder injects top learnings by injection_priority()
        |
        v
Next iteration sees: "Auth middleware expects User object [auto, iteration 7, unreviewed]"
```

### Key functions to wire

| Function | When to call | What it does |
|----------|-------------|-------------|
| `FeatureLearning::auto_extracted()` | After extracting errors from a failed iteration | Creates a new learning from error context |
| `check_deduplication()` | Before adding any new learning | Returns `Unique`, `Duplicate`, or `Conflict` |
| `sanitize_learning_text()` | Already called by constructors | Strips prompt injection patterns (F20) |
| `FeatureLearning::format_for_prompt()` | In prompt_builder when injecting learnings | Formats with provenance metadata |
| `FeatureLearning::injection_priority()` | When sorting learnings for prompt | Higher priority = injected first (more attention from Haiku) |
| `select_for_pruning()` | When learnings exceed 50 per feature | Returns indices of weakest learnings to remove |
| `FeatureLearning::mark_reviewed()` | During Opus review cycles | Upgrades source to OpusReviewed, increments review_count |

### Learning storage

Learnings live in the Feature entity (in SQLite via `sqlite-db` crate, or the YAML files). They are NOT stored in Qdrant — they're small enough to inject directly into prompts.

The prompt builder should sort learnings by `injection_priority()` and inject the top N (budget ~20 learnings, ~2000 tokens) directly into the prompt text.

## File Layout After Full Integration

```
src-tauri/
  Cargo.toml                          # +ralph-rag, +qdrant-client, +reqwest
  src/
    memory/
      mod.rs                          # pub mod extractor; pub mod store; pub mod health;
      extractor.rs                    # MemoryExtractor: ClaudeOutput -> ExtractionResult
      store.rs                        # MemoryStore: Ollama embed + Qdrant upsert/search
      health.rs                       # RagHealthCheck: startup validation
    bin/
      ralph-feature-memory-mcp.rs     # MCP sidecar binary for Claude to search memory
    claude_client.rs                  # MODIFIED: capture tool_use + result events
    loop_engine.rs                    # MODIFIED: extract + journal + qdrant after iteration
    prompt_builder.rs                 # MODIFIED: inject learnings + RAG search instructions
    mcp_generator.rs                  # MODIFIED: include feature-memory MCP sidecar
    commands.rs                       # MODIFIED: add get_rag_status

crates/ralph-rag/                     # Types only (already complete)
  src/
    model.rs                          # IterationRecord, ErrorEntry, DecisionEntry, etc.
    extraction.rs                     # ExtractionResult, RawIterationOutput, patterns
    journal.rs                        # JournalEntry, journal I/O functions
    qdrant_schema.rs                  # MemoryPayload, CollectionConfig, collection_name()
    learning.rs                       # FeatureLearning, dedup, sanitization, pruning
    config.rs                         # RagConfig, RagStatus

.ralph/db/memory/                     # Created at runtime (per project)
  authentication.jsonl                # One JSONL file per feature
  payments.jsonl
```

## Critical Rules

1. **Extraction MUST happen AFTER stagnation hash check** (F28). If extraction writes to `features.yaml` (auto-accumulating `context_files`), it changes the hash, and stagnation detection sees false "progress."

2. **RAG failures NEVER break the loop.** All Qdrant/Ollama operations are fire-and-forget. Log warnings, continue iteration.

3. **RAG status checked ONCE at loop start.** Not per-iteration. If services go down mid-loop, writes fail silently and MCP sidecar returns empty results.

4. **JSONL journal is the source of truth.** Qdrant is a disposable search index. If Qdrant data corrupts or the embedding model changes, rebuild from journals using `read_journal()`.

5. **Learnings are observations, not rules** (F10). Always frame as "observed that..." with provenance. Agents must verify before relying. This prevents feedback loop amplification.

6. **Feature-scoped isolation.** Memory for "authentication" never bleeds into "payments". Each feature gets its own JSONL file and its own Qdrant collection.

## Testing Strategy

### Phase 1 tests (journal only)
- Unit test: `MemoryExtractor::extract()` with mock `ClaudeOutput` events
- Unit test: journal write + read roundtrip
- Integration test: run a mock iteration, verify JSONL file created with correct content
- Existing ralph-rag tests (26 unit + 14 doc) cover all type behavior

### Phase 2 tests (Qdrant + Ollama)
- Unit test: `MemoryStore` with mock HTTP responses (use `wiremock` or similar)
- Integration test: requires running Qdrant + Ollama (skip in CI, run locally)
- Test `RagStatus::disabled()` path — verify loop works without RAG

### Phase 3 tests (MCP sidecar)
- Unit test: MCP sidecar JSON-RPC request/response parsing
- Integration test: spawn sidecar, send MCP tool calls, verify responses
- Test with empty Qdrant collection (first iteration case)

## Quick Reference: ralph-rag Public API

See `crates/ralph-rag/README.md` for the complete type table. The key entry points:

```rust
// Build an iteration record
let record = extraction_result.into_record(iter_num, task_id, title, feature, discipline);

// Write to journal
let entry = JournalEntry::new(record);
let line = entry.to_json_line()?;

// Build embedding text for Qdrant
let text = record.embedding_text();  // capped at 4000 chars

// Generate Qdrant point ID (deterministic)
let id = record.point_id("/path/to/project");

// Build Qdrant payload
let payload = MemoryPayload::from_record(&record, &text, "nomic-embed-text");

// Get collection name (collision-safe)
let collection = collection_name("/path/to/project", "authentication");

// Check learning deduplication
match check_deduplication("new learning text", &existing_learnings) {
    DeduplicationResult::Unique => { /* add */ }
    DeduplicationResult::Duplicate { existing_index } => { /* bump hit_count */ }
    DeduplicationResult::Conflict { existing_index, new_text } => { /* flag */ }
}

// Format learning for prompt injection
let prompt_text = learning.format_for_prompt();
// -> "Auth middleware expects User object [auto, iteration 7, unreviewed]"

// Get default config
let config = RagConfig::default();
// -> Qdrant localhost:6333, Ollama localhost:11434, nomic-embed-text 768d
```
