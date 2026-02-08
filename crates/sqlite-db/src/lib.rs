pub mod acronym;
mod comments;
mod disciplines;
mod errors;
mod export;
mod features;
mod metadata;
mod recipe_configs;
mod tasks;
pub mod types;

// Re-export public types
pub use recipe_configs::{RecipeConfigData, RecipeConfigInput, SectionSettingsData};
pub use types::{
    CommentAuthor, Discipline, DisciplineInput, Feature, FeatureInput, InferredTaskStatus,
    McpServerConfig, Priority, ProjectMetadata, Task, TaskComment, TaskInput, TaskProvenance,
    TaskStatus,
};

// Re-export ralph-rag learning types used by consumers
pub use ralph_rag::{
    check_deduplication, sanitize_learning_text, select_for_pruning, DeduplicationResult,
    FeatureLearning, LearningSource,
};

use errors::{codes, ralph_map_err};
use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};
use std::path::Path;

pub struct SqliteDb {
    conn: Connection,
}

impl SqliteDb {
    pub fn open(path: &Path) -> Result<Self, String> {
        let mut conn = Connection::open(path)
            .map_err(ralph_map_err!(codes::DB_OPEN, "Failed to open database"))?;

        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;",
        )
        .map_err(ralph_map_err!(codes::DB_OPEN, "Failed to set PRAGMAs"))?;

        let migrations = Migrations::new(vec![
            M::up(include_str!("migrations/001_initial.sql")),
            M::up(include_str!("migrations/002_feature_rag_fields.sql")),
            M::up(include_str!("migrations/003_recipe_configs.sql")),
        ]);

        migrations
            .to_latest(&mut conn)
            .map_err(ralph_map_err!(codes::DB_OPEN, "Failed to run migrations"))?;

        Ok(Self { conn })
    }

    pub fn execute_raw(&self, sql: &str) -> Result<(), String> {
        self.conn
            .execute_batch(sql)
            .map_err(ralph_map_err!(codes::DB_WRITE, "Raw SQL failed"))
    }

    pub fn open_in_memory() -> Result<Self, String> {
        let mut conn = Connection::open_in_memory().map_err(ralph_map_err!(
            codes::DB_OPEN,
            "Failed to open in-memory database"
        ))?;

        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(ralph_map_err!(codes::DB_OPEN, "Failed to set PRAGMAs"))?;

        let migrations = Migrations::new(vec![
            M::up(include_str!("migrations/001_initial.sql")),
            M::up(include_str!("migrations/002_feature_rag_fields.sql")),
            M::up(include_str!("migrations/003_recipe_configs.sql")),
        ]);

        migrations
            .to_latest(&mut conn)
            .map_err(ralph_map_err!(codes::DB_OPEN, "Failed to run migrations"))?;

        Ok(Self { conn })
    }
}
