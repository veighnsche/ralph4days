use crate::SqliteDb;
use ralph_macros::ipc_type;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionSettingsData {
    pub enabled: bool,
    pub instruction_override: Option<String>,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeConfigInput {
    pub name: String,
    pub base_recipe: String,
    pub section_order: Vec<String>,
    pub sections: HashMap<String, SectionSettingsData>,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeConfigData {
    pub name: String,
    pub base_recipe: String,
    pub section_order: Vec<String>,
    pub sections: HashMap<String, SectionSettingsData>,
    pub created: Option<String>,
    pub updated: Option<String>,
}

impl SqliteDb {
    pub fn save_recipe_config(&self, input: RecipeConfigInput) -> Result<(), String> {
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let section_order_json =
            serde_json::to_string(&input.section_order).map_err(|e| format!("JSON error: {e}"))?;
        let sections_json =
            serde_json::to_string(&input.sections).map_err(|e| format!("JSON error: {e}"))?;

        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM recipe_configs WHERE name = ?1",
                [&input.name],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check recipe config: {e}"))?;

        if exists {
            self.conn
                .execute(
                    "UPDATE recipe_configs SET base_recipe = ?1, section_order = ?2, \
                     sections = ?3, updated = ?4 WHERE name = ?5",
                    rusqlite::params![
                        input.base_recipe,
                        section_order_json,
                        sections_json,
                        now,
                        input.name,
                    ],
                )
                .map_err(|e| format!("Failed to update recipe config: {e}"))?;
        } else {
            self.conn
                .execute(
                    "INSERT INTO recipe_configs (name, base_recipe, section_order, sections, created, updated) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![
                        input.name,
                        input.base_recipe,
                        section_order_json,
                        sections_json,
                        now,
                        now,
                    ],
                )
                .map_err(|e| format!("Failed to insert recipe config: {e}"))?;
        }

        Ok(())
    }

    pub fn get_recipe_config(&self, name: &str) -> Result<Option<RecipeConfigData>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT name, base_recipe, section_order, sections, created, updated \
                 FROM recipe_configs WHERE name = ?1",
            )
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let result = stmt.query_row([name], |row| {
            let name: String = row.get(0)?;
            let base_recipe: String = row.get(1)?;
            let section_order_json: String = row.get(2)?;
            let sections_json: String = row.get(3)?;
            let created: Option<String> = row.get(4)?;
            let updated: Option<String> = row.get(5)?;
            Ok((
                name,
                base_recipe,
                section_order_json,
                sections_json,
                created,
                updated,
            ))
        });

        match result {
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("Failed to query recipe config: {e}")),
            Ok((name, base_recipe, section_order_json, sections_json, created, updated)) => {
                let section_order: Vec<String> = serde_json::from_str(&section_order_json)
                    .map_err(|e| format!("Failed to parse section_order: {e}"))?;
                let sections: HashMap<String, SectionSettingsData> =
                    serde_json::from_str(&sections_json)
                        .map_err(|e| format!("Failed to parse sections: {e}"))?;
                Ok(Some(RecipeConfigData {
                    name,
                    base_recipe,
                    section_order,
                    sections,
                    created,
                    updated,
                }))
            }
        }
    }

    pub fn list_recipe_configs(&self) -> Result<Vec<String>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT name FROM recipe_configs ORDER BY name")
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let names = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| format!("Failed to query recipe configs: {e}"))?
            .collect::<Result<Vec<String>, _>>()
            .map_err(|e| format!("Failed to collect recipe config names: {e}"))?;

        Ok(names)
    }

    pub fn delete_recipe_config(&self, name: &str) -> Result<(), String> {
        let rows = self
            .conn
            .execute("DELETE FROM recipe_configs WHERE name = ?1", [name])
            .map_err(|e| format!("Failed to delete recipe config: {e}"))?;
        if rows == 0 {
            return Err(format!("Recipe config \"{name}\" not found"));
        }
        Ok(())
    }
}
