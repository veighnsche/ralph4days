//! Feature-scoped RAG data model for Ralph's cross-iteration agent memory.
//!
//! This crate defines every type that flows through Ralph's memory pipeline:
//! what gets extracted from Claude iterations, how it's stored in JSONL journals,
//! how it's embedded for semantic search, and how learnings accumulate
//! across iterations to prevent compounding mistakes.
//!
//! ## Architecture
//!
//! ```text
//! Claude CLI (stream-json output)
//!         |
//!         v
//!  RawIterationOutput        collector buffers events during iteration
//!         |
//!         v
//!   ExtractionResult         extractor parses errors, decisions, files
//!         |
//!         v
//!   IterationRecord          the canonical unit of feature memory
//!         |
//!         +---------> JournalEntry ---> .ralph/db/memory/{feature}.jsonl
//!         |                              (source of truth, append-only)
//!         |
//!         +---------> Embedding ------> SQLite comment_embeddings table
//!                                        (vector search index)
//!
//!   FeatureLearning          distilled knowledge from iteration history
//!         |                  (deduplicated, sanitized, prioritized)
//!         v
//!   Prompt builder           injects top learnings into next iteration
//! ```
//!
//! ## Modules
//!
//! - **[`model`]** — Core data structures: [`IterationRecord`], [`IterationOutcome`],
//!   [`ErrorEntry`], [`DecisionEntry`], [`FileTouched`], [`ModelTier`](model::ModelTier).
//!   One `IterationRecord` = one Claude CLI invocation = one task attempted.
//!
//! - **[`extraction`]** — Pipeline types for parsing Claude's stream-json output.
//!   [`ExtractionResult`] bridges raw output to `IterationRecord`. Includes error
//!   patterns, decision patterns, and file exclusion lists.
//!
//! - **[`journal`]** — JSONL source-of-truth storage. One file per feature at
//!   `.ralph/db/memory/{feature}.jsonl`. Append-only, versioned, git-trackable.
//!   [`JournalEntry`] wraps records with a schema version for forward compat.
//!
//! - **[`embedding`]** — Embedding text construction and hashing for feature comments.
//!
//! - **[`learning`]** — Accumulated knowledge with Jaccard deduplication,
//!   negation-aware conflict detection, prompt injection sanitization, and
//!   staleness-paradox-aware pruning. [`FeatureLearning`] is the highest-value RAG content.
//!
//! - **[`config`]** — All configurable values: endpoints, thresholds, limits, prefixes.
//!   [`RagConfig`] defaults to local Ollama with nomic-embed-text.
//!
//! ## Design Principles
//!
//! - **Agents write, agents consume.** Humans rarely touch this data.
//! - **Feature-scoped isolation.** Memory for "authentication" never bleeds into "payments".
//! - **SQLite is the source of truth.** Embeddings are stored alongside comments.
//! - **Prevent compounding mistakes.** If iteration 3 hit an error, iteration 4 must know.
//! - **Observations, not rules.** Learnings are framed as "verify before relying" to prevent
//!   feedback loop amplification (failure class F10).
//!
//! ## Quick Start
//!
//! ```rust
//! use ralph_rag::model::*;
//! use ralph_rag::JournalEntry;
//!
//! // Build an iteration record
//! let record = IterationRecord {
//!     iteration_number: 1,
//!     task_id: 1,
//!     task_title: "Setup auth".into(),
//!     feature: "authentication".into(),
//!     discipline: "backend".into(),
//!     timestamp: "2026-02-07T10:00:00Z".into(),
//!     outcome: IterationOutcome::Success,
//!     summary: "OAuth2 flow implemented".into(),
//!     errors: vec![],
//!     decisions: vec![],
//!     files_touched: vec![],
//!     tokens_used: Some(30000),
//!     duration_ms: Some(90000),
//!     model_tier: ModelTier::Haiku,
//! };
//!
//! // Wrap in a journal entry and serialize to JSONL
//! let entry = JournalEntry::new(record);
//! let line = entry.to_json_line().unwrap();
//! assert!(line.contains("authentication"));
//! ```

pub mod config;
pub mod embedding;
pub mod extraction;
pub mod journal;
pub mod learning;
pub mod model;

// Re-export the primary types that consumers need
pub use config::RagConfig;
pub use embedding::{build_embedding_text, hash_text};
pub use extraction::ExtractionResult;
pub use journal::JournalEntry;
pub use learning::{
    check_deduplication, sanitize_learning_text, select_for_pruning, DeduplicationResult,
    FeatureLearning, LearningSource,
};
pub use model::{
    DecisionEntry, ErrorEntry, ErrorType, FileAction, FileTouched, IterationOutcome,
    IterationRecord,
};
