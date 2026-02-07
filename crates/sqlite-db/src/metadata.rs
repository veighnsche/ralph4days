use crate::types::*;
use crate::SqliteDb;

impl SqliteDb {
    /// Get project metadata.
    pub fn get_project_info(&self) -> ProjectMetadata {
        self.conn
            .query_row(
                "SELECT project_title, project_description, project_created FROM metadata WHERE id = 1",
                [],
                |row| {
                    Ok(ProjectMetadata {
                        title: row.get(0)?,
                        description: row.get(1)?,
                        created: row.get(2)?,
                    })
                },
            )
            .unwrap_or_else(|_| ProjectMetadata {
                title: "Untitled Project".to_string(),
                description: None,
                created: None,
            })
    }

    /// Initialize metadata singleton row. Called during project setup.
    pub fn initialize_metadata(
        &self,
        title: String,
        description: Option<String>,
    ) -> Result<(), String> {
        let now = chrono::Utc::now().format("%Y-%m-%d").to_string();

        self.conn
            .execute(
                "INSERT OR REPLACE INTO metadata (id, project_title, project_description, project_created) \
                 VALUES (1, ?1, ?2, ?3)",
                rusqlite::params![title, description, now],
            )
            .map_err(|e| format!("Failed to initialize metadata: {}", e))?;

        Ok(())
    }
}
