pub mod acronym;
mod comments;
mod disciplines;
mod export;
mod features;
mod metadata;
mod stats;
mod tasks;
pub mod types;

// Re-export public types
pub use types::{
    CommentAuthor, Discipline, Feature, FeatureInput, GroupStats, InferredTaskStatus,
    McpServerConfig, Priority, ProjectMetadata, ProjectProgress, Task, TaskComment, TaskInput,
    TaskProvenance, TaskStatus,
};

// Re-export ralph-rag learning types used by consumers
pub use ralph_rag::{
    check_deduplication, sanitize_learning_text, select_for_pruning, DeduplicationResult,
    FeatureLearning, LearningSource,
};

use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};
use std::path::Path;

/// SQLite-backed database for Ralph project data.
pub struct SqliteDb {
    conn: Connection,
}

impl SqliteDb {
    /// Open (or create) a SQLite database at the given path.
    /// Sets PRAGMAs, runs migrations, and returns a ready-to-use database.
    pub fn open(path: &Path) -> Result<Self, String> {
        let mut conn =
            Connection::open(path).map_err(|e| format!("Failed to open database: {}", e))?;

        // Set PRAGMAs for performance and correctness
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;",
        )
        .map_err(|e| format!("Failed to set PRAGMAs: {}", e))?;

        // Run migrations
        let migrations = Migrations::new(vec![
            M::up(include_str!("migrations/001_initial.sql")),
            M::up(include_str!("migrations/002_feature_rag_fields.sql")),
        ]);

        migrations
            .to_latest(&mut conn)
            .map_err(|e| format!("Failed to run migrations: {}", e))?;

        Ok(Self { conn })
    }

    /// Execute raw SQL (for test fixtures only — not for production use).
    pub fn execute_raw(&self, sql: &str) -> Result<(), String> {
        self.conn
            .execute_batch(sql)
            .map_err(|e| format!("Raw SQL failed: {}", e))
    }

    /// Open an in-memory database (for testing).
    pub fn open_in_memory() -> Result<Self, String> {
        let mut conn = Connection::open_in_memory()
            .map_err(|e| format!("Failed to open in-memory database: {}", e))?;

        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|e| format!("Failed to set PRAGMAs: {}", e))?;

        let migrations = Migrations::new(vec![
            M::up(include_str!("migrations/001_initial.sql")),
            M::up(include_str!("migrations/002_feature_rag_fields.sql")),
        ]);

        migrations
            .to_latest(&mut conn)
            .map_err(|e| format!("Failed to run migrations: {}", e))?;

        Ok(Self { conn })
    }
}
