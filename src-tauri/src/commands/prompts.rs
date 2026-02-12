use super::state::{AppState, CommandContext};
use ralph_macros::ipc_type;
use sqlite_db::{PromptBuilderConfigData, PromptBuilderConfigInput};
use tauri::State;

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptPreviewSection {
    pub name: String,
    pub content: String,
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptPreview {
    pub sections: Vec<PromptPreviewSection>,
    pub full_prompt: String,
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionConfig {
    pub name: String,
    pub enabled: bool,
    pub instruction_override: Option<String>,
}

#[tauri::command]
pub fn preview_custom_prompt_builder(
    state: State<'_, AppState>,
    sections: Vec<SectionConfig>,
    user_input: Option<String>,
) -> Result<PromptPreview, String> {
    let ctx = CommandContext::from_tauri_state(&state);
    let project_path = ctx.locked_project_path()?;

    let overrides: std::collections::HashMap<String, String> = sections
        .iter()
        .filter_map(|s| {
            if s.enabled {
                s.instruction_override
                    .as_ref()
                    .map(|override_val| (s.name.clone(), override_val.clone()))
            } else {
                None
            }
        })
        .collect();

    let ctx = state.build_prompt_context(&project_path, user_input, overrides, None)?;

    let enabled_names: Vec<&str> = sections
        .iter()
        .filter(|s| s.enabled)
        .map(|s| s.name.as_str())
        .collect();

    let built_sections: Vec<PromptPreviewSection> =
        prompt_builder::build_custom_sections(&enabled_names, &ctx)
            .into_iter()
            .map(|s| PromptPreviewSection {
                name: s.name,
                content: s.content,
            })
            .collect();

    let full_prompt = built_sections
        .iter()
        .map(|s| s.content.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");

    Ok(PromptPreview {
        sections: built_sections,
        full_prompt,
    })
}

#[tauri::command]
pub fn list_prompt_builder_configs(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    CommandContext::from_tauri_state(&state).db(sqlite_db::SqliteDb::list_prompt_builder_configs)
}

#[tauri::command]
pub fn get_prompt_builder_config(
    state: State<'_, AppState>,
    name: String,
) -> Result<Option<PromptBuilderConfigData>, String> {
    CommandContext::from_tauri_state(&state).db(|db| db.get_prompt_builder_config(&name))
}

#[tauri::command]
pub fn save_prompt_builder_config(
    state: State<'_, AppState>,
    config: PromptBuilderConfigInput,
) -> Result<(), String> {
    CommandContext::from_tauri_state(&state).db(|db| db.save_prompt_builder_config(config))
}

#[tauri::command]
pub fn delete_prompt_builder_config(
    state: State<'_, AppState>,
    name: String,
) -> Result<(), String> {
    CommandContext::from_tauri_state(&state).db(|db| db.delete_prompt_builder_config(&name))
}
