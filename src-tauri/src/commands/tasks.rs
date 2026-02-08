use super::state::{get_db, normalize_feature_name, parse_priority, parse_provenance, AppState};
use crate::errors::{codes, ralph_err};
use serde::Deserialize;
use tauri::State;

#[derive(Deserialize)]
pub struct CreateTaskParams {
    pub feature: String,
    pub discipline: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub tags: Vec<String>,
    pub depends_on: Option<Vec<u32>>,
    pub acceptance_criteria: Option<Vec<String>>,
    pub context_files: Option<Vec<String>>,
    pub output_artifacts: Option<Vec<String>>,
    pub hints: Option<String>,
    pub estimated_turns: Option<u32>,
    pub provenance: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateTaskParams {
    pub id: u32,
    pub feature: String,
    pub discipline: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub tags: Vec<String>,
    pub depends_on: Option<Vec<u32>>,
    pub acceptance_criteria: Option<Vec<String>>,
    pub context_files: Option<Vec<String>>,
    pub output_artifacts: Option<Vec<String>>,
    pub hints: Option<String>,
    pub estimated_turns: Option<u32>,
    pub provenance: Option<String>,
}

#[tauri::command]
pub fn create_task(state: State<'_, AppState>, params: CreateTaskParams) -> Result<String, String> {
    let db = get_db(&state)?;

    let task_input = sqlite_db::TaskInput {
        feature: normalize_feature_name(&params.feature)?,
        discipline: params.discipline,
        title: params.title,
        description: params.description,
        priority: parse_priority(params.priority.as_deref()),
        tags: params.tags,
        depends_on: params.depends_on.unwrap_or_default(),
        acceptance_criteria: params.acceptance_criteria,
        context_files: params.context_files.unwrap_or_default(),
        output_artifacts: params.output_artifacts.unwrap_or_default(),
        hints: params.hints,
        estimated_turns: params.estimated_turns,
        provenance: parse_provenance(params.provenance.as_deref()),
    };

    let task_id = db.create_task(task_input)?;
    Ok(task_id.to_string())
}

#[tauri::command]
pub fn update_task(state: State<'_, AppState>, params: UpdateTaskParams) -> Result<(), String> {
    let db = get_db(&state)?;

    let task_input = sqlite_db::TaskInput {
        feature: normalize_feature_name(&params.feature)?,
        discipline: params.discipline,
        title: params.title,
        description: params.description,
        priority: parse_priority(params.priority.as_deref()),
        tags: params.tags,
        depends_on: params.depends_on.unwrap_or_default(),
        acceptance_criteria: params.acceptance_criteria,
        context_files: params.context_files.unwrap_or_default(),
        output_artifacts: params.output_artifacts.unwrap_or_default(),
        hints: params.hints,
        estimated_turns: params.estimated_turns,
        provenance: parse_provenance(params.provenance.as_deref()),
    };

    db.update_task(params.id, task_input)
}

#[tauri::command]
pub fn set_task_status(state: State<'_, AppState>, id: u32, status: String) -> Result<(), String> {
    let db = get_db(&state)?;
    let status = sqlite_db::TaskStatus::parse(&status).ok_or_else(|| {
        crate::errors::RalphError {
            code: codes::TASK_VALIDATION,
            message: format!("Invalid status: {status}"),
        }
        .to_string()
    })?;
    db.set_task_status(id, status)
}

#[tauri::command]
pub fn delete_task(state: State<'_, AppState>, id: u32) -> Result<(), String> {
    let db = get_db(&state)?;
    db.delete_task(id)
}

#[tauri::command]
pub fn add_task_comment(
    state: State<'_, AppState>,
    task_id: u32,
    author: String,
    agent_task_id: Option<u32>,
    body: String,
) -> Result<(), String> {
    let db = get_db(&state)?;
    let comment_author = match author.as_str() {
        "human" => sqlite_db::CommentAuthor::Human,
        "agent" => sqlite_db::CommentAuthor::Agent,
        _ => return ralph_err!(codes::COMMENT_OPS, "Invalid author: {author}"),
    };
    db.add_comment(task_id, comment_author, agent_task_id, body)
}

#[tauri::command]
pub fn update_task_comment(
    state: State<'_, AppState>,
    task_id: u32,
    comment_id: u32,
    body: String,
) -> Result<(), String> {
    let db = get_db(&state)?;
    db.update_comment(task_id, comment_id, body)
}

#[tauri::command]
pub fn delete_task_comment(
    state: State<'_, AppState>,
    task_id: u32,
    comment_id: u32,
) -> Result<(), String> {
    let db = get_db(&state)?;
    db.delete_comment(task_id, comment_id)
}

#[tauri::command]
pub fn get_tasks(state: State<'_, AppState>) -> Result<Vec<sqlite_db::Task>, String> {
    let db = get_db(&state)?;
    Ok(db.get_tasks())
}

#[tauri::command]
pub fn get_feature_stats(state: State<'_, AppState>) -> Result<Vec<sqlite_db::GroupStats>, String> {
    let db = get_db(&state)?;
    Ok(db.get_feature_stats())
}

#[tauri::command]
pub fn get_discipline_stats(
    state: State<'_, AppState>,
) -> Result<Vec<sqlite_db::GroupStats>, String> {
    let db = get_db(&state)?;
    Ok(db.get_discipline_stats())
}

#[tauri::command]
pub fn get_project_progress(
    state: State<'_, AppState>,
) -> Result<sqlite_db::ProjectProgress, String> {
    let db = get_db(&state)?;
    Ok(db.get_project_progress())
}

#[tauri::command]
pub fn get_all_tags(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let db = get_db(&state)?;
    Ok(db.get_all_tags())
}
