use super::remote_proxy::{remote_invoke_args, remote_invoke_no_args};
use super::state::{AppState, CommandContext};
use ralph_backend::tasks::{
    TasksAskAnswerArgs, TasksCommentReplyAddArgs, TasksCreateArgs, TasksDeleteArgs, TasksGetArgs,
    TasksSetStatusArgs, TasksSignalAddArgs, TasksSignalCommentDeleteArgs,
    TasksSignalCommentUpdateArgs, TasksSignalCommentsListArgs, TasksSignalDeleteArgs,
    TasksSignalSummariesGetArgs, TasksSignalUpdateArgs, TasksUpdateArgs,
};
use tauri::State;

#[tauri::command]
pub async fn tasks_create(
    state: State<'_, AppState>,
    args: TasksCreateArgs,
) -> Result<String, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "tasks_create", args).await;
    }

    CommandContext::from_tauri_state(&state).db(|db| ralph_backend::tasks::tasks_create(db, args))
}

#[tauri::command]
pub async fn tasks_update(
    state: State<'_, AppState>,
    args: TasksUpdateArgs,
) -> Result<sqlite_db::Task, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "tasks_update", args).await;
    }

    CommandContext::from_tauri_state(&state).db(|db| ralph_backend::tasks::tasks_update(db, args))
}

#[tauri::command]
pub async fn tasks_set_status(
    state: State<'_, AppState>,
    args: TasksSetStatusArgs,
) -> Result<sqlite_db::Task, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "tasks_set_status", args).await;
    }

    CommandContext::from_tauri_state(&state)
        .db(|db| ralph_backend::tasks::tasks_set_status(db, args))
}

#[tauri::command]
pub async fn tasks_delete(state: State<'_, AppState>, args: TasksDeleteArgs) -> Result<(), String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "tasks_delete", args).await;
    }

    CommandContext::from_tauri_state(&state).db(|db| ralph_backend::tasks::tasks_delete(db, args))
}

#[tauri::command]
pub async fn tasks_signal_add(
    state: State<'_, AppState>,
    args: TasksSignalAddArgs,
) -> Result<sqlite_db::Task, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "tasks_signal_add", args).await;
    }

    CommandContext::from_tauri_state(&state)
        .db(|db| ralph_backend::tasks::tasks_signal_add(db, args))
}

#[tauri::command]
pub async fn tasks_signal_update(
    state: State<'_, AppState>,
    args: TasksSignalUpdateArgs,
) -> Result<sqlite_db::Task, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "tasks_signal_update", args).await;
    }

    CommandContext::from_tauri_state(&state)
        .db(|db| ralph_backend::tasks::tasks_signal_update(db, args))
}

#[tauri::command]
pub async fn tasks_signal_delete(
    state: State<'_, AppState>,
    args: TasksSignalDeleteArgs,
) -> Result<sqlite_db::Task, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "tasks_signal_delete", args).await;
    }

    CommandContext::from_tauri_state(&state)
        .db(|db| ralph_backend::tasks::tasks_signal_delete(db, args))
}

#[tauri::command]
pub async fn tasks_list(state: State<'_, AppState>) -> Result<Vec<sqlite_db::Task>, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_no_args(&rpc, "tasks_list").await;
    }

    CommandContext::from_tauri_state(&state).db(ralph_backend::tasks::tasks_list)
}

#[tauri::command]
pub async fn tasks_get(
    state: State<'_, AppState>,
    args: TasksGetArgs,
) -> Result<sqlite_db::Task, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "tasks_get", args).await;
    }

    CommandContext::from_tauri_state(&state).db(|db| ralph_backend::tasks::tasks_get(db, args))
}

#[tauri::command]
pub async fn tasks_list_items(
    state: State<'_, AppState>,
) -> Result<Vec<sqlite_db::TaskListItem>, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_no_args(&rpc, "tasks_list_items").await;
    }

    CommandContext::from_tauri_state(&state).db(ralph_backend::tasks::tasks_list_items)
}

#[tauri::command]
pub async fn tasks_signal_summaries_get(
    state: State<'_, AppState>,
    args: TasksSignalSummariesGetArgs,
) -> Result<std::collections::HashMap<u32, sqlite_db::TaskSignalSummary>, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "tasks_signal_summaries_get", args).await;
    }

    CommandContext::from_tauri_state(&state)
        .db(|db| ralph_backend::tasks::tasks_signal_summaries_get(db, args))
}

#[tauri::command]
pub async fn tasks_ask_answer(
    state: State<'_, AppState>,
    args: TasksAskAnswerArgs,
) -> Result<(), String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "tasks_ask_answer", args).await;
    }

    CommandContext::from_tauri_state(&state)
        .db(|db| ralph_backend::tasks::tasks_ask_answer(db, args))
}

#[tauri::command]
pub async fn tasks_comment_reply_add(
    state: State<'_, AppState>,
    args: TasksCommentReplyAddArgs,
) -> Result<sqlite_db::Task, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "tasks_comment_reply_add", args).await;
    }

    CommandContext::from_tauri_state(&state)
        .db(|db| ralph_backend::tasks::tasks_comment_reply_add(db, args))
}

#[tauri::command]
pub async fn tasks_signal_comment_add(
    state: State<'_, AppState>,
    args: sqlite_db::TaskSignalCommentCreateInput,
) -> Result<u32, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "tasks_signal_comment_add", args).await;
    }

    CommandContext::from_tauri_state(&state)
        .db(|db| ralph_backend::tasks::tasks_signal_comment_add(db, args))
}

#[tauri::command]
pub async fn tasks_signal_comment_update(
    state: State<'_, AppState>,
    args: TasksSignalCommentUpdateArgs,
) -> Result<(), String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "tasks_signal_comment_update", args).await;
    }

    CommandContext::from_tauri_state(&state)
        .db(|db| ralph_backend::tasks::tasks_signal_comment_update(db, args))
}

#[tauri::command]
pub async fn tasks_signal_comment_delete(
    state: State<'_, AppState>,
    args: TasksSignalCommentDeleteArgs,
) -> Result<(), String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "tasks_signal_comment_delete", args).await;
    }

    CommandContext::from_tauri_state(&state)
        .db(|db| ralph_backend::tasks::tasks_signal_comment_delete(db, args))
}

#[tauri::command]
pub async fn tasks_signal_comments_list(
    state: State<'_, AppState>,
    args: TasksSignalCommentsListArgs,
) -> Result<Vec<sqlite_db::TaskSignalComment>, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "tasks_signal_comments_list", args).await;
    }

    CommandContext::from_tauri_state(&state)
        .db(|db| ralph_backend::tasks::tasks_signal_comments_list(db, args))
}
