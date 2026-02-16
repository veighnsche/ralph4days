use core_contracts::domain::TaskSignalCommentCreateInput;
use core_contracts::tasks::{
    TasksAskAnswerArgs, TasksCommentReplyAddArgs, TasksCreateArgs, TasksDeleteArgs, TasksGetArgs,
    TasksSetStatusArgs, TasksSignalAddArgs, TasksSignalCommentDeleteArgs,
    TasksSignalCommentUpdateArgs, TasksSignalCommentsListArgs, TasksSignalDeleteArgs,
    TasksSignalSummariesGetArgs, TasksSignalUpdateArgs, TasksUpdateArgs,
};
use core_errors::RalphResult;

use crate::rpc_codec::{decode_args, encode_result, require_null_payload};
use crate::state::AppState;

pub fn tasks_create(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: TasksCreateArgs = decode_args("tasks_create", payload)?;
    let created = service_project::session::with_db(&state.db, |db| {
        service_tasks::tasks::tasks_create(db, args)
    })?;
    encode_result("tasks_create", created)
}

pub fn tasks_update(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: TasksUpdateArgs = decode_args("tasks_update", payload)?;
    let updated = service_project::session::with_db(&state.db, |db| {
        service_tasks::tasks::tasks_update(db, args)
    })?;
    encode_result("tasks_update", updated)
}

pub fn tasks_set_status(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: TasksSetStatusArgs = decode_args("tasks_set_status", payload)?;
    let updated = service_project::session::with_db(&state.db, |db| {
        service_tasks::tasks::tasks_set_status(db, args)
    })?;
    encode_result("tasks_set_status", updated)
}

pub fn tasks_delete(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: TasksDeleteArgs = decode_args("tasks_delete", payload)?;
    service_project::session::with_db(&state.db, |db| service_tasks::tasks::tasks_delete(db, args))?;
    Ok(serde_json::Value::Null)
}

pub fn tasks_signal_add(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: TasksSignalAddArgs = decode_args("tasks_signal_add", payload)?;
    let updated = service_project::session::with_db(&state.db, |db| {
        service_tasks::tasks::tasks_signal_add(db, args)
    })?;
    encode_result("tasks_signal_add", updated)
}

pub fn tasks_signal_update(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: TasksSignalUpdateArgs = decode_args("tasks_signal_update", payload)?;
    let updated = service_project::session::with_db(&state.db, |db| {
        service_tasks::tasks::tasks_signal_update(db, args)
    })?;
    encode_result("tasks_signal_update", updated)
}

pub fn tasks_signal_delete(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: TasksSignalDeleteArgs = decode_args("tasks_signal_delete", payload)?;
    let updated = service_project::session::with_db(&state.db, |db| {
        service_tasks::tasks::tasks_signal_delete(db, args)
    })?;
    encode_result("tasks_signal_delete", updated)
}

pub fn tasks_list(state: &AppState, payload: serde_json::Value) -> RalphResult<serde_json::Value> {
    require_null_payload("tasks_list", payload)?;
    let tasks = service_project::session::with_db(&state.db, service_tasks::tasks::tasks_list)?;
    encode_result("tasks_list", tasks)
}

pub fn tasks_get(state: &AppState, payload: serde_json::Value) -> RalphResult<serde_json::Value> {
    let args: TasksGetArgs = decode_args("tasks_get", payload)?;
    let task =
        service_project::session::with_db(&state.db, |db| service_tasks::tasks::tasks_get(db, args))?;
    encode_result("tasks_get", task)
}

pub fn tasks_list_items(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    require_null_payload("tasks_list_items", payload)?;
    let items = service_project::session::with_db(&state.db, service_tasks::tasks::tasks_list_items)?;
    encode_result("tasks_list_items", items)
}

pub fn tasks_signal_summaries_get(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: TasksSignalSummariesGetArgs = decode_args("tasks_signal_summaries_get", payload)?;
    let summaries = service_project::session::with_db(&state.db, |db| {
        service_tasks::tasks::tasks_signal_summaries_get(db, args)
    })?;
    encode_result("tasks_signal_summaries_get", summaries)
}

pub fn tasks_ask_answer(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: TasksAskAnswerArgs = decode_args("tasks_ask_answer", payload)?;
    service_project::session::with_db(&state.db, |db| {
        service_tasks::tasks::tasks_ask_answer(db, args)
    })?;
    Ok(serde_json::Value::Null)
}

pub fn tasks_comment_reply_add(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: TasksCommentReplyAddArgs = decode_args("tasks_comment_reply_add", payload)?;
    let updated = service_project::session::with_db(&state.db, |db| {
        service_tasks::tasks::tasks_comment_reply_add(db, args)
    })?;
    encode_result("tasks_comment_reply_add", updated)
}

pub fn tasks_signal_comment_add(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: TaskSignalCommentCreateInput = decode_args("tasks_signal_comment_add", payload)?;
    let id = service_project::session::with_db(&state.db, |db| {
        service_tasks::tasks::tasks_signal_comment_add(db, args)
    })?;
    encode_result("tasks_signal_comment_add", id)
}

pub fn tasks_signal_comment_update(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: TasksSignalCommentUpdateArgs = decode_args("tasks_signal_comment_update", payload)?;
    service_project::session::with_db(&state.db, |db| {
        service_tasks::tasks::tasks_signal_comment_update(db, args)
    })?;
    Ok(serde_json::Value::Null)
}

pub fn tasks_signal_comment_delete(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: TasksSignalCommentDeleteArgs = decode_args("tasks_signal_comment_delete", payload)?;
    service_project::session::with_db(&state.db, |db| {
        service_tasks::tasks::tasks_signal_comment_delete(db, args)
    })?;
    Ok(serde_json::Value::Null)
}

pub fn tasks_signal_comments_list(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: TasksSignalCommentsListArgs = decode_args("tasks_signal_comments_list", payload)?;
    let comments = service_project::session::with_db(&state.db, |db| {
        service_tasks::tasks::tasks_signal_comments_list(db, args)
    })?;
    encode_result("tasks_signal_comments_list", comments)
}
