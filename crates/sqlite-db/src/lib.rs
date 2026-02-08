pub mod acronym;
mod comments;
mod disciplines;
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

use ralph_errors::{codes, RalphResultExt};
use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};
use std::path::Path;

pub trait Clock: Send + Sync {
    fn now(&self) -> chrono::DateTime<chrono::Utc>;
}

pub struct RealClock;
impl Clock for RealClock {
    fn now(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }
}

pub struct FixedClock(pub chrono::DateTime<chrono::Utc>);
impl Clock for FixedClock {
    fn now(&self) -> chrono::DateTime<chrono::Utc> {
        self.0
    }
}

pub struct SqliteDb {
    conn: Connection,
    clock: Box<dyn Clock>,
}

impl SqliteDb {
    pub fn open(path: &Path) -> Result<Self, String> {
        let mut conn =
            Connection::open(path).ralph_err(codes::DB_OPEN, "Failed to open database")?;

        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;",
        )
        .ralph_err(codes::DB_OPEN, "Failed to set PRAGMAs")?;

        let migrations = Migrations::new(vec![
            M::up(include_str!("migrations/001_initial.sql")),
            M::up(include_str!("migrations/002_feature_rag_fields.sql")),
            M::up(include_str!("migrations/003_recipe_configs.sql")),
        ]);

        migrations
            .to_latest(&mut conn)
            .ralph_err(codes::DB_OPEN, "Failed to run migrations")?;

        Ok(Self {
            conn,
            clock: Box::new(RealClock),
        })
    }

    pub fn open_with_clock(path: &Path, clock: Box<dyn Clock>) -> Result<Self, String> {
        let mut conn =
            Connection::open(path).ralph_err(codes::DB_OPEN, "Failed to open database")?;

        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;",
        )
        .ralph_err(codes::DB_OPEN, "Failed to set PRAGMAs")?;

        let migrations = Migrations::new(vec![
            M::up(include_str!("migrations/001_initial.sql")),
            M::up(include_str!("migrations/002_feature_rag_fields.sql")),
            M::up(include_str!("migrations/003_recipe_configs.sql")),
        ]);

        migrations
            .to_latest(&mut conn)
            .ralph_err(codes::DB_OPEN, "Failed to run migrations")?;

        Ok(Self { conn, clock })
    }

    pub(crate) fn now(&self) -> chrono::DateTime<chrono::Utc> {
        self.clock.now()
    }

    pub fn execute_raw(&self, sql: &str) -> Result<(), String> {
        self.conn
            .execute_batch(sql)
            .ralph_err(codes::DB_WRITE, "Raw SQL failed")
    }

    pub fn open_in_memory() -> Result<Self, String> {
        Self::open_in_memory_with_clock(Box::new(RealClock))
    }

    pub fn open_in_memory_with_clock(clock: Box<dyn Clock>) -> Result<Self, String> {
        let mut conn = Connection::open_in_memory()
            .ralph_err(codes::DB_OPEN, "Failed to open in-memory database")?;

        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .ralph_err(codes::DB_OPEN, "Failed to set PRAGMAs")?;

        let migrations = Migrations::new(vec![
            M::up(include_str!("migrations/001_initial.sql")),
            M::up(include_str!("migrations/002_feature_rag_fields.sql")),
            M::up(include_str!("migrations/003_recipe_configs.sql")),
        ]);

        migrations
            .to_latest(&mut conn)
            .ralph_err(codes::DB_OPEN, "Failed to run migrations")?;

        Ok(Self { conn, clock })
    }
}
