use super::state::{get_db, get_locked_project_path, AppState};
use ralph_errors::{codes, ralph_err, RalphResultExt};
use ralph_macros::ipc_type;
use serde::Deserialize;
use tauri::State;

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerConfigData {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: std::collections::HashMap<String, String>,
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CropBoxData {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisciplineCropsData {
    pub face: CropBoxData,
    pub card: CropBoxData,
    pub upperbody: Option<CropBoxData>,
    pub portrait: Option<CropBoxData>,
    pub landscape: Option<CropBoxData>,
    pub strip: Option<CropBoxData>,
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisciplineImagePromptData {
    pub positive: String,
    pub negative: String,
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisciplineConfig {
    pub name: String,
    pub display_name: String,
    pub icon: String,
    pub color: String,
    pub acronym: String,
    pub description: Option<String>,
    pub system_prompt: Option<String>,
    pub skills: Vec<String>,
    pub conventions: Option<String>,
    pub mcp_servers: Vec<McpServerConfigData>,
    pub stack_id: Option<u8>,
    pub image_path: Option<String>,
    pub crops: Option<DisciplineCropsData>,
    pub image_prompt: Option<DisciplineImagePromptData>,
}

#[tauri::command]
pub fn get_disciplines_config(state: State<'_, AppState>) -> Result<Vec<DisciplineConfig>, String> {
    let db = get_db(&state)?;
    Ok(db
        .get_disciplines()
        .iter()
        .map(|d| DisciplineConfig {
            name: d.name.clone(),
            display_name: d.display_name.clone(),
            icon: d.icon.clone(),
            color: d.color.clone(),
            acronym: d.acronym.clone(),
            description: d.description.clone(),
            system_prompt: d.system_prompt.clone(),
            skills: d.skills.clone(),
            conventions: d.conventions.clone(),
            mcp_servers: d
                .mcp_servers
                .iter()
                .map(|m| McpServerConfigData {
                    name: m.name.clone(),
                    command: m.command.clone(),
                    args: m.args.clone(),
                    env: m.env.clone(),
                })
                .collect(),
            stack_id: d.stack_id,
            image_path: d.image_path.clone(),
            crops: d
                .crops
                .as_deref()
                .and_then(|s| serde_json::from_str::<DisciplineCropsData>(s).ok()),
            image_prompt: d
                .image_prompt
                .as_deref()
                .and_then(|s| serde_json::from_str::<DisciplineImagePromptData>(s).ok()),
        })
        .collect())
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureLearningData {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iteration: Option<u32>,
    pub created: String,
    pub hit_count: u32,
    pub reviewed: bool,
    pub review_count: u32,
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureData {
    pub name: String,
    pub display_name: String,
    pub acronym: String,
    pub description: Option<String>,
    pub created: Option<String>,
    pub knowledge_paths: Vec<String>,
    pub context_files: Vec<String>,
    pub architecture: Option<String>,
    pub boundaries: Option<String>,
    pub learnings: Vec<FeatureLearningData>,
    pub dependencies: Vec<String>,
}

fn learning_source_str(source: sqlite_db::LearningSource) -> String {
    match source {
        sqlite_db::LearningSource::Auto => "auto".into(),
        sqlite_db::LearningSource::Agent => "agent".into(),
        sqlite_db::LearningSource::Human => "human".into(),
        sqlite_db::LearningSource::OpusReviewed => "opus_reviewed".into(),
    }
}

fn to_learning_data(l: &sqlite_db::FeatureLearning) -> FeatureLearningData {
    FeatureLearningData {
        text: l.text.clone(),
        reason: l.reason.clone(),
        source: learning_source_str(l.source),
        task_id: l.task_id,
        iteration: l.iteration,
        created: l.created.clone(),
        hit_count: l.hit_count,
        reviewed: l.reviewed,
        review_count: l.review_count,
    }
}

#[tauri::command]
pub fn get_features(state: State<'_, AppState>) -> Result<Vec<FeatureData>, String> {
    let db = get_db(&state)?;
    Ok(db
        .get_features()
        .iter()
        .map(|f| FeatureData {
            name: f.name.clone(),
            display_name: f.display_name.clone(),
            acronym: f.acronym.clone(),
            description: f.description.clone(),
            created: f.created.clone(),
            knowledge_paths: f.knowledge_paths.clone(),
            context_files: f.context_files.clone(),
            architecture: f.architecture.clone(),
            boundaries: f.boundaries.clone(),
            learnings: f.learnings.iter().map(to_learning_data).collect(),
            dependencies: f.dependencies.clone(),
        })
        .collect())
}

#[derive(Deserialize)]
pub struct CreateFeatureParams {
    pub name: String,
    pub display_name: String,
    pub acronym: String,
    pub description: Option<String>,
    pub architecture: Option<String>,
    pub boundaries: Option<String>,
    pub knowledge_paths: Option<Vec<String>>,
    pub context_files: Option<Vec<String>>,
    pub dependencies: Option<Vec<String>>,
}

#[tauri::command]
pub fn create_feature(
    state: State<'_, AppState>,
    params: CreateFeatureParams,
) -> Result<(), String> {
    let db = get_db(&state)?;

    db.create_feature(sqlite_db::FeatureInput {
        name: params.name,
        display_name: params.display_name,
        acronym: params.acronym,
        description: params.description,
        architecture: params.architecture,
        boundaries: params.boundaries,
        knowledge_paths: params.knowledge_paths.unwrap_or_default(),
        context_files: params.context_files.unwrap_or_default(),
        dependencies: params.dependencies.unwrap_or_default(),
    })
}

#[derive(Deserialize)]
pub struct UpdateFeatureParams {
    pub name: String,
    pub display_name: String,
    pub acronym: String,
    pub description: Option<String>,
    pub architecture: Option<String>,
    pub boundaries: Option<String>,
    pub knowledge_paths: Option<Vec<String>>,
    pub context_files: Option<Vec<String>>,
    pub dependencies: Option<Vec<String>>,
}

#[tauri::command]
pub fn update_feature(
    state: State<'_, AppState>,
    params: UpdateFeatureParams,
) -> Result<(), String> {
    let db = get_db(&state)?;
    db.update_feature(sqlite_db::FeatureInput {
        name: params.name,
        display_name: params.display_name,
        acronym: params.acronym,
        description: params.description,
        architecture: params.architecture,
        boundaries: params.boundaries,
        knowledge_paths: params.knowledge_paths.unwrap_or_default(),
        context_files: params.context_files.unwrap_or_default(),
        dependencies: params.dependencies.unwrap_or_default(),
    })
}

#[derive(Deserialize)]
pub struct CreateDisciplineParams {
    pub name: String,
    pub display_name: String,
    pub acronym: String,
    pub icon: String,
    pub color: String,
    pub system_prompt: Option<String>,
    pub skills: Option<Vec<String>>,
    pub conventions: Option<String>,
    pub mcp_servers: Option<Vec<McpServerConfigData>>,
}

#[tauri::command]
pub fn create_discipline(
    state: State<'_, AppState>,
    params: CreateDisciplineParams,
) -> Result<(), String> {
    let db = get_db(&state)?;

    let normalized_name = params
        .name
        .to_lowercase()
        .trim()
        .replace(char::is_whitespace, "-");

    let skills_json = serde_json::to_string(&params.skills.unwrap_or_default())
        .ralph_err(codes::DISCIPLINE_OPS, "Failed to serialize skills")?;

    let mcp_servers: Vec<sqlite_db::McpServerConfig> = params
        .mcp_servers
        .unwrap_or_default()
        .iter()
        .map(|m| sqlite_db::McpServerConfig {
            name: m.name.clone(),
            command: m.command.clone(),
            args: m.args.clone(),
            env: m.env.clone(),
        })
        .collect();

    let mcp_json = serde_json::to_string(&mcp_servers)
        .ralph_err(codes::DISCIPLINE_OPS, "Failed to serialize mcp_servers")?;

    db.create_discipline(sqlite_db::DisciplineInput {
        name: normalized_name,
        display_name: params.display_name,
        acronym: params.acronym,
        icon: params.icon,
        color: params.color,
        description: None,
        system_prompt: params.system_prompt,
        skills: skills_json,
        conventions: params.conventions,
        mcp_servers: mcp_json,
        image_path: None,
        crops: None,
        image_prompt: None,
    })
}

#[derive(Deserialize)]
pub struct UpdateDisciplineParams {
    pub name: String,
    pub display_name: String,
    pub acronym: String,
    pub icon: String,
    pub color: String,
    pub system_prompt: Option<String>,
    pub skills: Option<Vec<String>>,
    pub conventions: Option<String>,
    pub mcp_servers: Option<Vec<McpServerConfigData>>,
}

#[tauri::command]
pub fn update_discipline(
    state: State<'_, AppState>,
    params: UpdateDisciplineParams,
) -> Result<(), String> {
    let db = get_db(&state)?;

    let skills_json = serde_json::to_string(&params.skills.unwrap_or_default())
        .ralph_err(codes::DISCIPLINE_OPS, "Failed to serialize skills")?;

    let mcp_servers: Vec<sqlite_db::McpServerConfig> = params
        .mcp_servers
        .unwrap_or_default()
        .iter()
        .map(|m| sqlite_db::McpServerConfig {
            name: m.name.clone(),
            command: m.command.clone(),
            args: m.args.clone(),
            env: m.env.clone(),
        })
        .collect();

    let mcp_json = serde_json::to_string(&mcp_servers)
        .ralph_err(codes::DISCIPLINE_OPS, "Failed to serialize mcp_servers")?;

    db.update_discipline(sqlite_db::DisciplineInput {
        name: params.name,
        display_name: params.display_name,
        acronym: params.acronym,
        icon: params.icon,
        color: params.color,
        description: None,
        system_prompt: params.system_prompt,
        skills: skills_json,
        conventions: params.conventions,
        mcp_servers: mcp_json,
        image_path: None,
        crops: None,
        image_prompt: None,
    })
}

#[tauri::command]
pub fn append_feature_learning(
    state: State<'_, AppState>,
    feature_name: String,
    text: String,
    reason: Option<String>,
    source: Option<String>,
    task_id: Option<u32>,
    iteration: Option<u32>,
) -> Result<bool, String> {
    let db = get_db(&state)?;

    let learning = match source.as_deref() {
        Some("human") => sqlite_db::FeatureLearning::from_human(text, reason),
        Some("agent") => sqlite_db::FeatureLearning::from_agent(text, reason, task_id),
        Some("auto") | None => {
            sqlite_db::FeatureLearning::auto_extracted(text, iteration.unwrap_or(0), task_id)
        }
        Some(other) => return ralph_err!(codes::FEATURE_OPS, "Invalid learning source: {other}"),
    };

    db.append_feature_learning(&feature_name, learning, 50)
}

#[tauri::command]
pub fn remove_feature_learning(
    state: State<'_, AppState>,
    feature_name: String,
    index: usize,
) -> Result<(), String> {
    let db = get_db(&state)?;
    db.remove_feature_learning(&feature_name, index)
}

#[tauri::command]
pub fn add_feature_context_file(
    state: State<'_, AppState>,
    feature_name: String,
    file_path: String,
) -> Result<bool, String> {
    let db = get_db(&state)?;
    db.add_feature_context_file(&feature_name, &file_path, 100)
}

#[tauri::command]
pub fn delete_feature(state: State<'_, AppState>, name: String) -> Result<(), String> {
    let db = get_db(&state)?;
    db.delete_feature(name)
}

#[tauri::command]
pub fn delete_discipline(state: State<'_, AppState>, name: String) -> Result<(), String> {
    let db = get_db(&state)?;
    db.delete_discipline(name)
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualIdentityData {
    pub style: String,
    pub theme: String,
    pub tone: String,
    pub references: String,
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StackMetadataData {
    pub stack_id: u8,
    pub name: String,
    pub description: String,
    pub philosophy: String,
    pub visual_identity: VisualIdentityData,
    pub when_to_use: Vec<String>,
    pub discipline_count: u8,
    pub characteristics: Vec<String>,
}

#[tauri::command]
pub fn get_stack_metadata() -> Vec<StackMetadataData> {
    predefined_disciplines::get_all_stack_metadata()
        .iter()
        .map(|m| StackMetadataData {
            stack_id: m.stack_id,
            name: m.name.clone(),
            description: m.description.clone(),
            philosophy: m.philosophy.clone(),
            visual_identity: VisualIdentityData {
                style: m.visual_identity.style.clone(),
                theme: m.visual_identity.theme.clone(),
                tone: m.visual_identity.tone.clone(),
                references: m.visual_identity.references.clone(),
            },
            when_to_use: m.when_to_use.clone(),
            discipline_count: m.discipline_count,
            characteristics: m.characteristics.clone(),
        })
        .collect()
}

#[tauri::command]
pub fn get_discipline_image_data(
    state: State<'_, AppState>,
    name: String,
) -> Result<Option<String>, String> {
    use base64::Engine;

    let db = get_db(&state)?;
    let disciplines = db.get_disciplines();
    let disc = disciplines.iter().find(|d| d.name == name);

    let Some(disc) = disc else {
        return Ok(None);
    };
    let Some(ref image_path) = disc.image_path else {
        return Ok(None);
    };

    let project_path = get_locked_project_path(&state)?;
    let abs_path = project_path.join(".ralph").join(image_path);

    std::fs::read(&abs_path).map_or(Ok(None), |bytes| {
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
        Ok(Some(b64))
    })
}
