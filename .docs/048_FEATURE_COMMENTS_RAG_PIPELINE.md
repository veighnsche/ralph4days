# 048: Feature Comments RAG Pipeline (Phase B)

Phase A shipped the `feature_comments` SQLite table, Rust CRUD, IPC commands, frontend UI, and an interim prompt builder that injects ALL comments. Phase B makes it smart: embed comments into Qdrant, search semantically at prompt-build time, inject only the top-K most relevant ones.

## Architecture

```
Feature Comment written to SQLite (Phase A — done)
        ↓
Background embed job (on comment add/update)
        ↓
Ollama: embed(document_prefix + body + reason) → 768-dim vector
        ↓
Qdrant: upsert to collection {proj_hash}-{feat_hash}
        with payload: comment_id, category, author, body, reason
        with RecordType::FeatureComment
        ↓
At prompt-build time:
        ↓
Task title + description + tags → embed as query (query_prefix)
        ↓
Qdrant: search(query_vector, filter=RecordType::FeatureComment, top_k=10, min_score=0.4)
        ↓
Top-K comments injected into prompt via feature_context section
```

## Existing Infrastructure (ralph-rag crate)

### RagConfig (`crates/ralph-rag/src/config.rs`)
- `qdrant_url`: "http://localhost:6333" (REST)
- `qdrant_grpc_url`: "http://localhost:6334" (gRPC)
- `ollama_url`: "http://localhost:11434"
- `embedding_model`: "nomic-embed-text"
- `embedding_dims`: 768
- `min_search_score`: 0.4
- `max_search_results`: 20
- `embedding_timeout_secs`: 60
- `health_check_timeout_secs`: 10
- `query_prefix`: "search_query: "
- `document_prefix`: "search_document: "

### RagStatus (`crates/ralph-rag/src/config.rs`)
- `available(model, dims)` / `unavailable(qdrant_ok, ollama_ok, error)` / `disabled()`
- Checked once at loop start to determine if RAG is operational

### Qdrant Schema (`crates/ralph-rag/src/qdrant_schema.rs`)
- `collection_name(project_path, feature_name)` → `"{proj_hash[:8]}-{feat_hash[:8]}"`
- `CollectionConfig`: HNSW m:64, ef_construct:512, search_ef:128, cosine distance, 768 dims
- `MemoryPayload` — heavy struct designed for iteration records (task_id, outcome, summary, errors_json, etc.)
- `RecordType` enum: `Iteration`, `FeatureSnapshot`, `Metadata`
- `required_payload_indexes()` → task_id(Integer), outcome(Keyword), record_type(Keyword), discipline(Keyword)

### Learning System (`crates/ralph-rag/src/learning.rs`)
- `FeatureLearning` struct with dedup, sanitization, pruning — **now superseded by feature_comments**
- `sanitize_learning_text()` — still useful for sanitizing comment bodies
- `check_deduplication()` — Jaccard similarity, negation-aware — could be reused for comments
- Exports still in `ralph-rag/src/lib.rs` but removed from `sqlite-db/src/lib.rs` re-exports

## What Needs Building

### 1. Ollama HTTP Client
**Location**: new module in `ralph-rag` (or new crate `ralph-embedding`)

```
POST http://localhost:11434/api/embed
{
  "model": "nomic-embed-text",
  "input": "search_document: the text to embed"
}
→ { "embeddings": [[0.1, 0.2, ...]] }  // 768 floats
```

Functions:
- `embed_text(text: &str, config: &RagConfig) → Result<Vec<f32>, EmbedError>`
- `embed_batch(texts: &[String], config: &RagConfig) → Result<Vec<Vec<f32>>, EmbedError>`
- `check_ollama_health(config: &RagConfig) → Result<bool, EmbedError>`

The `document_prefix` / `query_prefix` from RagConfig must be prepended to text before embedding. nomic-embed-text requires these prefixes for good retrieval quality.

### 2. Qdrant Client
**Location**: new module in `ralph-rag`

Dependencies: `qdrant-client` crate (Rust gRPC client)

Functions:
- `ensure_collection(client, collection_name, config) → Result<()>`
- `upsert_points(client, collection_name, points) → Result<()>`
- `search(client, collection_name, query_vector, filter, limit, min_score) → Result<Vec<ScoredPoint>>`
- `delete_points(client, collection_name, point_ids) → Result<()>`

Point ID strategy: use `uuid::v5(NAMESPACE, "{feature_name}:{comment_id}")` for deterministic IDs that survive re-indexing.

### 3. FeatureComment Payload
**New struct** (lighter than `MemoryPayload` which is iteration-focused):

```rust
struct CommentPayload {
    record_type: RecordType,  // RecordType::FeatureComment
    comment_id: i64,
    feature_name: String,
    category: String,
    author: String,
    body: String,
    reason: Option<String>,
    embedding_text: String,
    embedding_model: String,
    embedding_hash: String,  // SHA256 of embedding_text for dedup
}
```

### 4. RecordType Extension
Add `FeatureComment` variant to `RecordType` enum in `qdrant_schema.rs`.
Add `"category"` to `required_payload_indexes()` as Keyword type.

### 5. Sync Job
**Trigger**: After `add_feature_comment` or `update_feature_comment` IPC calls.

Strategy options:
- **Option A**: Embed inline during IPC call (simplest, blocks UI briefly ~100ms)
- **Option B**: Background task that polls for un-embedded comments (more complex, better UX)
- **Option C**: Tauri async command that spawns tokio task (non-blocking IPC response)

Recommendation: Option A for now (single comment embedding is fast), upgrade to C if latency becomes noticeable.

Embedding text formula: `"{document_prefix}{category}: {body}" + optional " (why: {reason})"`

### 6. Search Function
```rust
pub async fn search_feature_comments(
    task_query: &str,       // task title + description + tags
    feature_name: &str,
    project_path: &str,
    config: &RagConfig,
) → Result<Vec<ScoredComment>>
```

Steps:
1. Embed `"{query_prefix}{task_query}"` via Ollama
2. Build Qdrant filter: `record_type == "feature_comment"`
3. Search collection `collection_name(project_path, feature_name)`
4. Map results to `ScoredComment { comment_id, category, body, reason, score }`
5. Filter by `min_search_score`

### 7. Prompt Builder Integration
**File**: `crates/prompt-builder/src/sections/feature_context.rs`

Replace current "inject all comments" with:
1. If RAG available: call `search_feature_comments()` with task context, inject top-K
2. If RAG unavailable: fall back to injecting all comments (current behavior)

This requires `PromptContext` to carry either:
- Pre-computed `relevant_comments: Vec<ScoredComment>` (caller does search)
- Or RAG config + async capability (prompt builder does search)

Recommendation: Pre-compute in the caller. Prompt builder stays synchronous and deterministic.

### 8. Delete Handling
When a comment is deleted from SQLite, its Qdrant point must also be deleted.
Use the deterministic UUID v5 point ID to delete without searching.

## Dependencies to Add

```toml
# ralph-rag/Cargo.toml
qdrant-client = "1"        # Qdrant gRPC client
reqwest = { version = "0.12", features = ["json"] }  # Ollama HTTP
tokio = { workspace = true }  # async runtime (may already be available)
```

## Graceful Degradation

RAG is optional infrastructure. If Qdrant or Ollama is down:
- Comment CRUD works normally (SQLite is source of truth)
- Prompt builder falls back to injecting all comments
- No errors shown to user (just less-smart comment selection)
- `RagStatus::disabled()` / `RagStatus::unavailable()` used for status reporting

## Testing Strategy

1. Unit tests for Ollama client (mock HTTP responses)
2. Unit tests for embedding text generation
3. Integration tests require running Qdrant + Ollama (gated behind feature flag or env var)
4. Prompt builder tests verify fallback behavior when RAG unavailable

## Open Questions

1. **Where to embed**: In `ralph-rag` crate or new `ralph-embedding` crate? ralph-rag already has SHA256/hex deps and the config. Leaning toward extending ralph-rag.
2. **When to embed**: Inline during IPC (simple) vs background job (better UX)? Start with inline.
3. **Collection sharing**: Should FeatureComment points share a collection with Iteration points, or use separate collections? Current `collection_name()` is per-feature, so they'd share. Filter by `record_type` keeps them separate logically.
4. **Batch re-indexing**: Need a "rebuild index" command for when Qdrant is wiped? The JSONL journal pattern from ralph-rag supports this, but comments are in SQLite not JSONL. Would need a `reindex_all_comments(feature_name)` function.
