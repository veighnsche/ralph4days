# ralph-rag

Feature-scoped RAG data model for Ralph's cross-iteration agent memory. Defines every type that flows through the memory pipeline: what gets extracted from Claude iterations, how it's stored in JSONL journals, how it's indexed in Qdrant, and how learnings accumulate across iterations.

## Architecture

```text
Claude CLI (stream-json output)
        |
        v
 RawIterationOutput        collector buffers events during iteration
        |
        v
  ExtractionResult         extractor parses errors, decisions, files
        |
        v
  IterationRecord          the canonical unit of feature memory
        |
        +---------> JournalEntry ---> .ralph/db/memory/{feature}.jsonl
        |                              (source of truth, append-only)
        |
        +---------> MemoryPayload --> Qdrant collection {proj_hash}-{feat_hash}
                                       (disposable search index)

  FeatureLearning          distilled knowledge from iteration history
        |                  (deduplicated, sanitized, prioritized)
        v
  Prompt builder           injects top learnings into next iteration
```

## Modules

| Module | Purpose | Key Types |
|--------|---------|-----------|
| `model` | Core iteration data structures | `IterationRecord`, `IterationOutcome`, `ErrorEntry`, `DecisionEntry`, `FileTouched`, `ModelTier` |
| `extraction` | Pipeline types for parsing Claude output | `ExtractionResult`, `RawIterationOutput`, `ToolUseEvent`, `ResultEvent` |
| `journal` | JSONL source-of-truth storage | `JournalEntry`, `journal_path()`, `read_journal()` |
| `qdrant_schema` | Qdrant collection schema and naming | `MemoryPayload`, `CollectionConfig`, `RecordType`, `collection_name()` |
| `learning` | Accumulated knowledge with dedup/sanitization | `FeatureLearning`, `LearningSource`, `check_deduplication()`, `sanitize_learning_text()` |
| `config` | All configurable values in one place | `RagConfig`, `RagStatus` |

## Public Type Reference

### Core Records

| Type | Description |
|------|-------------|
| `IterationRecord` | Complete record of one loop iteration scoped to one feature |
| `IterationOutcome` | Enum: `Success`, `Failure`, `Partial`, `Timeout`, `RateLimited` |
| `ErrorEntry` | An error extracted from iteration output with optional file/line |
| `ErrorType` | Classification: `Runtime`, `Compile`, `Test`, `Lint`, `Permission`, `Logic`, `Unknown` |
| `DecisionEntry` | A decision the agent made (description + optional rationale) |
| `FileTouched` | A file path + action taken during an iteration |
| `FileAction` | Enum: `Created`, `Modified`, `Read`, `Deleted` |
| `ModelTier` | Enum: `Haiku` (task iteration) or `Opus` (review iteration) |

### Extraction Pipeline

| Type | Description |
|------|-------------|
| `ExtractionResult` | Bridge between raw output and `IterationRecord` |
| `RawIterationOutput` | Buffered events collected during a Claude CLI invocation |
| `ToolUseEvent` | A tool_use event with name and input JSON |
| `ResultEvent` | Data from the stream-json "result" event |

### Storage & Search

| Type | Description |
|------|-------------|
| `JournalEntry` | Versioned wrapper around `IterationRecord` for JSONL storage |
| `MemoryPayload` | Qdrant point payload with all searchable/filterable fields |
| `CollectionConfig` | Qdrant HNSW parameters and search thresholds |
| `RecordType` | Enum: `Iteration`, `FeatureSnapshot`, `Metadata` |

### Learnings

| Type | Description |
|------|-------------|
| `FeatureLearning` | A single learning with provenance, dedup tracking, review state |
| `LearningSource` | Enum: `Auto`, `Agent`, `Human`, `OpusReviewed` (priority ordered) |
| `DeduplicationResult` | Enum: `Unique`, `Duplicate`, `Conflict` (negation-aware) |

### Configuration

| Type | Description |
|------|-------------|
| `RagConfig` | All RAG tunables: endpoints, thresholds, limits, prefixes |
| `RagStatus` | Runtime health check result for Qdrant + Ollama |

## File Layout on Disk

```text
.ralph/
  db/
    memory/
      authentication.jsonl    # one JSONL file per feature
      payments.jsonl           # each line = one JournalEntry
      event-search.jsonl
  rag_config.json              # optional RagConfig overrides
```

Qdrant collections are named `{sha256(project_path)[:8]}-{sha256(feature)[:8]}` to prevent multi-project collisions.

## Failure Classes

This crate addresses these failure modes from the RAG design document:

| ID | Failure | Mitigation |
|----|---------|------------|
| F1 | Bulk replacement of learnings | PATCH/append-only semantics |
| F2 | Unbounded field growth | Max chars on summary (2000), learning (500), embedding (4000) |
| F3 | Missing provenance | Every learning tracks source, iteration, timestamp |
| F7 | Embedding text too long | Truncation at 4000 chars in `embedding_text()` |
| F9 | Path traversal | `FileTouched.path` rejects absolute paths and `..` |
| F10 | Feedback loop amplification | Learnings framed as observations, not rules |
| F11 | Duplicate learnings | Jaccard similarity >0.7 dedup with `check_deduplication()` |
| F16 | Re-embedding unchanged records | SHA256 hash in `MemoryPayload.embedding_hash` |
| F18 | Multi-project collection collision | Project hash in collection names |
| F19 | Embedding model change | `embedding_model` stored per-payload; journal enables rebuild |
| F20 | Prompt injection in learnings | `sanitize_learning_text()` strips injection patterns |
| F21 | Negation-aware dedup | "use X" vs "don't use X" detected as conflict, not duplicate |
| F22 | Qdrant data loss | JSONL journal is source of truth; Qdrant is disposable |
| F24 | Orphan collections | `expected_collections()` for cleanup |
| F25 | Task-specific vs feature-wide | `FeatureLearning.task_id` scoping |
| F27 | False verification | `reviewed` field, not "verified"; Opus hallucinates too |
| F28 | Extraction corrupts stagnation | Extraction runs AFTER stagnation hash check |
| F31 | Missing learning context | `reason` field explains WHY the learning exists |
| F33 | Noisy context_files | `should_exclude_from_auto_accumulation()` filters infra files |
| F35 | Staleness paradox | Conservative pruning: only auto+unreviewed+hit_count=1 |
| F36 | Silent deserialization fallback | Custom `Deserialize` impl, no `serde(untagged)` |

## Usage Examples

### Creating an iteration record

```rust
use ralph_rag::model::*;

let record = IterationRecord {
    iteration_number: 7,
    task_id: 42,
    task_title: "Build login form".into(),
    feature: "authentication".into(),
    discipline: "frontend".into(),
    timestamp: "2026-02-07T14:30:00Z".into(),
    outcome: IterationOutcome::Failure,
    summary: "Auth middleware returns wrong shape".into(),
    errors: vec![ErrorEntry {
        message: "TypeError: Cannot read property 'user' of undefined".into(),
        error_type: Some(ErrorType::Runtime),
        file_path: Some("src/middleware/auth.ts".into()),
        line: Some(42),
    }],
    decisions: vec![DecisionEntry {
        description: "Used React Hook Form for form state".into(),
        rationale: Some("Better performance with validation".into()),
    }],
    files_touched: vec![],
    tokens_used: Some(45000),
    duration_ms: Some(120000),
    model_tier: ModelTier::Haiku,
};

// Generate embedding text for Qdrant
let text = record.embedding_text();
assert!(text.contains("Build login form"));

// Generate deterministic Qdrant point ID
let id = record.point_id("/home/user/myproject");
```

### Checking learning deduplication

```rust
use ralph_rag::learning::*;

let existing = vec![
    FeatureLearning::auto_extracted(
        "Auth middleware expects User object not userId".into(), 5, None,
    ),
];

match check_deduplication("Auth middleware expects User object instead of userId", &existing) {
    DeduplicationResult::Duplicate { existing_index } => {
        // Increment hit_count on existing[existing_index]
    }
    DeduplicationResult::Conflict { existing_index, new_text } => {
        // Negation detected — flag for human review
    }
    DeduplicationResult::Unique => {
        // Safe to add as new learning
    }
}
```

### Writing to the journal

```rust
use ralph_rag::{JournalEntry, IterationRecord};
use ralph_rag::model::*;

let record = IterationRecord {
    iteration_number: 1,
    task_id: 1,
    task_title: "Setup project".into(),
    feature: "infrastructure".into(),
    discipline: "devops".into(),
    timestamp: "2026-02-07T10:00:00Z".into(),
    outcome: IterationOutcome::Success,
    summary: "Project scaffolded".into(),
    errors: vec![],
    decisions: vec![],
    files_touched: vec![],
    tokens_used: None,
    duration_ms: None,
    model_tier: ModelTier::Haiku,
};

let entry = JournalEntry::new(record);
let json_line = entry.to_json_line().unwrap();
// Append json_line + "\n" to .ralph/db/memory/infrastructure.jsonl
```

## Design Principles

- **Agents write, agents consume.** Humans rarely touch this data.
- **Feature-scoped isolation.** Memory for "authentication" never bleeds into "payments".
- **Qdrant is disposable.** JSONL is the source of truth. Qdrant can be rebuilt anytime.
- **Prevent compounding mistakes.** If iteration 3 hit an error, iteration 4 must know.
- **Observations, not rules.** Learnings are framed as "verify before relying" to prevent feedback loop amplification.

## License

MIT OR Apache-2.0
