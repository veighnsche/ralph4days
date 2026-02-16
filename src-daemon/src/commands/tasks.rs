use ralph_errors::RalphResult;

use crate::rpc_codec::{decode_args, encode_result, require_null_payload};
use crate::state::AppState;

pub fn tasks_create(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: ralph_backend::tasks::TasksCreateArgs = decode_args("tasks_create", payload)?;
    let created = ralph_backend::session::with_db(&state.db, |db| {
        ralph_backend::tasks::tasks_create(db, args)
    })?;
    encode_result("tasks_create", created)
}

pub fn tasks_update(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: ralph_backend::tasks::TasksUpdateArgs = decode_args("tasks_update", payload)?;
    let updated = ralph_backend::session::with_db(&state.db, |db| {
        ralph_backend::tasks::tasks_update(db, args)
    })?;
    encode_result("tasks_update", updated)
}

pub fn tasks_set_status(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: ralph_backend::tasks::TasksSetStatusArgs = decode_args("tasks_set_status", payload)?;
    let updated = ralph_backend::session::with_db(&state.db, |db| {
        ralph_backend::tasks::tasks_set_status(db, args)
    })?;
    encode_result("tasks_set_status", updated)
}

pub fn tasks_delete(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: ralph_backend::tasks::TasksDeleteArgs = decode_args("tasks_delete", payload)?;
    ralph_backend::session::with_db(&state.db, |db| ralph_backend::tasks::tasks_delete(db, args))?;
    Ok(serde_json::Value::Null)
}

pub fn tasks_signal_add(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: ralph_backend::tasks::TasksSignalAddArgs = decode_args("tasks_signal_add", payload)?;
    let updated = ralph_backend::session::with_db(&state.db, |db| {
        ralph_backend::tasks::tasks_signal_add(db, args)
    })?;
    encode_result("tasks_signal_add", updated)
}

pub fn tasks_signal_update(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: ralph_backend::tasks::TasksSignalUpdateArgs =
        decode_args("tasks_signal_update", payload)?;
    let updated = ralph_backend::session::with_db(&state.db, |db| {
        ralph_backend::tasks::tasks_signal_update(db, args)
    })?;
    encode_result("tasks_signal_update", updated)
}

pub fn tasks_signal_delete(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: ralph_backend::tasks::TasksSignalDeleteArgs =
        decode_args("tasks_signal_delete", payload)?;
    let updated = ralph_backend::session::with_db(&state.db, |db| {
        ralph_backend::tasks::tasks_signal_delete(db, args)
    })?;
    encode_result("tasks_signal_delete", updated)
}

pub fn tasks_list(state: &AppState, payload: serde_json::Value) -> RalphResult<serde_json::Value> {
    require_null_payload("tasks_list", payload)?;
    let tasks = ralph_backend::session::with_db(&state.db, ralph_backend::tasks::tasks_list)?;
    encode_result("tasks_list", tasks)
}

pub fn tasks_get(state: &AppState, payload: serde_json::Value) -> RalphResult<serde_json::Value> {
    let args: ralph_backend::tasks::TasksGetArgs = decode_args("tasks_get", payload)?;
    let task =
        ralph_backend::session::with_db(&state.db, |db| ralph_backend::tasks::tasks_get(db, args))?;
    encode_result("tasks_get", task)
}

pub fn tasks_list_items(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    require_null_payload("tasks_list_items", payload)?;
    let items = ralph_backend::session::with_db(&state.db, ralph_backend::tasks::tasks_list_items)?;
    encode_result("tasks_list_items", items)
}

pub fn tasks_signal_summaries_get(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: ralph_backend::tasks::TasksSignalSummariesGetArgs =
        decode_args("tasks_signal_summaries_get", payload)?;
    let summaries = ralph_backend::session::with_db(&state.db, |db| {
        ralph_backend::tasks::tasks_signal_summaries_get(db, args)
    })?;
    encode_result("tasks_signal_summaries_get", summaries)
}

pub fn tasks_ask_answer(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: ralph_backend::tasks::TasksAskAnswerArgs = decode_args("tasks_ask_answer", payload)?;
    ralph_backend::session::with_db(&state.db, |db| {
        ralph_backend::tasks::tasks_ask_answer(db, args)
    })?;
    Ok(serde_json::Value::Null)
}

pub fn tasks_comment_reply_add(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: ralph_backend::tasks::TasksCommentReplyAddArgs =
        decode_args("tasks_comment_reply_add", payload)?;
    let updated = ralph_backend::session::with_db(&state.db, |db| {
        ralph_backend::tasks::tasks_comment_reply_add(db, args)
    })?;
    encode_result("tasks_comment_reply_add", updated)
}

pub fn tasks_signal_comment_add(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: sqlite_db::TaskSignalCommentCreateInput =
        decode_args("tasks_signal_comment_add", payload)?;
    let id = ralph_backend::session::with_db(&state.db, |db| {
        ralph_backend::tasks::tasks_signal_comment_add(db, args)
    })?;
    encode_result("tasks_signal_comment_add", id)
}

pub fn tasks_signal_comment_update(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: ralph_backend::tasks::TasksSignalCommentUpdateArgs =
        decode_args("tasks_signal_comment_update", payload)?;
    ralph_backend::session::with_db(&state.db, |db| {
        ralph_backend::tasks::tasks_signal_comment_update(db, args)
    })?;
    Ok(serde_json::Value::Null)
}

pub fn tasks_signal_comment_delete(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: ralph_backend::tasks::TasksSignalCommentDeleteArgs =
        decode_args("tasks_signal_comment_delete", payload)?;
    ralph_backend::session::with_db(&state.db, |db| {
        ralph_backend::tasks::tasks_signal_comment_delete(db, args)
    })?;
    Ok(serde_json::Value::Null)
}

pub fn tasks_signal_comments_list(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: ralph_backend::tasks::TasksSignalCommentsListArgs =
        decode_args("tasks_signal_comments_list", payload)?;
    let comments = ralph_backend::session::with_db(&state.db, |db| {
        ralph_backend::tasks::tasks_signal_comments_list(db, args)
    })?;
    encode_result("tasks_signal_comments_list", comments)
}
