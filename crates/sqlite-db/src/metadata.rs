use crate::types::*;
use crate::SqliteDb;
use ralph_errors::{codes, ralph_map_err};

impl SqliteDb {
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
                title: "Untitled Project".to_owned(),
                description: None,
                created: None,
            })
    }

    pub fn initialize_metadata(
        &self,
        title: String,
        description: Option<String>,
    ) -> Result<(), String> {
        let now = self.now().format("%Y-%m-%d").to_string();

        self.conn
            .execute(
                "INSERT OR REPLACE INTO metadata (id, project_title, project_description, project_created) \
                 VALUES (1, ?1, ?2, ?3)",
                rusqlite::params![title, description, now],
            )
            .map_err(ralph_map_err!(codes::DB_WRITE, "Failed to initialize metadata"))?;

        Ok(())
    }
}
