use ralph_macros::ipc_type;
use serde::Deserialize;

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagePromptData {
    pub positive: String,
    pub negative: String,
}

#[allow(dead_code)]
#[ipc_type]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisciplinePromptData {
    pub subject: String,
    pub accent_color: String,
}

#[allow(dead_code)]
#[ipc_type]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StackPromptsData {
    pub name: String,
    pub description: String,
    pub positive: String,
    pub negative: String,
    pub disciplines: std::collections::HashMap<String, DisciplinePromptData>,
}

#[derive(Deserialize)]
pub struct BuildPromptParams {
    pub stack_id: u8,
    pub discipline_name: String,
}

#[tauri::command]
pub fn get_image_prompt_config() -> Result<String, String> {
    let config = sqlite_db::image_prompts::ImagePromptsConfig::load()?;
    serde_json::to_string_pretty(&config).map_err(|e| format!("Failed to serialize config: {e}"))
}

#[tauri::command]
pub fn build_image_prompt(params: BuildPromptParams) -> Result<ImagePromptData, String> {
    let config = sqlite_db::image_prompts::ImagePromptsConfig::load()?;
    let (positive, negative) = config.build_prompt(params.stack_id, &params.discipline_name)?;

    Ok(ImagePromptData { positive, negative })
}

#[tauri::command]
pub fn get_stack_disciplines(stack_id: u8) -> Result<Vec<String>, String> {
    let config = sqlite_db::image_prompts::ImagePromptsConfig::load()?;
    config.get_stack_disciplines(stack_id)
}
