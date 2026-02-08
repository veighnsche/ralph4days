//! Types for the memory extraction pipeline.
//!
//! After each Claude iteration, Ralph parses the stream-json output
//! and extracts structured data. This module defines what gets extracted
//! and the intermediate types in that pipeline.
//!
//! ## Pipeline Flow
//!
//! ```text
//! stream-json lines → RawIterationOutput → ExtractionResult → IterationRecord
//!                     (collected during)   (parsed after)      (stored in JSONL
//!                                                               + embedded in Qdrant)
//! ```
//!
//! ## IMPORTANT: Extraction happens AFTER stagnation check (F28)
//!
//! The loop engine flow MUST be:
//! 1. Pre-iteration hash
//! 2. Run Claude iteration
//! 3. Post-iteration hash → stagnation check
//! 4. THEN: memory extraction → write to JSONL/Qdrant
//!
//! If extraction writes to features.yaml (auto-accumulating context_files),
//! it changes the file hash, and stagnation detection sees false "progress."

use crate::model::*;
use serde::{Deserialize, Serialize};

/// Raw output collected during a Claude iteration.
///
/// The ClaudeClient streams events. We collect them into this struct
/// for post-iteration extraction.
///
/// This is NOT yet an IterationRecord — it's the raw material.
#[derive(Debug, Clone, Default)]
pub struct RawIterationOutput {
    /// All text blocks from assistant events.
    pub assistant_text: Vec<String>,

    /// Tool use events: (tool_name, input_json).
    /// Source: assistant event content blocks with type "tool_use".
    pub tool_uses: Vec<ToolUseEvent>,

    /// Result event data (if the iteration completed normally).
    pub result: Option<ResultEvent>,

    /// Whether the iteration was rate-limited.
    pub rate_limited: bool,

    /// Whether the iteration timed out.
    pub timed_out: bool,
}

/// A tool_use event extracted from stream-json.
///
/// Claude's stream-json includes these in assistant message content:
/// ```json
/// {"type": "tool_use", "name": "Write", "input": {"file_path": "/path/to/file.ts", ...}}
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUseEvent {
    /// Tool name: "Read", "Write", "Edit", "Glob", "Grep", "Bash", etc.
    pub name: String,

    /// Tool input as raw JSON (varies per tool).
    pub input: serde_json::Value,
}

impl ToolUseEvent {
    /// Extract file_path from a tool_use input, if it's a file operation.
    pub fn file_path(&self) -> Option<String> {
        self.input
            .get("file_path")
            .and_then(|v| v.as_str())
            .map(std::borrow::ToOwned::to_owned)
    }

    /// Classify this tool use as a file action.
    pub fn file_action(&self) -> Option<FileAction> {
        match self.name.as_str() {
            "Write" => Some(FileAction::Created), // May be modified — caller checks existence
            "Edit" => Some(FileAction::Modified),
            "Read" => Some(FileAction::Read),
            _ => None,
        }
    }
}

/// Data from the stream-json "result" event.
///
/// Currently ignored by ClaudeClient — this is what we need to capture.
/// ```json
/// {"type": "result", "subtype": "success", "duration_ms": 2686,
///  "result": "I completed the task...", "cost_usd": 0.045, ...}
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultEvent {
    /// "success" or "error"
    pub subtype: String,

    /// Claude's summary of what it did (the best source for IterationRecord.summary).
    #[serde(default)]
    pub result_text: Option<String>,

    /// Wall-clock duration in milliseconds.
    #[serde(default)]
    pub duration_ms: Option<u64>,

    /// API cost in USD.
    #[serde(default)]
    pub cost_usd: Option<f64>,
}

/// The result of extracting structured data from raw iteration output.
///
/// This is the bridge between RawIterationOutput and IterationRecord.
/// The extractor analyzes the raw output and produces this.
#[derive(Debug, Clone)]
pub struct ExtractionResult {
    /// Summary of what happened.
    /// Source priority: result_event.result_text > last significant assistant text.
    pub summary: String,

    /// Outcome determined from task status changes + error presence.
    pub outcome: IterationOutcome,

    /// Errors extracted from assistant text (regex patterns).
    pub errors: Vec<ErrorEntry>,

    /// Decisions extracted from assistant text (pattern matching).
    pub decisions: Vec<DecisionEntry>,

    /// Files touched, extracted from tool_use events.
    pub files_touched: Vec<FileTouched>,

    /// Total tokens (from result event or estimated from text length).
    pub tokens_used: Option<u32>,

    /// Duration in milliseconds (from result event).
    pub duration_ms: Option<u64>,

    /// Which model ran this iteration.
    pub model_tier: ModelTier,
}

impl ExtractionResult {
    /// Convert to an IterationRecord with context from the loop engine.
    ///
    /// # Examples
    ///
    /// ```
    /// use ralph_rag::extraction::ExtractionResult;
    /// use ralph_rag::model::*;
    ///
    /// let result = ExtractionResult {
    ///     summary: "Implemented OAuth2 flow".into(),
    ///     outcome: IterationOutcome::Success,
    ///     errors: vec![],
    ///     decisions: vec![DecisionEntry {
    ///         description: "Used passport.js for OAuth".into(),
    ///         rationale: None,
    ///     }],
    ///     files_touched: vec![],
    ///     tokens_used: Some(25000),
    ///     duration_ms: Some(60000),
    ///     model_tier: ModelTier::Haiku,
    /// };
    ///
    /// let record = result.into_record(
    ///     3, 12, "Setup OAuth".into(),
    ///     "authentication".into(), "backend".into(),
    /// );
    /// assert_eq!(record.iteration_number, 3);
    /// assert_eq!(record.task_id, 12);
    /// assert_eq!(record.feature, "authentication");
    /// assert!(record.timestamp.is_empty());
    /// ```
    pub fn into_record(
        self,
        iteration_number: u32,
        task_id: u32,
        task_title: String,
        feature: String,
        discipline: String,
    ) -> IterationRecord {
        // timestamp left empty — the persistence layer stamps it at write time
        IterationRecord {
            iteration_number,
            task_id,
            task_title,
            feature,
            discipline,
            timestamp: String::new(),
            outcome: self.outcome,
            summary: self.summary,
            errors: self.errors,
            decisions: self.decisions,
            files_touched: self.files_touched,
            tokens_used: self.tokens_used,
            duration_ms: self.duration_ms,
            model_tier: self.model_tier,
        }
    }
}

/// Error patterns to search for in assistant text.
///
/// These regex-friendly patterns identify errors in Claude's output.
/// Used by the extractor to populate ExtractionResult.errors.
pub const ERROR_PATTERNS: &[(&str, ErrorType)] = &[
    ("TypeError:", ErrorType::Runtime),
    ("ReferenceError:", ErrorType::Runtime),
    ("SyntaxError:", ErrorType::Runtime),
    ("Error:", ErrorType::Runtime),
    ("panic", ErrorType::Runtime),
    ("FAILED", ErrorType::Test),
    ("AssertionError", ErrorType::Test),
    ("test failed", ErrorType::Test),
    ("TS2", ErrorType::Compile), // TypeScript errors: TS2304, TS2322, etc.
    ("error[E", ErrorType::Compile), // Rust errors: error[E0308], etc.
    ("cannot find", ErrorType::Compile),
    ("Permission denied", ErrorType::Permission),
    ("EACCES", ErrorType::Permission),
    ("ENOENT", ErrorType::Permission),
];

/// Decision patterns to search for in assistant text.
///
/// These identify when Claude made an explicit choice.
pub const DECISION_PATTERNS: &[&str] = &[
    "I'll use",
    "I will use",
    "choosing",
    "decided to",
    "going with",
    "opted for",
    "switching to",
    "instead of",
    "rather than",
    "approach:",
    "strategy:",
];

/// Files to exclude from auto-accumulation into Feature.context_files (F33).
///
/// These infrastructure files add noise, not signal.
pub const AUTO_ACCUMULATE_EXCLUDE: &[&str] = &[
    "package.json",
    "package-lock.json",
    "bun.lockb",
    "yarn.lock",
    "pnpm-lock.yaml",
    "tsconfig.json",
    "vite.config.ts",
    "vite.config.js",
    ".gitignore",
    ".eslintrc",
    "biome.json",
    ".prettierrc",
    "CLAUDE.md",
    "CLAUDE.RALPH.md",
    "Cargo.toml",
    "Cargo.lock",
    "justfile",
    "Justfile",
];

/// Directory prefixes to exclude from auto-accumulation (F33).
pub const AUTO_ACCUMULATE_EXCLUDE_DIRS: &[&str] = &[
    "node_modules/",
    ".git/",
    "target/",
    "dist/",
    "build/",
    ".ralph/",
    ".specs/",
    ".docs/",
];

/// Check if a file path should be excluded from auto-accumulation.
///
/// # Examples
///
/// ```
/// use ralph_rag::extraction::should_exclude_from_auto_accumulation;
///
/// // Infrastructure files are excluded
/// assert!(should_exclude_from_auto_accumulation("package.json"));
/// assert!(should_exclude_from_auto_accumulation("node_modules/foo/bar.js"));
/// assert!(should_exclude_from_auto_accumulation("some/path/file.log"));
///
/// // Source files are included
/// assert!(!should_exclude_from_auto_accumulation("src/components/Login.tsx"));
/// assert!(!should_exclude_from_auto_accumulation("tests/auth.test.ts"));
/// ```
pub fn should_exclude_from_auto_accumulation(path: &str) -> bool {
    let filename = path.rsplit('/').next().unwrap_or(path);

    // Check filename exclusions
    if AUTO_ACCUMULATE_EXCLUDE.contains(&filename) {
        return true;
    }

    // Check directory prefix exclusions
    if AUTO_ACCUMULATE_EXCLUDE_DIRS
        .iter()
        .any(|dir| path.contains(dir))
    {
        return true;
    }

    // Exclude lockfiles, logs, sourcemaps
    if filename.ends_with(".lock") || filename.ends_with(".log") || filename.ends_with(".map") {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_use_extracts_file_path() {
        let event = ToolUseEvent {
            name: "Write".into(),
            input: serde_json::json!({
                "file_path": "/home/user/project/src/auth.ts",
                "content": "..."
            }),
        };
        assert_eq!(
            event.file_path(),
            Some("/home/user/project/src/auth.ts".into())
        );
    }

    #[test]
    fn tool_use_classifies_actions() {
        assert_eq!(
            ToolUseEvent {
                name: "Write".into(),
                input: serde_json::json!({})
            }
            .file_action(),
            Some(FileAction::Created)
        );
        assert_eq!(
            ToolUseEvent {
                name: "Edit".into(),
                input: serde_json::json!({})
            }
            .file_action(),
            Some(FileAction::Modified)
        );
        assert_eq!(
            ToolUseEvent {
                name: "Bash".into(),
                input: serde_json::json!({})
            }
            .file_action(),
            None
        );
    }

    #[test]
    fn excludes_infrastructure_files() {
        assert!(should_exclude_from_auto_accumulation("package.json"));
        assert!(should_exclude_from_auto_accumulation(
            "node_modules/foo/bar.js"
        ));
        assert!(should_exclude_from_auto_accumulation(
            ".ralph/db/tasks.yaml"
        ));
        assert!(should_exclude_from_auto_accumulation("some/path/file.log"));
    }

    #[test]
    fn includes_source_files() {
        assert!(!should_exclude_from_auto_accumulation(
            "src/components/LoginForm.tsx"
        ));
        assert!(!should_exclude_from_auto_accumulation("src/lib/auth.ts"));
        assert!(!should_exclude_from_auto_accumulation("tests/auth.test.ts"));
    }
}
