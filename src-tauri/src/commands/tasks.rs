use super::state::{AppState, CommandContext};
use ralph_errors::codes;
use serde::Deserialize;
use tauri::State;

fn get_task_or_error(db: &sqlite_db::SqliteDb, id: u32) -> Result<sqlite_db::Task, String> {
    db.get_task_by_id(id).ok_or_else(|| {
        ralph_errors::err_string(
            codes::TASK_OPS,
            format!("Task {id} not found after mutation"),
        )
    })
}

#[derive(Deserialize)]
pub struct CreateTaskParams {
    pub subsystem: String,
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
    pub agent: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub thinking: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateTaskParams {
    pub id: u32,
    pub subsystem: String,
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
    pub agent: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub thinking: Option<bool>,
}

#[tauri::command]
pub fn create_task(state: State<'_, AppState>, params: CreateTaskParams) -> Result<String, String> {
    let ctx = CommandContext::from_tauri_state(&state);
    let task_input = sqlite_db::TaskInput {
        subsystem: params.subsystem,
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
        agent: params.agent,
        model: params.model,
        effort: params.effort,
        thinking: params.thinking,
    };

    let task_id = ctx.db(|db| db.create_task(task_input))?;
    Ok(task_id.to_string())
}

#[tauri::command]
pub fn update_task(
    state: State<'_, AppState>,
    params: UpdateTaskParams,
) -> Result<sqlite_db::Task, String> {
    let ctx = CommandContext::from_tauri_state(&state);
    let task_id = params.id;
    let task_input = sqlite_db::TaskInput {
        subsystem: params.subsystem,
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
        agent: params.agent,
        model: params.model,
        effort: params.effort,
        thinking: params.thinking,
    };

    ctx.db(|db| {
        db.update_task(task_id, task_input)?;
        get_task_or_error(db, task_id)
    })
}

#[tauri::command]
pub fn set_task_status(
    state: State<'_, AppState>,
    id: u32,
    status: String,
) -> Result<sqlite_db::Task, String> {
    let ctx = CommandContext::from_tauri_state(&state);
    let status = sqlite_db::TaskStatus::parse(&status).ok_or_else(|| {
        ralph_errors::err_string(codes::TASK_VALIDATION, format!("Invalid status: {status}"))
    })?;
    ctx.db(|db| {
        db.set_task_status(id, status)?;
        get_task_or_error(db, id)
    })
}

#[tauri::command]
pub fn delete_task(state: State<'_, AppState>, id: u32) -> Result<(), String> {
    CommandContext::from_tauri_state(&state).db(|db| db.delete_task(id))
}

#[tauri::command]
pub fn add_task_signal(
    state: State<'_, AppState>,
    task_id: u32,
    discipline: Option<String>,
    agent_task_id: Option<u32>,
    priority: Option<String>,
    body: String,
) -> Result<sqlite_db::Task, String> {
    CommandContext::from_tauri_state(&state).db(|db| {
        db.add_signal(task_id, discipline, agent_task_id, priority, body)?;
        get_task_or_error(db, task_id)
    })
}

#[tauri::command]
pub fn update_task_signal(
    state: State<'_, AppState>,
    task_id: u32,
    signal_id: u32,
    body: String,
) -> Result<sqlite_db::Task, String> {
    CommandContext::from_tauri_state(&state).db(|db| {
        db.update_signal(task_id, signal_id, body)?;
        get_task_or_error(db, task_id)
    })
}

#[tauri::command]
pub fn delete_task_signal(
    state: State<'_, AppState>,
    task_id: u32,
    signal_id: u32,
) -> Result<sqlite_db::Task, String> {
    CommandContext::from_tauri_state(&state).db(|db| {
        db.delete_signal(task_id, signal_id)?;
        get_task_or_error(db, task_id)
    })
}

#[tauri::command]
pub fn get_tasks(state: State<'_, AppState>) -> Result<Vec<sqlite_db::Task>, String> {
    CommandContext::from_tauri_state(&state).db(|db| Ok(db.get_tasks()))
}

#[tauri::command]
pub fn get_signal_summaries(
    state: State<'_, AppState>,
    task_ids: Vec<u32>,
) -> Result<std::collections::HashMap<u32, sqlite_db::TaskSignalSummary>, String> {
    CommandContext::from_tauri_state(&state).db(|db| db.get_signal_summaries(&task_ids))
}

#[tauri::command]
pub fn answer_ask(
    state: State<'_, AppState>,
    signal_id: u32,
    answer: String,
) -> Result<(), String> {
    CommandContext::from_tauri_state(&state).db(|db| db.answer_ask(signal_id, answer))
}

#[tauri::command]
pub fn add_reply_to_comment(
    state: State<'_, AppState>,
    task_id: u32,
    parent_comment_id: u32,
    priority: Option<String>,
    body: String,
) -> Result<sqlite_db::Task, String> {
    CommandContext::from_tauri_state(&state).db(|db| {
        db.add_signal_with_parent(task_id, None, priority, body, Some(parent_comment_id))?;
        get_task_or_error(db, task_id)
    })
}

#[tauri::command]
pub fn add_task_signal_comment(
    state: State<'_, AppState>,
    params: sqlite_db::TaskSignalCommentCreateInput,
) -> Result<u32, String> {
    CommandContext::from_tauri_state(&state).db(|db| db.add_task_signal_comment(params))
}

#[tauri::command]
pub fn update_task_signal_comment(
    state: State<'_, AppState>,
    comment_id: u32,
    body: String,
) -> Result<(), String> {
    CommandContext::from_tauri_state(&state)
        .db(|db| db.update_task_signal_comment(comment_id, body))
}

#[tauri::command]
pub fn delete_task_signal_comment(
    state: State<'_, AppState>,
    comment_id: u32,
) -> Result<(), String> {
    CommandContext::from_tauri_state(&state).db(|db| db.delete_task_signal_comment(comment_id))
}

#[tauri::command]
pub fn get_task_signal_comments(
    state: State<'_, AppState>,
    signal_id: u32,
) -> Result<Vec<sqlite_db::TaskSignalComment>, String> {
    CommandContext::from_tauri_state(&state).db(|db| Ok(db.get_task_signal_comments(signal_id)))
}
