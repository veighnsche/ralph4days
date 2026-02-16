use ralph_backend::prompt_builder_configs_contract::{
    PromptBuilderConfigDeleteArgs, PromptBuilderConfigGetArgs, PromptBuilderConfigSaveArgs,
};
use ralph_backend::prompt_builder_configs_service;
use ralph_backend::prompt_builder_preview::{PromptBuilderPreviewArgs, PromptBuilderPreviewDeps};
use ralph_errors::{codes, RalphResult, RalphResultExt};

use crate::rpc_codec::{decode_args, encode_result, require_null_payload};
use crate::state::AppState;

pub fn prompt_builder_config_list(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    require_null_payload("prompt_builder_config_list", payload)?;
    let names = ralph_backend::session::with_db(&state.db, |db| {
        prompt_builder_configs_service::prompt_builder_config_list(db)
    })?;
    encode_result("prompt_builder_config_list", names)
}

pub fn prompt_builder_config_get(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: PromptBuilderConfigGetArgs = decode_args("prompt_builder_config_get", payload)?;
    let config = ralph_backend::session::with_db(&state.db, |db| {
        prompt_builder_configs_service::prompt_builder_config_get(db, args)
    })?;
    encode_result("prompt_builder_config_get", config)
}

pub fn prompt_builder_config_save(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: PromptBuilderConfigSaveArgs = decode_args("prompt_builder_config_save", payload)?;
    ralph_backend::session::with_db(&state.db, |db| {
        prompt_builder_configs_service::prompt_builder_config_save(db, args)
    })?;
    Ok(serde_json::Value::Null)
}

pub fn prompt_builder_config_delete(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: PromptBuilderConfigDeleteArgs = decode_args("prompt_builder_config_delete", payload)?;
    ralph_backend::session::with_db(&state.db, |db| {
        prompt_builder_configs_service::prompt_builder_config_delete(db, args)
    })?;
    Ok(serde_json::Value::Null)
}

pub fn prompt_builder_preview(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: PromptBuilderPreviewArgs = decode_args("prompt_builder_preview", payload)?;
    let project_path = ralph_backend::session::locked_project_path(&state.locked_project)?;
    let api_server_port = *state
        .api_server_port
        .lock()
        .ralph_err(codes::INTERNAL, "API server port mutex poisoned")?;

    let preview = ralph_backend::session::with_db(&state.db, |db| {
        ralph_backend::prompt_builder_preview::prompt_builder_preview(
            PromptBuilderPreviewDeps {
                db,
                project_path: project_path.as_path(),
                mcp_dir: state.mcp_dir.as_path(),
                codebase_snapshot: &state.codebase_snapshot,
                api_server_port,
            },
            args,
        )
    })?;

    encode_result("prompt_builder_preview", preview)
}
