use super::state::{get_db, normalize_feature_name, AppState};
use serde::Deserialize;
use tauri::State;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerConfigData {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisciplineConfig {
    pub name: String,
    pub display_name: String,
    pub icon: String,
    pub color: String,
    pub acronym: String,
    pub system_prompt: Option<String>,
    pub skills: Vec<String>,
    pub conventions: Option<String>,
    pub mcp_servers: Vec<McpServerConfigData>,
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
        })
        .collect())
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureConfig {
    pub name: String,
    pub display_name: String,
    pub acronym: String,
}

#[tauri::command]
pub fn get_features_config(state: State<'_, AppState>) -> Result<Vec<FeatureConfig>, String> {
    let db = get_db(&state)?;
    Ok(db
        .get_features()
        .iter()
        .map(|f| FeatureConfig {
            name: f.name.clone(),
            display_name: f.display_name.clone(),
            acronym: f.acronym.clone(),
        })
        .collect())
}

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

    let normalized_name = normalize_feature_name(&params.name)?;

    db.create_feature(sqlite_db::FeatureInput {
        name: normalized_name,
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

#[tauri::command]
pub fn create_discipline(
    state: State<'_, AppState>,
    name: String,
    display_name: String,
    acronym: String,
    icon: String,
    color: String,
) -> Result<(), String> {
    let db = get_db(&state)?;

    let normalized_name = name.to_lowercase().trim().replace(char::is_whitespace, "-");

    db.create_discipline(normalized_name, display_name, acronym, icon, color)
}

#[tauri::command]
pub fn update_discipline(
    state: State<'_, AppState>,
    name: String,
    display_name: String,
    acronym: String,
    icon: String,
    color: String,
) -> Result<(), String> {
    let db = get_db(&state)?;
    db.update_discipline(name, display_name, acronym, icon, color)
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
        Some(other) => return Err(format!("Invalid learning source: {other}")),
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
