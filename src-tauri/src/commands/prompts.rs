use super::state::{get_locked_project_path, parse_prompt_type, AppState};
use tauri::State;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptPreviewSection {
    pub name: String,
    pub content: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptPreview {
    pub sections: Vec<PromptPreviewSection>,
    pub full_prompt: String,
}

#[tauri::command]
pub fn preview_prompt(
    state: State<'_, AppState>,
    prompt_type: String,
    instruction_override: Option<String>,
    user_input: Option<String>,
) -> Result<PromptPreview, String> {
    let project_path = get_locked_project_path(&state)?;
    let pt = parse_prompt_type(&prompt_type)?;
    let overrides = instruction_override.map_or_else(std::collections::HashMap::new, |text| {
        let section_name = format!("{prompt_type}_instructions");
        let mut map = std::collections::HashMap::new();
        map.insert(section_name, text);
        map
    });
    let ctx = state.build_prompt_context(&project_path, user_input, overrides)?;

    let sections: Vec<PromptPreviewSection> = prompt_builder::build_sections(pt, &ctx)
        .into_iter()
        .map(|s| PromptPreviewSection {
            name: s.name,
            content: s.content,
        })
        .collect();

    let full_prompt = sections
        .iter()
        .map(|s| s.content.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");

    Ok(PromptPreview {
        sections,
        full_prompt,
    })
}

#[tauri::command]
pub fn get_default_instructions(prompt_type: String) -> Result<String, String> {
    let pt = parse_prompt_type(&prompt_type)?;
    Ok(prompt_builder::default_instructions(pt))
}

#[tauri::command]
pub fn save_prompt_instructions(
    state: State<'_, AppState>,
    prompt_type: String,
    text: String,
) -> Result<(), String> {
    let project_path = get_locked_project_path(&state)?;
    let prompts_dir = project_path.join(".ralph").join("prompts");
    std::fs::create_dir_all(&prompts_dir)
        .map_err(|e| format!("Failed to create prompts dir: {e}"))?;
    let file_path = prompts_dir.join(format!("{prompt_type}_instructions.md"));
    std::fs::write(&file_path, &text).map_err(|e| format!("Failed to save instructions: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn load_prompt_instructions(
    state: State<'_, AppState>,
    prompt_type: String,
) -> Result<Option<String>, String> {
    let project_path = get_locked_project_path(&state)?;
    let file_path = project_path
        .join(".ralph")
        .join("prompts")
        .join(format!("{prompt_type}_instructions.md"));
    match std::fs::read_to_string(&file_path) {
        Ok(text) => Ok(Some(text)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("Failed to read instructions: {e}")),
    }
}

#[tauri::command]
pub fn reset_prompt_instructions(
    state: State<'_, AppState>,
    prompt_type: String,
) -> Result<(), String> {
    let project_path = get_locked_project_path(&state)?;
    let file_path = project_path
        .join(".ralph")
        .join("prompts")
        .join(format!("{prompt_type}_instructions.md"));
    match std::fs::remove_file(&file_path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("Failed to delete instructions: {e}")),
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionConfig {
    pub name: String,
    pub enabled: bool,
    pub instruction_override: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomRecipe {
    pub name: String,
    pub base_recipe: Option<String>,
    pub sections: Vec<SectionConfig>,
}

#[tauri::command]
pub fn get_section_metadata() -> Vec<prompt_builder::SectionInfo> {
    prompt_builder::sections::metadata::all_sections()
}

#[tauri::command]
pub fn get_recipe_sections(prompt_type: String) -> Result<Vec<SectionConfig>, String> {
    let pt = parse_prompt_type(&prompt_type)?;
    let names = prompt_builder::get_recipe_section_names(pt);
    let all_meta = prompt_builder::sections::metadata::all_sections();

    Ok(names
        .iter()
        .map(|name| SectionConfig {
            name: (*name).to_owned(),
            enabled: true,
            instruction_override: None,
        })
        .chain(
            all_meta
                .iter()
                .filter(|info| !names.contains(&info.name))
                .map(|info| SectionConfig {
                    name: info.name.to_owned(),
                    enabled: false,
                    instruction_override: None,
                }),
        )
        .collect())
}

#[tauri::command]
pub fn preview_custom_recipe(
    state: State<'_, AppState>,
    sections: Vec<SectionConfig>,
    user_input: Option<String>,
) -> Result<PromptPreview, String> {
    let project_path = get_locked_project_path(&state)?;

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

    let ctx = state.build_prompt_context(&project_path, user_input, overrides)?;

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
pub fn list_saved_recipes(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let project_path = get_locked_project_path(&state)?;
    let prompts_dir = project_path.join(".ralph").join("prompts");

    if !prompts_dir.exists() {
        return Ok(vec![]);
    }

    let mut names = Vec::new();
    let entries =
        std::fs::read_dir(&prompts_dir).map_err(|e| format!("Failed to read prompts dir: {e}"))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                names.push(stem.to_owned());
            }
        }
    }

    names.sort();
    Ok(names)
}

#[tauri::command]
pub fn load_saved_recipe(state: State<'_, AppState>, name: String) -> Result<CustomRecipe, String> {
    let project_path = get_locked_project_path(&state)?;
    let file_path = project_path
        .join(".ralph")
        .join("prompts")
        .join(format!("{name}.json"));

    let content =
        std::fs::read_to_string(&file_path).map_err(|e| format!("Failed to read recipe: {e}"))?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse recipe: {e}"))
}

#[tauri::command]
pub fn save_recipe(state: State<'_, AppState>, recipe: CustomRecipe) -> Result<(), String> {
    let project_path = get_locked_project_path(&state)?;
    let prompts_dir = project_path.join(".ralph").join("prompts");
    std::fs::create_dir_all(&prompts_dir)
        .map_err(|e| format!("Failed to create prompts dir: {e}"))?;

    let file_path = prompts_dir.join(format!("{}.json", recipe.name));
    let content =
        serde_json::to_string_pretty(&recipe).map_err(|e| format!("Failed to serialize: {e}"))?;

    std::fs::write(&file_path, content).map_err(|e| format!("Failed to write recipe: {e}"))
}

#[tauri::command]
pub fn delete_recipe(state: State<'_, AppState>, name: String) -> Result<(), String> {
    let project_path = get_locked_project_path(&state)?;
    let file_path = project_path
        .join(".ralph")
        .join("prompts")
        .join(format!("{name}.json"));

    match std::fs::remove_file(&file_path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("Failed to delete recipe: {e}")),
    }
}
