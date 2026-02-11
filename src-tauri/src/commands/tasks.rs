use super::state::{get_db, AppState};
use ralph_errors::codes;
use serde::Deserialize;
use tauri::State;

#[derive(Deserialize)]
pub struct CreateTaskParams {
    pub feature: String,
    pub discipline: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<sqlite_db::Priority>,
    pub tags: Vec<String>,
    pub depends_on: Option<Vec<u32>>,
    pub acceptance_criteria: Option<Vec<String>>,
    pub context_files: Option<Vec<String>>,
    pub output_artifacts: Option<Vec<String>>,
    pub hints: Option<String>,
    pub estimated_turns: Option<u32>,
    pub provenance: Option<sqlite_db::TaskProvenance>,
}

#[derive(Deserialize)]
pub struct UpdateTaskParams {
    pub id: u32,
    pub feature: String,
    pub discipline: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<sqlite_db::Priority>,
    pub tags: Vec<String>,
    pub depends_on: Option<Vec<u32>>,
    pub acceptance_criteria: Option<Vec<String>>,
    pub context_files: Option<Vec<String>>,
    pub output_artifacts: Option<Vec<String>>,
    pub hints: Option<String>,
    pub estimated_turns: Option<u32>,
    pub provenance: Option<sqlite_db::TaskProvenance>,
}

#[tauri::command]
pub fn create_task(state: State<'_, AppState>, params: CreateTaskParams) -> Result<String, String> {
    let db = get_db(&state)?;

    let task_input = sqlite_db::TaskInput {
        feature: params.feature,
        discipline: params.discipline,
        title: params.title,
        description: params.description,
        status: None,
        priority: params.priority,
        tags: params.tags,
        depends_on: params.depends_on.unwrap_or_default(),
        acceptance_criteria: params.acceptance_criteria,
        context_files: params.context_files.unwrap_or_default(),
        output_artifacts: params.output_artifacts.unwrap_or_default(),
        hints: params.hints,
        estimated_turns: params.estimated_turns,
        provenance: params.provenance,
    };

    let task_id = db.create_task(task_input)?;
    Ok(task_id.to_string())
}

#[tauri::command]
pub fn update_task(state: State<'_, AppState>, params: UpdateTaskParams) -> Result<(), String> {
    let db = get_db(&state)?;

    let task_input = sqlite_db::TaskInput {
        feature: params.feature,
        discipline: params.discipline,
        title: params.title,
        description: params.description,
        status: None,
        priority: params.priority,
        tags: params.tags,
        depends_on: params.depends_on.unwrap_or_default(),
        acceptance_criteria: params.acceptance_criteria,
        context_files: params.context_files.unwrap_or_default(),
        output_artifacts: params.output_artifacts.unwrap_or_default(),
        hints: params.hints,
        estimated_turns: params.estimated_turns,
        provenance: params.provenance,
    };

    db.update_task(params.id, task_input)
}

#[tauri::command]
pub fn set_task_status(state: State<'_, AppState>, id: u32, status: String) -> Result<(), String> {
    let db = get_db(&state)?;
    let status = sqlite_db::TaskStatus::parse(&status).ok_or_else(|| {
        ralph_errors::err_string(codes::TASK_VALIDATION, format!("Invalid status: {status}"))
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
    discipline: Option<String>,
    agent_task_id: Option<u32>,
    priority: Option<String>,
    body: String,
) -> Result<(), String> {
    let db = get_db(&state)?;
    db.add_comment(task_id, discipline, agent_task_id, priority, body)
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
pub fn get_task_signals(
    state: State<'_, AppState>,
    task_id: u32,
) -> Result<Vec<sqlite_db::TaskSignal>, String> {
    let db = get_db(&state)?;
    db.get_task_signals(task_id)
}

#[tauri::command]
pub fn get_signal_summaries(
    state: State<'_, AppState>,
    task_ids: Vec<u32>,
) -> Result<std::collections::HashMap<u32, sqlite_db::TaskSignalSummary>, String> {
    let db = get_db(&state)?;
    db.get_signal_summaries(&task_ids)
}

#[tauri::command]
pub fn answer_ask(
    state: State<'_, AppState>,
    signal_id: u32,
    answer: String,
) -> Result<(), String> {
    let db = get_db(&state)?;
    db.answer_ask(signal_id, answer)
}
