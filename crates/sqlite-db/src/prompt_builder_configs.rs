use crate::SqliteDb;
use ralph_errors::{codes, ralph_err, RalphResultExt};
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
pub struct PromptBuilderConfigInput {
    pub name: String,
    pub base_prompt: String,
    pub section_order: Vec<String>,
    pub sections: HashMap<String, SectionSettingsData>,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptBuilderConfigData {
    pub name: String,
    pub base_prompt: String,
    pub section_order: Vec<String>,
    pub sections: HashMap<String, SectionSettingsData>,
    pub created: Option<String>,
    pub updated: Option<String>,
}

impl SqliteDb {
    pub fn save_prompt_builder_config(
        &self,
        input: PromptBuilderConfigInput,
    ) -> Result<(), String> {
        let now = self.now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let section_order_json =
            serde_json::to_string(&input.section_order).ralph_err(codes::DB_WRITE, "JSON error")?;
        let sections_json =
            serde_json::to_string(&input.sections).ralph_err(codes::DB_WRITE, "JSON error")?;

        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM prompt_builder_configs WHERE name = ?1",
                [&input.name],
                |row| row.get(0),
            )
            .ralph_err(codes::DB_READ, "Failed to check prompt builder config")?;

        if exists {
            self.conn
                .execute(
                    "UPDATE prompt_builder_configs SET base_prompt = ?1, section_order = ?2, \
                     sections = ?3, updated = ?4 WHERE name = ?5",
                    rusqlite::params![
                        input.base_prompt,
                        section_order_json,
                        sections_json,
                        now,
                        input.name,
                    ],
                )
                .ralph_err(codes::DB_WRITE, "Failed to update prompt builder config")?;
        } else {
            self.conn
                .execute(
                    "INSERT INTO prompt_builder_configs (name, base_prompt, section_order, sections, created, updated) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![
                        input.name,
                        input.base_prompt,
                        section_order_json,
                        sections_json,
                        now,
                        now,
                    ],
                )
                .ralph_err(codes::DB_WRITE, "Failed to insert prompt builder config")?;
        }

        Ok(())
    }

    pub fn get_prompt_builder_config(
        &self,
        name: &str,
    ) -> Result<Option<PromptBuilderConfigData>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT name, base_prompt, section_order, sections, created, updated \
                 FROM prompt_builder_configs WHERE name = ?1",
            )
            .ralph_err(codes::DB_READ, "Failed to prepare query")?;

        let result = stmt.query_row([name], |row| {
            let name: String = row.get(0)?;
            let base_prompt: String = row.get(1)?;
            let section_order_json: String = row.get(2)?;
            let sections_json: String = row.get(3)?;
            let created: Option<String> = row.get(4)?;
            let updated: Option<String> = row.get(5)?;
            Ok((
                name,
                base_prompt,
                section_order_json,
                sections_json,
                created,
                updated,
            ))
        });

        match result {
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => ralph_err!(codes::DB_READ, "Failed to query prompt builder config: {e}"),
            Ok((name, base_prompt, section_order_json, sections_json, created, updated)) => {
                let section_order: Vec<String> = serde_json::from_str(&section_order_json)
                    .ralph_err(codes::DB_READ, "Failed to parse section_order")?;
                let sections: HashMap<String, SectionSettingsData> =
                    serde_json::from_str(&sections_json)
                        .ralph_err(codes::DB_READ, "Failed to parse sections")?;
                Ok(Some(PromptBuilderConfigData {
                    name,
                    base_prompt,
                    section_order,
                    sections,
                    created,
                    updated,
                }))
            }
        }
    }

    pub fn list_prompt_builder_configs(&self) -> Result<Vec<String>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT name FROM prompt_builder_configs ORDER BY name")
            .ralph_err(codes::DB_READ, "Failed to prepare query")?;

        let names = stmt
            .query_map([], |row| row.get(0))
            .ralph_err(codes::DB_READ, "Failed to query prompt builder configs")?
            .collect::<Result<Vec<String>, _>>()
            .ralph_err(
                codes::DB_READ,
                "Failed to collect prompt builder config names",
            )?;

        Ok(names)
    }

    pub fn delete_prompt_builder_config(&self, name: &str) -> Result<(), String> {
        let rows = self
            .conn
            .execute("DELETE FROM prompt_builder_configs WHERE name = ?1", [name])
            .ralph_err(codes::DB_WRITE, "Failed to delete prompt builder config")?;
        if rows == 0 {
            return ralph_err!(codes::DB_READ, "Prompt builder config \"{name}\" not found");
        }
        Ok(())
    }
}
