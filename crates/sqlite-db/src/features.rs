use crate::types::*;
use crate::SqliteDb;
use ralph_errors::{codes, ralph_err, RalphResultExt};

impl SqliteDb {
    pub fn create_feature(&self, input: FeatureInput) -> Result<(), String> {
        if input.name.trim().is_empty() {
            return ralph_err!(codes::FEATURE_OPS, "Feature name cannot be empty");
        }
        if input.display_name.trim().is_empty() {
            return ralph_err!(codes::FEATURE_OPS, "Feature display name cannot be empty");
        }

        crate::acronym::validate_acronym_format(&input.acronym)?;

        if self.check_exists("features", "name", &input.name)? {
            return ralph_err!(
                codes::FEATURE_OPS,
                "Feature '{}' already exists",
                input.name
            );
        }

        if self.check_exists("features", "acronym", &input.acronym)? {
            return ralph_err!(
                codes::FEATURE_OPS,
                "Acronym '{}' is already used by another feature",
                input.acronym
            );
        }

        let now = self.now().format("%Y-%m-%d").to_string();

        self.conn
            .execute(
                "INSERT INTO features (name, display_name, acronym, description, created, status) \
                 VALUES (?1, ?2, ?3, ?4, ?5, 'active')",
                rusqlite::params![
                    input.name,
                    input.display_name,
                    input.acronym,
                    input.description,
                    now,
                ],
            )
            .ralph_err(codes::DB_WRITE, "Failed to insert feature")?;

        Ok(())
    }

    pub fn update_feature(&self, input: FeatureInput) -> Result<(), String> {
        if input.display_name.trim().is_empty() {
            return ralph_err!(codes::FEATURE_OPS, "Feature display name cannot be empty");
        }

        crate::acronym::validate_acronym_format(&input.acronym)?;

        if !self.check_exists("features", "name", &input.name)? {
            return ralph_err!(
                codes::FEATURE_OPS,
                "Feature '{}' does not exist",
                input.name
            );
        }

        if self.check_exists_excluding(
            "features",
            "acronym",
            &input.acronym,
            "name",
            &input.name,
        )? {
            return ralph_err!(
                codes::FEATURE_OPS,
                "Acronym '{}' is already used by another feature",
                input.acronym
            );
        }

        self.conn
            .execute(
                "UPDATE features SET display_name = ?1, acronym = ?2, description = ?3 WHERE name = ?4",
                rusqlite::params![
                    input.display_name,
                    input.acronym,
                    input.description,
                    input.name,
                ],
            )
            .ralph_err(codes::DB_WRITE, "Failed to update feature")?;

        Ok(())
    }

    pub fn delete_feature(&self, name: String) -> Result<(), String> {
        let feature_id = self.get_id_from_name("features", &name)?;

        let mut stmt = self
            .conn
            .prepare(
                "SELECT rt.id, td.title \
                 FROM runtime_tasks rt \
                 JOIN task_details td ON rt.details_id = td.id \
                 WHERE rt.feature_id = ?1",
            )
            .ralph_err(codes::DB_READ, "Failed to prepare query")?;

        let tasks: Vec<(u32, String)> = stmt
            .query_map([feature_id], |row| Ok((row.get(0)?, row.get(1)?)))
            .ralph_err(codes::DB_READ, "Failed to query tasks")?
            .filter_map(std::result::Result::ok)
            .collect();

        if let Some((task_id, task_title)) = tasks.first() {
            return ralph_err!(
                codes::FEATURE_OPS,
                "Cannot delete feature '{name}': task {task_id} ('{task_title}') belongs to it"
            );
        }

        let affected = self
            .conn
            .execute("DELETE FROM features WHERE name = ?1", [&name])
            .ralph_err(codes::DB_WRITE, "Failed to delete feature")?;

        if affected == 0 {
            return ralph_err!(codes::FEATURE_OPS, "Feature '{name}' does not exist");
        }

        Ok(())
    }

    pub fn get_features(&self) -> Vec<Feature> {
        let Ok(mut stmt) = self.conn.prepare(
            "SELECT id, name, display_name, acronym, description, created, status \
             FROM features ORDER BY name",
        ) else {
            return vec![];
        };

        let mut comments_map = self.get_all_comments_by_feature();

        stmt.query_map([], |row| {
            let status_str: String = row.get(6)?;
            let name: String = row.get(1)?;
            Ok(Feature {
                id: row.get(0)?,
                name,
                display_name: row.get(2)?,
                acronym: row.get(3)?,
                description: row.get(4)?,
                created: row.get(5)?,
                status: FeatureStatus::parse(&status_str).unwrap_or(FeatureStatus::Active),
                comments: vec![],
            })
        })
        .map_or_else(
            |_| vec![],
            |rows| {
                rows.filter_map(std::result::Result::ok)
                    .map(|mut f| {
                        f.comments = comments_map.remove(&f.name).unwrap_or_default();
                        f
                    })
                    .collect()
            },
        )
    }
}
