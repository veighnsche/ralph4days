use super::state::{AppState, CommandContext};
use ralph_errors::codes;
use ralph_macros::ipc_type;
use serde::{Deserialize, Serialize};
use tauri::State;

fn get_task_or_error(db: &sqlite_db::SqliteDb, id: u32) -> Result<sqlite_db::Task, String> {
    db.get_task_by_id(id).ok_or_else(|| {
        ralph_errors::err_string(
            codes::TASK_OPS,
            format!("Task {id} not found after mutation"),
        )
    })
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksCreateArgs {
    pub subsystem: String,
    pub discipline: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<sqlite_db::Priority>,
    pub tags: Vec<String>,
    pub depends_on: Vec<u32>,
    pub acceptance_criteria: Vec<String>,
    pub context_files: Vec<String>,
    pub output_artifacts: Vec<String>,
    pub hints: Option<String>,
    pub estimated_turns: Option<u32>,
    pub provenance: Option<sqlite_db::TaskProvenance>,
    pub agent: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub thinking: Option<bool>,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksUpdateArgs {
    pub id: u32,
    pub subsystem: String,
    pub discipline: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<sqlite_db::Priority>,
    pub tags: Vec<String>,
    pub depends_on: Vec<u32>,
    pub acceptance_criteria: Vec<String>,
    pub context_files: Vec<String>,
    pub output_artifacts: Vec<String>,
    pub hints: Option<String>,
    pub estimated_turns: Option<u32>,
    pub provenance: Option<sqlite_db::TaskProvenance>,
    pub agent: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub thinking: Option<bool>,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksSetStatusArgs {
    pub id: u32,
    pub status: String,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksDeleteArgs {
    pub id: u32,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksGetArgs {
    pub id: u32,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksSignalAddArgs {
    pub task_id: u32,
    pub discipline: Option<String>,
    pub agent_task_id: Option<u32>,
    pub priority: Option<String>,
    pub body: String,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksSignalUpdateArgs {
    pub task_id: u32,
    pub signal_id: u32,
    pub body: String,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksSignalDeleteArgs {
    pub task_id: u32,
    pub signal_id: u32,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksSignalSummariesGetArgs {
    pub task_ids: Vec<u32>,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksAskAnswerArgs {
    pub signal_id: u32,
    pub answer: String,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksCommentReplyAddArgs {
    pub task_id: u32,
    pub parent_comment_id: u32,
    pub priority: Option<String>,
    pub body: String,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksSignalCommentUpdateArgs {
    pub comment_id: u32,
    pub body: String,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksSignalCommentDeleteArgs {
    pub comment_id: u32,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksSignalCommentsListArgs {
    pub signal_id: u32,
}

#[tauri::command]
pub fn tasks_create(state: State<'_, AppState>, args: TasksCreateArgs) -> Result<String, String> {
    let ctx = CommandContext::from_tauri_state(&state);
    let task_input = sqlite_db::TaskInput {
        subsystem: args.subsystem,
        discipline: args.discipline,
        title: args.title,
        description: args.description,
        status: None,
        priority: args.priority,
        tags: args.tags,
        depends_on: args.depends_on,
        acceptance_criteria: Some(args.acceptance_criteria),
        context_files: args.context_files,
        output_artifacts: args.output_artifacts,
        hints: args.hints,
        estimated_turns: args.estimated_turns,
        provenance: args.provenance,
        agent: args.agent,
        model: args.model,
        effort: args.effort,
        thinking: args.thinking,
    };

    let task_id = ctx.db(|db| db.create_task(task_input))?;
    Ok(task_id.to_string())
}

#[tauri::command]
pub fn tasks_update(
    state: State<'_, AppState>,
    args: TasksUpdateArgs,
) -> Result<sqlite_db::Task, String> {
    let ctx = CommandContext::from_tauri_state(&state);
    let task_id = args.id;
    let task_input = sqlite_db::TaskInput {
        subsystem: args.subsystem,
        discipline: args.discipline,
        title: args.title,
        description: args.description,
        status: None,
        priority: args.priority,
        tags: args.tags,
        depends_on: args.depends_on,
        acceptance_criteria: Some(args.acceptance_criteria),
        context_files: args.context_files,
        output_artifacts: args.output_artifacts,
        hints: args.hints,
        estimated_turns: args.estimated_turns,
        provenance: args.provenance,
        agent: args.agent,
        model: args.model,
        effort: args.effort,
        thinking: args.thinking,
    };

    ctx.db(|db| {
        db.update_task(task_id, task_input)?;
        get_task_or_error(db, task_id)
    })
}

#[tauri::command]
pub fn tasks_set_status(
    state: State<'_, AppState>,
    args: TasksSetStatusArgs,
) -> Result<sqlite_db::Task, String> {
    let ctx = CommandContext::from_tauri_state(&state);
    let status = sqlite_db::TaskStatus::parse(&args.status).ok_or_else(|| {
        ralph_errors::err_string(
            codes::TASK_VALIDATION,
            format!("Invalid status: {}", args.status),
        )
    })?;
    ctx.db(|db| {
        db.set_task_status(args.id, status)?;
        get_task_or_error(db, args.id)
    })
}

#[tauri::command]
pub fn tasks_delete(state: State<'_, AppState>, args: TasksDeleteArgs) -> Result<(), String> {
    CommandContext::from_tauri_state(&state).db(|db| db.delete_task(args.id))
}

#[tauri::command]
pub fn tasks_signal_add(
    state: State<'_, AppState>,
    args: TasksSignalAddArgs,
) -> Result<sqlite_db::Task, String> {
    CommandContext::from_tauri_state(&state).db(|db| {
        db.add_signal(
            args.task_id,
            args.discipline,
            args.agent_task_id,
            args.priority,
            args.body,
        )?;
        get_task_or_error(db, args.task_id)
    })
}

#[tauri::command]
pub fn tasks_signal_update(
    state: State<'_, AppState>,
    args: TasksSignalUpdateArgs,
) -> Result<sqlite_db::Task, String> {
    CommandContext::from_tauri_state(&state).db(|db| {
        db.update_signal(args.task_id, args.signal_id, args.body)?;
        get_task_or_error(db, args.task_id)
    })
}

#[tauri::command]
pub fn tasks_signal_delete(
    state: State<'_, AppState>,
    args: TasksSignalDeleteArgs,
) -> Result<sqlite_db::Task, String> {
    CommandContext::from_tauri_state(&state).db(|db| {
        db.delete_signal(args.task_id, args.signal_id)?;
        get_task_or_error(db, args.task_id)
    })
}

#[tauri::command]
pub fn tasks_list(state: State<'_, AppState>) -> Result<Vec<sqlite_db::Task>, String> {
    CommandContext::from_tauri_state(&state).db(|db| Ok(db.get_tasks()))
}

#[tauri::command]
pub fn tasks_get(
    state: State<'_, AppState>,
    args: TasksGetArgs,
) -> Result<sqlite_db::Task, String> {
    CommandContext::from_tauri_state(&state).db(|db| get_task_or_error(db, args.id))
}

#[tauri::command]
pub fn tasks_list_items(
    state: State<'_, AppState>,
) -> Result<Vec<sqlite_db::TaskListItem>, String> {
    CommandContext::from_tauri_state(&state).db(sqlite_db::SqliteDb::get_task_list_items)
}

#[tauri::command]
pub fn tasks_signal_summaries_get(
    state: State<'_, AppState>,
    args: TasksSignalSummariesGetArgs,
) -> Result<std::collections::HashMap<u32, sqlite_db::TaskSignalSummary>, String> {
    CommandContext::from_tauri_state(&state).db(|db| db.get_signal_summaries(&args.task_ids))
}

#[tauri::command]
pub fn tasks_ask_answer(
    state: State<'_, AppState>,
    args: TasksAskAnswerArgs,
) -> Result<(), String> {
    CommandContext::from_tauri_state(&state).db(|db| db.answer_ask(args.signal_id, args.answer))
}

#[tauri::command]
pub fn tasks_comment_reply_add(
    state: State<'_, AppState>,
    args: TasksCommentReplyAddArgs,
) -> Result<sqlite_db::Task, String> {
    CommandContext::from_tauri_state(&state).db(|db| {
        db.add_signal_with_parent(
            args.task_id,
            None,
            args.priority,
            args.body,
            Some(args.parent_comment_id),
        )?;
        get_task_or_error(db, args.task_id)
    })
}

#[tauri::command]
pub fn tasks_signal_comment_add(
    state: State<'_, AppState>,
    args: sqlite_db::TaskSignalCommentCreateInput,
) -> Result<u32, String> {
    CommandContext::from_tauri_state(&state).db(|db| db.add_task_signal_comment(args))
}

#[tauri::command]
pub fn tasks_signal_comment_update(
    state: State<'_, AppState>,
    args: TasksSignalCommentUpdateArgs,
) -> Result<(), String> {
    CommandContext::from_tauri_state(&state)
        .db(|db| db.update_task_signal_comment(args.comment_id, args.body))
}

#[tauri::command]
pub fn tasks_signal_comment_delete(
    state: State<'_, AppState>,
    args: TasksSignalCommentDeleteArgs,
) -> Result<(), String> {
    CommandContext::from_tauri_state(&state).db(|db| db.delete_task_signal_comment(args.comment_id))
}

#[tauri::command]
pub fn tasks_signal_comments_list(
    state: State<'_, AppState>,
    args: TasksSignalCommentsListArgs,
) -> Result<Vec<sqlite_db::TaskSignalComment>, String> {
    CommandContext::from_tauri_state(&state)
        .db(|db| Ok(db.get_task_signal_comments(args.signal_id)))
}
