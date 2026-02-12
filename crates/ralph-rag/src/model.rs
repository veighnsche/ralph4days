//! Core data structures representing what happened in each iteration.
//!
//! An `IterationRecord` is the fundamental unit of feature memory. It captures
//! everything an agent did (or failed to do) in a single execution iteration.
//!
//! ## Who writes these?
//! Ralph's `MemoryExtractor` (not yet built) parses Claude's stream-json output
//! and produces an `IterationRecord`. Agents never write these directly.
//!
//! ## Who reads these?
//! 1. SQLite — embedded as vectors for semantic search
//! 2. MCP sidecar — returns as search results to the next iteration's agent
//! 3. Prompt builder — injects recent failures/learnings into prompts
//! 4. Frontend — displays iteration timeline (secondary)

use serde::{Deserialize, Serialize};

/// The complete record of what happened in one execution iteration, scoped to one feature.
///
/// One iteration = one Claude CLI invocation = one task attempted.
/// This is the source-of-truth record that gets written to JSONL and embedded for search.
///
/// ## Sizing
/// A typical record is 500-2000 bytes in JSONL. A feature with 100 iterations
/// produces ~100-200KB of journal data. This is trivially small.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationRecord {
    /// Which iteration number in the execution sequence (1-indexed, monotonically increasing).
    pub iteration_number: u32,

    /// The task ID that was targeted in this iteration.
    /// Ralph selects the task before launching Claude (Doc 015 Phase 0c).
    pub task_id: u32,

    /// Task title (denormalized for search/display without joins).
    pub task_title: String,

    /// Feature name this iteration belongs to (from the task's feature field).
    /// This is the scoping key — all memory for this feature lives together.
    pub feature: String,

    /// Discipline name (from the task's discipline field).
    /// Useful for filtering: "show me all frontend failures for auth".
    pub discipline: String,

    /// ISO 8601 timestamp when the iteration completed.
    pub timestamp: String,

    /// What was the outcome of this iteration?
    pub outcome: IterationOutcome,

    /// Free-text summary of what happened.
    /// Source: Claude's `result` event or last significant assistant text.
    /// This is the PRIMARY embedding text — it must be meaningful.
    ///
    /// Max 2000 chars (F2: unbounded growth prevention).
    pub summary: String,

    /// Errors encountered during the iteration.
    /// Source: Regex extraction from assistant text + tool error responses.
    ///
    /// These are critical for preventing compounding mistakes.
    /// When an agent searches "what went wrong with login form?", these are
    /// what make the search useful.
    pub errors: Vec<ErrorEntry>,

    /// Decisions the agent made during the iteration.
    /// Source: Extracted from assistant text patterns like "I'll use...", "choosing...",
    /// "decided to...", or from tool_use patterns (picked one library over another).
    ///
    /// These prevent contradictory decisions across iterations:
    /// If iteration 5 decided "use React Hook Form", iteration 8 shouldn't switch to Formik.
    pub decisions: Vec<DecisionEntry>,

    /// Files the agent touched during the iteration.
    /// Source: Parsed from tool_use events (Read, Write, Edit tool calls).
    ///
    /// Used for:
    /// - Auto-populating Feature.context_files (F33: with exclusion patterns)
    /// - Answering "what files are relevant to this feature?"
    /// - Detecting scope violations (touched files outside feature boundaries)
    pub files_touched: Vec<FileTouched>,

    /// Total tokens consumed by this iteration (from result event).
    pub tokens_used: Option<u32>,

    /// Wall-clock duration in milliseconds.
    pub duration_ms: Option<u64>,

    /// Whether this was a Haiku task iteration or an Opus review iteration.
    pub model_tier: ModelTier,
}

impl IterationRecord {
    /// Build the combined text that gets embedded for semantic search.
    ///
    /// This text determines search quality. It concatenates the most semantically
    /// meaningful fields. Order matters — models embed earlier text with more weight.
    ///
    /// Format:
    /// ```text
    /// Task: {task_title}
    /// Outcome: {outcome}
    /// {summary}
    /// Errors: {errors}
    /// Decisions: {decisions}
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use ralph_rag::model::*;
    ///
    /// let record = IterationRecord {
    ///     iteration_number: 7,
    ///     task_id: 42,
    ///     task_title: "Build login form".into(),
    ///     feature: "authentication".into(),
    ///     discipline: "frontend".into(),
    ///     timestamp: "2026-02-07T14:30:00Z".into(),
    ///     outcome: IterationOutcome::Failure,
    ///     summary: "Auth middleware returns wrong shape".into(),
    ///     errors: vec![ErrorEntry {
    ///         message: "TypeError: Cannot read 'user'".into(),
    ///         error_type: Some(ErrorType::Runtime),
    ///         file_path: Some("src/auth.ts".into()),
    ///         line: Some(42),
    ///     }],
    ///     decisions: vec![],
    ///     files_touched: vec![],
    ///     tokens_used: None,
    ///     duration_ms: None,
    ///     model_tier: ModelTier::Haiku,
    /// };
    ///
    /// let text = record.embedding_text();
    /// assert!(text.contains("Build login form"));
    /// assert!(text.contains("failure"));
    /// assert!(text.contains("TypeError"));
    /// ```
    pub fn embedding_text(&self) -> String {
        let mut text = format!(
            "Task: {}\nOutcome: {}\n{}",
            self.task_title,
            self.outcome.as_str(),
            self.summary
        );

        if !self.errors.is_empty() {
            text.push_str("\nErrors:");
            for err in &self.errors {
                text.push_str(&format!("\n- {}", err.message));
                if let Some(path) = &err.file_path {
                    text.push_str(&format!(" (in {path})"));
                }
            }
        }

        if !self.decisions.is_empty() {
            text.push_str("\nDecisions:");
            for dec in &self.decisions {
                text.push_str(&format!("\n- {}", dec.description));
            }
        }

        // Cap at 4000 chars to keep embedding focused (F2, F7)
        if text.len() > 4000 {
            text.truncate(4000);
            text.push_str("\n[truncated]");
        }

        text
    }

    /// Generate a deterministic point ID for vector storage.
    /// Same iteration+task always produces the same ID → idempotent upserts.
    ///
    /// # Examples
    ///
    /// ```
    /// use ralph_rag::model::*;
    ///
    /// let record = IterationRecord {
    ///     iteration_number: 7,
    ///     task_id: 42,
    ///     task_title: "Build login form".into(),
    ///     feature: "authentication".into(),
    ///     discipline: "frontend".into(),
    ///     timestamp: "2026-02-07T14:30:00Z".into(),
    ///     outcome: IterationOutcome::Success,
    ///     summary: "Done".into(),
    ///     errors: vec![],
    ///     decisions: vec![],
    ///     files_touched: vec![],
    ///     tokens_used: None,
    ///     duration_ms: None,
    ///     model_tier: ModelTier::Haiku,
    /// };
    ///
    /// let id1 = record.point_id("/home/user/project");
    /// let id2 = record.point_id("/home/user/project");
    /// assert_eq!(id1, id2); // deterministic
    ///
    /// let id3 = record.point_id("/home/user/other");
    /// assert_ne!(id1, id3); // different project = different ID
    /// ```
    pub fn point_id(&self, project_path: &str) -> String {
        let input = format!(
            "{}::{}::{}::{}",
            project_path, self.feature, self.iteration_number, self.task_id
        );
        let uuid = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, input.as_bytes());
        uuid.to_string()
    }
}

/// Outcome of a single iteration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IterationOutcome {
    /// Task status moved to done
    Success,
    /// No task status change, or explicit errors
    Failure,
    /// Task progressed (e.g. pending → in_progress) but not completed
    Partial,
    /// Iteration hit the timeout limit
    Timeout,
    /// Iteration was rate-limited
    RateLimited,
}

impl IterationOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failure => "failure",
            Self::Partial => "partial",
            Self::Timeout => "timeout",
            Self::RateLimited => "rate_limited",
        }
    }
}

/// Whether this was a regular Haiku iteration or an Opus review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelTier {
    Haiku,
    Opus,
}

/// An error encountered during an iteration.
///
/// Agents don't write these — Ralph extracts them from stream-json output.
/// Patterns: "TypeError:", "Error:", "FAILED", "panic", "Cannot", etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEntry {
    /// The error message text.
    /// Max 500 chars per entry (F2: unbounded growth).
    pub message: String,

    /// Classified error type (if determinable from pattern matching).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_type: Option<ErrorType>,

    /// File where the error occurred (if extractable from stack trace or error message).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,

    /// Line number (if extractable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
}

/// Classification of error types. Helps agents prioritize and search.
///
/// "Show me all runtime errors for authentication" is more useful than
/// "show me all errors for authentication."
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
    /// TypeError, ReferenceError, panic, segfault
    Runtime,
    /// Type mismatch, missing import, syntax error
    Compile,
    /// Test assertion failure
    Test,
    /// Linting error (eslint, clippy, oxlint)
    Lint,
    /// File access denied, network timeout, permission denied
    Permission,
    /// Wrong approach, bad assumption, logic error
    Logic,
    /// Could not classify
    Unknown,
}

/// A decision made by an agent during an iteration.
///
/// Decisions are the most valuable RAG content for preventing contradictions.
/// "Iteration 5 chose React Hook Form" prevents iteration 8 from switching to Formik.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionEntry {
    /// What was decided.
    /// Example: "Used React Hook Form instead of controlled inputs for form state"
    pub description: String,

    /// Why it was decided (if extractable from assistant text).
    /// Example: "Better performance with large forms, built-in validation"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
}

/// A file that was touched (read, written, edited, deleted) during an iteration.
///
/// Source: Parsed from stream-json tool_use events.
/// - Read tool → FileAction::Read
/// - Write tool → FileAction::Created (if new) or FileAction::Modified
/// - Edit tool → FileAction::Modified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTouched {
    /// Relative path from project root.
    /// Absolute paths and paths with `..` are rejected (F9: path validation).
    pub path: String,

    /// What the agent did to this file.
    pub action: FileAction,
}

/// What action was taken on a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileAction {
    Created,
    Modified,
    Read,
    Deleted,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedding_text_includes_key_fields() {
        let record = IterationRecord {
            iteration_number: 7,
            task_id: 42,
            task_title: "Build login form".into(),
            feature: "authentication".into(),
            discipline: "frontend".into(),
            timestamp: "2026-02-07T14:30:00Z".into(),
            outcome: IterationOutcome::Failure,
            summary: "Tried to build login form but auth middleware returns wrong shape".into(),
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
            duration_ms: Some(120_000),
            model_tier: ModelTier::Haiku,
        };

        let text = record.embedding_text();
        assert!(text.contains("Build login form"));
        assert!(text.contains("failure"));
        assert!(text.contains("TypeError"));
        assert!(text.contains("React Hook Form"));
    }

    #[test]
    fn point_id_is_deterministic() {
        let record = IterationRecord {
            iteration_number: 7,
            task_id: 42,
            task_title: "Build login form".into(),
            feature: "authentication".into(),
            discipline: "frontend".into(),
            timestamp: "2026-02-07T14:30:00Z".into(),
            outcome: IterationOutcome::Success,
            summary: "Done".into(),
            errors: vec![],
            decisions: vec![],
            files_touched: vec![],
            tokens_used: None,
            duration_ms: None,
            model_tier: ModelTier::Haiku,
        };

        let id1 = record.point_id("/home/user/ticketmaster");
        let id2 = record.point_id("/home/user/ticketmaster");
        assert_eq!(id1, id2);

        // Different project → different ID (F18: multi-project collision)
        let id3 = record.point_id("/home/user/other-project");
        assert_ne!(id1, id3);
    }

    #[test]
    fn embedding_text_capped_at_4000_chars() {
        let record = IterationRecord {
            iteration_number: 1,
            task_id: 1,
            task_title: "Test".into(),
            feature: "test".into(),
            discipline: "testing".into(),
            timestamp: "2026-02-07T14:30:00Z".into(),
            outcome: IterationOutcome::Failure,
            summary: "x".repeat(5000),
            errors: vec![],
            decisions: vec![],
            files_touched: vec![],
            tokens_used: None,
            duration_ms: None,
            model_tier: ModelTier::Haiku,
        };

        let text = record.embedding_text();
        assert!(text.len() <= 4020); // 4000 + "[truncated]" + newline
        assert!(text.ends_with("[truncated]"));
    }
}
