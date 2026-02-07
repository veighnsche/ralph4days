//! Qdrant collection schema and payload types.
//!
//! Defines what gets stored in Qdrant and how collections are named.
//! Qdrant is a DISPOSABLE search index — rebuilt from JSONL journals
//! whenever needed (F19, F22).
//!
//! ## Collection Layout (F18: multi-project collision prevention)
//!
//! ```text
//! Collection naming:
//!   {project_hash}-{feature_hash}
//!
//! Where:
//!   project_hash = sha256(canonical_project_path)[:8]
//!   feature_hash = sha256(feature_name)[:8]
//!
//! Example:
//!   Project: /home/user/ticketmaster
//!   Feature: authentication
//!   Collection: a1b2c3d4-e5f6a7b8
//! ```
//!
//! Two different projects with a feature named "authentication" get
//! different collections because the project hash differs.

use serde::{Deserialize, Serialize};

/// Collection configuration for Qdrant.
///
/// Adapted from Kilo Code's patterns:
/// - HNSW m:64, ef_construct:512 (quality-optimized)
/// - Cosine distance (best for natural language embeddings)
/// - On-disk storage (features are small, disk is fine)
#[derive(Debug, Clone)]
pub struct CollectionConfig {
    /// Vector dimensions (depends on embedding model).
    /// nomic-embed-text = 768, mxbai-embed-large = 1024
    pub vector_size: u32,

    /// HNSW graph parameter: number of edges per node.
    /// Higher = better recall, more memory. 64 is Kilo Code's value.
    pub hnsw_m: u32,

    /// HNSW construction parameter: search width during indexing.
    /// Higher = better index quality, slower build. 512 from Kilo Code.
    pub hnsw_ef_construct: u32,

    /// Search parameter: search width during queries.
    /// Higher = better recall, slower search. 128 is a good balance.
    pub search_ef: u32,

    /// Minimum similarity score to return a result.
    /// Below this = noise. 0.4 from Kilo Code.
    pub min_score: f32,

    /// Maximum results per search.
    pub max_results: u32,
}

impl Default for CollectionConfig {
    fn default() -> Self {
        Self {
            vector_size: 768, // nomic-embed-text default
            hnsw_m: 64,
            hnsw_ef_construct: 512,
            search_ef: 128,
            min_score: 0.4,
            max_results: 20,
        }
    }
}

/// Generate the Qdrant collection name for a feature in a project.
///
/// Format: `{project_hash[:8]}-{feature_hash[:8]}`
///
/// This prevents multi-project collision (F18): two projects with
/// a feature named "authentication" get different collections.
///
/// # Examples
///
/// ```
/// use ralph_rag::qdrant_schema::collection_name;
///
/// let name = collection_name("/home/user/ticketmaster", "authentication");
/// assert_eq!(name.len(), 17); // 8 hex + "-" + 8 hex
///
/// // Same inputs always produce same name
/// let name2 = collection_name("/home/user/ticketmaster", "authentication");
/// assert_eq!(name, name2);
///
/// // Different project = different collection (F18)
/// let other = collection_name("/home/user/other-project", "authentication");
/// assert_ne!(name, other);
/// ```
pub fn collection_name(project_path: &str, feature_name: &str) -> String {
    use sha2::{Digest, Sha256};

    let mut project_hasher = Sha256::new();
    project_hasher.update(project_path.as_bytes());
    let project_hash = hex::encode(project_hasher.finalize());

    let mut feature_hasher = Sha256::new();
    feature_hasher.update(feature_name.as_bytes());
    let feature_hash = hex::encode(feature_hasher.finalize());

    format!("{}-{}", &project_hash[..8], &feature_hash[..8])
}

/// List expected collection names for all features in a project.
/// Used for orphan cleanup (F24).
pub fn expected_collections(project_path: &str, feature_names: &[String]) -> Vec<String> {
    feature_names
        .iter()
        .map(|name| collection_name(project_path, name))
        .collect()
}

/// The payload stored alongside each Qdrant vector.
///
/// All fields are stored as Qdrant payload — returned with search results
/// so no second lookup is needed.
///
/// This is the data that an MCP tool returns to the agent when it
/// calls `search_feature_memory("login form validation")`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPayload {
    /// What type of record this is.
    pub record_type: RecordType,

    /// Iteration number (for ordering and display).
    pub iteration_number: u32,

    /// Task ID (for filtered queries: "get failures for task 42").
    pub task_id: u32,

    /// Task title (human-readable, returned in search results).
    pub task_title: String,

    /// Feature name (for validation — should match the collection's feature).
    pub feature: String,

    /// Discipline (for filtered queries: "show me all frontend failures").
    pub discipline: String,

    /// ISO 8601 timestamp.
    pub timestamp: String,

    /// Outcome as string: "success", "failure", "partial", "timeout", "rate_limited".
    pub outcome: String,

    /// The iteration summary.
    pub summary: String,

    /// JSON-encoded error entries.
    pub errors_json: String,

    /// JSON-encoded decision entries.
    pub decisions_json: String,

    /// JSON-encoded files_touched entries.
    pub files_touched_json: String,

    /// Tokens consumed (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_used: Option<u32>,

    /// The exact text that was embedded.
    /// Stored so we can compare hashes without re-generating (F16: re-computation cost).
    pub embedding_text: String,

    /// Which embedding model produced this vector.
    /// If model changes → dimension mismatch → rebuild needed (F19).
    pub embedding_model: String,

    /// SHA256 of embedding_text.
    /// Used for dedup: don't re-embed if hash unchanged (F16).
    pub embedding_hash: String,
}

impl MemoryPayload {
    /// Create a payload from an IterationRecord.
    ///
    /// # Examples
    ///
    /// ```
    /// use ralph_rag::model::*;
    /// use ralph_rag::qdrant_schema::MemoryPayload;
    ///
    /// let record = IterationRecord {
    ///     iteration_number: 7,
    ///     task_id: 42,
    ///     task_title: "Build login form".into(),
    ///     feature: "authentication".into(),
    ///     discipline: "frontend".into(),
    ///     timestamp: "2026-02-07T14:30:00Z".into(),
    ///     outcome: IterationOutcome::Success,
    ///     summary: "Implemented login form".into(),
    ///     errors: vec![],
    ///     decisions: vec![],
    ///     files_touched: vec![],
    ///     tokens_used: Some(30000),
    ///     duration_ms: None,
    ///     model_tier: ModelTier::Haiku,
    /// };
    ///
    /// let embedding_text = record.embedding_text();
    /// let payload = MemoryPayload::from_record(&record, &embedding_text, "nomic-embed-text");
    ///
    /// assert_eq!(payload.feature, "authentication");
    /// assert_eq!(payload.outcome, "success");
    /// assert!(!payload.embedding_hash.is_empty());
    /// ```
    pub fn from_record(
        record: &crate::model::IterationRecord,
        embedding_text: &str,
        embedding_model: &str,
    ) -> Self {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(embedding_text.as_bytes());
        let embedding_hash = hex::encode(hasher.finalize());

        Self {
            record_type: RecordType::Iteration,
            iteration_number: record.iteration_number,
            task_id: record.task_id,
            task_title: record.task_title.clone(),
            feature: record.feature.clone(),
            discipline: record.discipline.clone(),
            timestamp: record.timestamp.clone(),
            outcome: record.outcome.as_str().to_owned(),
            summary: record.summary.clone(),
            errors_json: serde_json::to_string(&record.errors).unwrap_or_default(),
            decisions_json: serde_json::to_string(&record.decisions).unwrap_or_default(),
            files_touched_json: serde_json::to_string(&record.files_touched).unwrap_or_default(),
            tokens_used: record.tokens_used,
            embedding_text: embedding_text.to_owned(),
            embedding_model: embedding_model.to_owned(),
            embedding_hash,
        }
    }
}

/// Type of record stored in Qdrant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordType {
    /// Per-iteration memory entry.
    Iteration,
    /// Feature-level snapshot (description + architecture + learnings).
    /// One per feature, re-embedded on every feature update.
    FeatureSnapshot,
    /// Collection metadata tracking point (Kilo Code pattern).
    /// Stores indexing state: last iteration indexed, model version, etc.
    Metadata,
}

/// Qdrant payload indexes to create for efficient filtered queries.
///
/// Without indexes, filtered queries do brute-force scan.
/// With indexes, Qdrant uses inverted indexes for fast filtering.
pub fn required_payload_indexes() -> Vec<(&'static str, PayloadIndexType)> {
    vec![
        ("task_id", PayloadIndexType::Integer),
        ("outcome", PayloadIndexType::Keyword),
        ("record_type", PayloadIndexType::Keyword),
        ("discipline", PayloadIndexType::Keyword),
    ]
}

/// Qdrant payload index types (maps to Qdrant's PayloadSchemaType).
#[derive(Debug, Clone, Copy)]
pub enum PayloadIndexType {
    Integer,
    Keyword,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collection_name_includes_project_hash() {
        let name1 = collection_name("/home/user/project-a", "auth");
        let name2 = collection_name("/home/user/project-b", "auth");
        // Same feature, different project → different collection (F18)
        assert_ne!(name1, name2);
    }

    #[test]
    fn collection_name_includes_feature_hash() {
        let name1 = collection_name("/home/user/project", "auth");
        let name2 = collection_name("/home/user/project", "payments");
        // Same project, different feature → different collection
        assert_ne!(name1, name2);
    }

    #[test]
    fn collection_name_is_deterministic() {
        let name1 = collection_name("/home/user/project", "auth");
        let name2 = collection_name("/home/user/project", "auth");
        assert_eq!(name1, name2);
    }

    #[test]
    fn collection_name_format() {
        let name = collection_name("/home/user/ticketmaster", "authentication");
        // Should be {8hex}-{8hex}
        assert_eq!(name.len(), 17); // 8 + 1 + 8
        assert_eq!(&name[8..9], "-");
    }
}
