use crate::types::*;
use crate::SqliteDb;
use ralph_errors::{codes, err_string, RalphResult, RalphResultExt};

impl SqliteDb {
    pub fn get_project_info(&self) -> RalphResult<ProjectMetadata> {
        let result = self.conn.query_row(
            "SELECT project_title, project_description, project_created FROM metadata WHERE id = 1",
            [],
            |row| {
                Ok(ProjectMetadata {
                    title: row.get(0)?,
                    description: row.get(1)?,
                    created: row.get(2)?,
                })
            },
        );

        match result {
            Ok(v) => Ok(v),
            Err(rusqlite::Error::QueryReturnedNoRows) => Err(err_string(
                codes::PROJECT_INIT,
                "Project metadata not initialized",
            )),
            Err(e) => Err(err_string(
                codes::DB_READ,
                format!("Failed to load project metadata: {e}"),
            )),
        }
    }

    pub fn initialize_metadata(
        &self,
        title: String,
        description: Option<String>,
    ) -> RalphResult<()> {
        let now = self.now().format("%Y-%m-%d").to_string();

        self.conn
            .execute(
                "INSERT OR REPLACE INTO metadata (id, project_title, project_description, project_created) \
                 VALUES (1, ?1, ?2, ?3)",
                rusqlite::params![title, description, now],
            )
            .ralph_err(codes::DB_WRITE, "Failed to initialize metadata")?;

        Ok(())
    }
}
