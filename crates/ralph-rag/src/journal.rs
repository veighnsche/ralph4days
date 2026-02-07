//! JSONL journal — the source of truth for iteration history.
//!
//! Qdrant is a DISPOSABLE search index. If the embedding model changes,
//! if Qdrant data corrupts, if collections need rebuilding — the journal
//! survives and Qdrant is rebuilt from it.
//!
//! ## File Layout (F19, F22)
//!
//! ```text
//! .ralph/db/memory/
//!   authentication.jsonl    ← one file per feature
//!   payments.jsonl
//!   event-search.jsonl
//! ```
//!
//! Each line is one `JournalEntry` as JSON. Append-only.
//! Human-inspectable, git-trackable, trivially parseable.
//!
//! ## Why JSONL over SQLite for this?
//!
//! - Append-only is the perfect access pattern (write once, scan to rebuild)
//! - No schema migrations needed — version field handles forward compat
//! - Git-trackable if desired (diff-friendly, line-per-record)
//! - Trivially portable (copy files to new machine)
//! - No locking needed (append is atomic at OS level for small writes)

use crate::model::IterationRecord;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Current schema version for journal entries.
/// Increment when the IterationRecord format changes.
/// Journal readers must handle old versions gracefully.
pub const JOURNAL_VERSION: u8 = 1;

/// One line in the JSONL journal file.
///
/// The version field enables forward compatibility:
/// - Reader sees version 1 → parse as current IterationRecord
/// - Reader sees version 2 → parse with v2 schema (future)
/// - Reader sees unknown version → skip with warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    /// Schema version of this entry.
    pub version: u8,

    /// The iteration record.
    pub record: IterationRecord,
}

impl JournalEntry {
    /// Create a new journal entry from an iteration record.
    ///
    /// # Examples
    ///
    /// ```
    /// use ralph_rag::model::*;
    /// use ralph_rag::journal::*;
    ///
    /// let record = IterationRecord {
    ///     iteration_number: 1,
    ///     task_id: 1,
    ///     task_title: "Setup project".into(),
    ///     feature: "infra".into(),
    ///     discipline: "devops".into(),
    ///     timestamp: "2026-02-07T10:00:00Z".into(),
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
    /// let entry = JournalEntry::new(record);
    /// assert_eq!(entry.version, JOURNAL_VERSION);
    /// ```
    pub fn new(record: IterationRecord) -> Self {
        Self {
            version: JOURNAL_VERSION,
            record,
        }
    }

    /// Serialize to a single JSON line (no trailing newline).
    ///
    /// # Examples
    ///
    /// ```
    /// use ralph_rag::model::*;
    /// use ralph_rag::journal::*;
    ///
    /// let record = IterationRecord {
    ///     iteration_number: 1,
    ///     task_id: 1,
    ///     task_title: "Test".into(),
    ///     feature: "auth".into(),
    ///     discipline: "backend".into(),
    ///     timestamp: "2026-02-07T10:00:00Z".into(),
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
    /// let entry = JournalEntry::new(record);
    /// let line = entry.to_json_line().unwrap();
    /// assert!(line.contains("\"auth\""));
    /// assert!(!line.contains('\n'));
    ///
    /// // Round-trips cleanly
    /// let parsed = JournalEntry::from_json_line(&line).unwrap();
    /// assert_eq!(parsed.record.feature, "auth");
    /// ```
    pub fn to_json_line(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from a single JSON line.
    pub fn from_json_line(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line)
    }
}

/// Path to a feature's journal file.
///
/// ```text
/// {project_path}/.ralph/db/memory/{feature_name}.jsonl
/// ```
pub fn journal_path(project_path: &Path, feature_name: &str) -> PathBuf {
    project_path
        .join(".ralph")
        .join("db")
        .join("memory")
        .join(format!("{}.jsonl", feature_name))
}

/// Path to the memory directory.
pub fn memory_dir(project_path: &Path) -> PathBuf {
    project_path.join(".ralph").join("db").join("memory")
}

/// Read all journal entries for a feature.
///
/// Skips malformed lines with a warning (resilient to corruption).
/// Returns entries in chronological order (append-only = already sorted).
pub fn read_journal(project_path: &Path, feature_name: &str) -> Vec<JournalEntry> {
    let path = journal_path(project_path, feature_name);

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return vec![], // File doesn't exist yet — empty history
    };

    content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            match JournalEntry::from_json_line(line) {
                Ok(entry) => {
                    if entry.version > JOURNAL_VERSION {
                        // Future version — skip but don't error
                        // This allows newer Ralph versions to write entries
                        // that older versions can safely ignore
                        None
                    } else {
                        Some(entry)
                    }
                }
                Err(_) => {
                    // Malformed line — skip silently
                    // Could be from a crash mid-write or manual editing
                    None
                }
            }
        })
        .collect()
}

/// Count journal entries for a feature without loading them all.
pub fn count_entries(project_path: &Path, feature_name: &str) -> usize {
    let path = journal_path(project_path, feature_name);
    match std::fs::read_to_string(&path) {
        Ok(content) => content.lines().filter(|l| !l.trim().is_empty()).count(),
        Err(_) => 0,
    }
}

/// List all features that have journal files.
pub fn list_features_with_history(project_path: &Path) -> Vec<String> {
    let dir = memory_dir(project_path);
    match std::fs::read_dir(&dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                if name.ends_with(".jsonl") {
                    Some(name.trim_end_matches(".jsonl").to_string())
                } else {
                    None
                }
            })
            .collect(),
        Err(_) => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    fn make_test_record(iteration: u32, feature: &str) -> IterationRecord {
        IterationRecord {
            iteration_number: iteration,
            task_id: 1,
            task_title: "Test task".into(),
            feature: feature.into(),
            discipline: "frontend".into(),
            timestamp: "2026-02-07T14:30:00Z".into(),
            outcome: IterationOutcome::Success,
            summary: "Did the thing".into(),
            errors: vec![],
            decisions: vec![],
            files_touched: vec![],
            tokens_used: None,
            duration_ms: None,
            model_tier: ModelTier::Haiku,
        }
    }

    #[test]
    fn journal_entry_roundtrips() {
        let record = make_test_record(7, "auth");
        let entry = JournalEntry::new(record);

        let json = entry.to_json_line().unwrap();
        let parsed = JournalEntry::from_json_line(&json).unwrap();

        assert_eq!(parsed.version, JOURNAL_VERSION);
        assert_eq!(parsed.record.iteration_number, 7);
        assert_eq!(parsed.record.feature, "auth");
    }

    #[test]
    fn journal_path_is_feature_scoped() {
        let path = journal_path(Path::new("/home/user/ticketmaster"), "authentication");
        assert_eq!(
            path,
            PathBuf::from("/home/user/ticketmaster/.ralph/db/memory/authentication.jsonl")
        );
    }
}
