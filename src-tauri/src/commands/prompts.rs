use super::state::{AppState, CommandContext};
use ralph_macros::ipc_type;
use serde::Deserialize;
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

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptBuilderPreviewArgs {
    pub sections: Vec<SectionConfig>,
    pub user_input: Option<String>,
}

#[tauri::command]
pub fn prompt_builder_preview(
    state: State<'_, AppState>,
    args: PromptBuilderPreviewArgs,
) -> Result<PromptPreview, String> {
    let PromptBuilderPreviewArgs {
        sections,
        user_input,
    } = args;
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
pub fn prompt_builder_config_list(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    CommandContext::from_tauri_state(&state).db(sqlite_db::SqliteDb::list_prompt_builder_configs)
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptBuilderConfigGetArgs {
    pub name: String,
}

#[tauri::command]
pub fn prompt_builder_config_get(
    state: State<'_, AppState>,
    args: PromptBuilderConfigGetArgs,
) -> Result<Option<PromptBuilderConfigData>, String> {
    CommandContext::from_tauri_state(&state).db(|db| db.get_prompt_builder_config(&args.name))
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptBuilderConfigSaveArgs {
    pub config: PromptBuilderConfigInput,
}

#[tauri::command]
pub fn prompt_builder_config_save(
    state: State<'_, AppState>,
    args: PromptBuilderConfigSaveArgs,
) -> Result<(), String> {
    CommandContext::from_tauri_state(&state).db(|db| db.save_prompt_builder_config(args.config))
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptBuilderConfigDeleteArgs {
    pub name: String,
}

#[tauri::command]
pub fn prompt_builder_config_delete(
    state: State<'_, AppState>,
    args: PromptBuilderConfigDeleteArgs,
) -> Result<(), String> {
    CommandContext::from_tauri_state(&state).db(|db| db.delete_prompt_builder_config(&args.name))
}
