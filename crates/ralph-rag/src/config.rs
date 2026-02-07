//! RAG system configuration.
//!
//! All configurable values in one place. No magic numbers scattered
//! across the codebase.

use serde::{Deserialize, Serialize};

/// Configuration for the RAG system.
///
/// Stored per-project in `.ralph/rag_config.json` (or defaults if absent).
/// Users can override embedding model, thresholds, etc.
///
/// # Examples
///
/// ```
/// use ralph_rag::config::RagConfig;
///
/// let config = RagConfig::default();
/// assert_eq!(config.embedding_model, "nomic-embed-text");
/// assert_eq!(config.embedding_dims, 768);
/// assert_eq!(config.max_learnings_per_feature, 50);
///
/// // Serialize to JSON for storage
/// let json = serde_json::to_string_pretty(&config).unwrap();
/// assert!(json.contains("nomic-embed-text"));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagConfig {
    /// Qdrant REST API endpoint.
    pub qdrant_url: String,

    /// Qdrant gRPC endpoint (used by Rust client for performance).
    pub qdrant_grpc_url: String,

    /// Ollama API endpoint.
    pub ollama_url: String,

    /// Embedding model name (must be pulled in Ollama).
    pub embedding_model: String,

    /// Expected embedding dimensions (must match model output).
    /// nomic-embed-text = 768, mxbai-embed-large = 1024
    pub embedding_dims: u32,

    /// Minimum similarity score for search results.
    /// Below this = noise. 0.4 from Kilo Code's default.
    pub min_search_score: f32,

    /// Maximum search results to return.
    pub max_search_results: u32,

    /// Maximum learnings per feature before pruning kicks in (F2).
    pub max_learnings_per_feature: usize,

    /// Maximum context_files per feature (F2).
    pub max_context_files_per_feature: usize,

    /// Maximum knowledge_paths per feature (F2).
    pub max_knowledge_paths_per_feature: usize,

    /// Maximum chars for description/architecture fields (F2).
    pub max_text_field_chars: usize,

    /// Maximum chars per learning entry (F2).
    pub max_learning_chars: usize,

    /// Jaccard similarity threshold for near-duplicate detection (F11).
    pub dedup_jaccard_threshold: f64,

    /// Token budget for knowledge_paths file injection.
    /// Files larger than this are truncated in prompt, full via MCP.
    pub knowledge_file_token_budget: usize,

    /// How many times a file must be touched across iterations
    /// before auto-adding to Feature.context_files.
    pub auto_accumulate_touch_threshold: u32,

    /// Embedding timeout in seconds.
    pub embedding_timeout_secs: u64,

    /// Health check timeout in seconds.
    pub health_check_timeout_secs: u64,

    /// nomic-embed-text uses query/document prefixes for better retrieval.
    /// These are prepended to text before embedding.
    pub query_prefix: String,
    pub document_prefix: String,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            qdrant_url: "http://localhost:6333".into(),
            qdrant_grpc_url: "http://localhost:6334".into(),
            ollama_url: "http://localhost:11434".into(),
            embedding_model: "nomic-embed-text".into(),
            embedding_dims: 768,
            min_search_score: 0.4,
            max_search_results: 20,
            max_learnings_per_feature: 50,
            max_context_files_per_feature: 30,
            max_knowledge_paths_per_feature: 10,
            max_text_field_chars: 3000,
            max_learning_chars: 500,
            dedup_jaccard_threshold: 0.7,
            knowledge_file_token_budget: 2000,
            auto_accumulate_touch_threshold: 3,
            embedding_timeout_secs: 60,
            health_check_timeout_secs: 10,
            query_prefix: "search_query: ".into(),
            document_prefix: "search_document: ".into(),
        }
    }
}

/// Status of the RAG system (checked once at loop start).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagStatus {
    /// Whether RAG is fully operational (both Qdrant + Ollama available).
    pub available: bool,

    /// Whether Qdrant is reachable.
    pub qdrant_ok: bool,

    /// Whether Ollama is reachable and has the embedding model.
    pub ollama_ok: bool,

    /// The active embedding model (if Ollama is ok).
    pub embedding_model: Option<String>,

    /// The embedding dimensions (if Ollama is ok).
    pub embedding_dims: Option<u32>,

    /// Human-readable error if unavailable.
    pub error: Option<String>,
}

impl RagStatus {
    /// RAG is fully available.
    pub fn available(model: String, dims: u32) -> Self {
        Self {
            available: true,
            qdrant_ok: true,
            ollama_ok: true,
            embedding_model: Some(model),
            embedding_dims: Some(dims),
            error: None,
        }
    }

    /// RAG is unavailable with a reason.
    pub fn unavailable(qdrant_ok: bool, ollama_ok: bool, error: String) -> Self {
        Self {
            available: false,
            qdrant_ok,
            ollama_ok,
            embedding_model: None,
            embedding_dims: None,
            error: Some(error),
        }
    }

    /// RAG is disabled (no services detected, not an error).
    pub fn disabled() -> Self {
        Self {
            available: false,
            qdrant_ok: false,
            ollama_ok: false,
            embedding_model: None,
            embedding_dims: None,
            error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_uses_nomic() {
        let config = RagConfig::default();
        assert_eq!(config.embedding_model, "nomic-embed-text");
        assert_eq!(config.embedding_dims, 768);
    }

    #[test]
    fn config_serializes_to_json() {
        let config = RagConfig::default();
        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("nomic-embed-text"));
    }
}
