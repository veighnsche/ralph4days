pub mod acronym;
mod agent_sessions;
mod comment_embeddings;
mod disciplines;
mod export;
mod feature_comments;
mod features;
mod helpers;
mod metadata;
mod prompt_builder_configs;
mod signals;
mod tasks;
pub mod types;

// Re-export public types
pub use comment_embeddings::ScoredCommentRow;
pub use feature_comments::AddFeatureCommentInput;
pub use prompt_builder_configs::{
    PromptBuilderConfigData, PromptBuilderConfigInput, SectionSettingsData,
};
pub use signals::{
    AskSignalInput, BlockedSignalInput, DoneSignalInput, FlagSignalInput, LearnedSignalInput,
    PartialSignalInput, StuckSignalInput, SuggestSignalInput,
};
pub use types::{
    AgentSession, AgentSessionCreateInput, AgentSessionUpdateInput, Discipline, DisciplineInput,
    Feature, FeatureComment, FeatureInput, FeatureStatus, McpServerConfig, Priority,
    ProjectMetadata, Task, TaskInput, TaskProvenance, TaskSignal, TaskSignalComment,
    TaskSignalCommentCreateInput, TaskSignalSummary, TaskStatus,
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
    pub fn open(path: &Path, clock: Option<Box<dyn Clock>>) -> Result<Self, String> {
        let mut conn =
            Connection::open(path).ralph_err(codes::DB_OPEN, "Failed to open database")?;

        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;",
        )
        .ralph_err(codes::DB_OPEN, "Failed to set PRAGMAs")?;

        let migrations = Migrations::new(vec![M::up(include_str!("migrations/001_initial.sql"))]);

        migrations
            .to_latest(&mut conn)
            .ralph_err(codes::DB_OPEN, "Failed to run migrations")?;

        Ok(Self {
            conn,
            clock: clock.unwrap_or_else(|| Box::new(RealClock)),
        })
    }

    pub(crate) fn now(&self) -> chrono::DateTime<chrono::Utc> {
        self.clock.now()
    }

    pub fn with_transaction<T, F>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&Self) -> Result<T, String>,
    {
        self.conn
            .execute_batch("BEGIN IMMEDIATE TRANSACTION;")
            .ralph_err(codes::DB_WRITE, "Failed to start transaction")?;

        match f(self) {
            Ok(value) => {
                self.conn
                    .execute_batch("COMMIT;")
                    .ralph_err(codes::DB_WRITE, "Failed to commit transaction")?;
                Ok(value)
            }
            Err(err) => {
                if let Err(rollback_err) = self.conn.execute_batch("ROLLBACK;") {
                    return Err(format!(
                        "{err} (rollback failed: [R-{}] {rollback_err})",
                        codes::DB_WRITE
                    ));
                }
                Err(err)
            }
        }
    }

    // ⚠️ WARNING: DO NOT add execute_raw() or any raw SQL execution method here!
    //
    // Rationale: Raw SQL execution bypasses type safety, validation, and the proper
    // MCP signal interface. It enables "reward hacking" where code takes shortcuts
    // instead of using the typed API (insert_done_signal, insert_ask_signal, etc.).
    //
    // If you need to execute SQL:
    // 1. For signals: Use the typed methods in signals.rs (insert_*_signal)
    // 2. For features: Use the typed methods in features.rs
    // 3. For new operations: Create a new typed method with proper validation
    //
    // The API server and fixture generator MUST use the same typed interface that
    // agents use via MCP. No exceptions.

    pub fn open_in_memory(clock: Option<Box<dyn Clock>>) -> Result<Self, String> {
        let mut conn = Connection::open_in_memory()
            .ralph_err(codes::DB_OPEN, "Failed to open in-memory database")?;

        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .ralph_err(codes::DB_OPEN, "Failed to set PRAGMAs")?;

        let migrations = Migrations::new(vec![M::up(include_str!("migrations/001_initial.sql"))]);

        migrations
            .to_latest(&mut conn)
            .ralph_err(codes::DB_OPEN, "Failed to run migrations")?;

        Ok(Self {
            conn,
            clock: clock.unwrap_or_else(|| Box::new(RealClock)),
        })
    }
}
