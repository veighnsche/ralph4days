use super::remote_proxy::{remote_invoke_args, remote_invoke_no_args};
use super::state::{AppState, CommandContext};
use ralph_backend::prompt_builder_configs_contract::{
    PromptBuilderConfigDeleteArgs, PromptBuilderConfigGetArgs, PromptBuilderConfigSaveArgs,
};
use ralph_backend::prompt_builder_configs_service;
use ralph_backend::prompt_builder_preview::{
    PromptBuilderPreviewArgs, PromptBuilderPreviewDeps, PromptPreview,
};
use ralph_errors::{codes, RalphResult, RalphResultExt};
use sqlite_db::PromptBuilderConfigData;
use tauri::State;

#[tauri::command]
pub async fn prompt_builder_preview(
    state: State<'_, AppState>,
    args: PromptBuilderPreviewArgs,
) -> RalphResult<PromptPreview> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "prompt_builder_preview", args).await;
    }

    let ctx = CommandContext::from_tauri_state(&state);
    let project_path = ctx.locked_project_path()?;

    let api_port = *state
        .inner()
        .api_server_port
        .lock()
        .ralph_err(codes::INTERNAL, "API server port mutex poisoned")?;

    ctx.db(|db| {
        ralph_backend::prompt_builder_preview::prompt_builder_preview(
            PromptBuilderPreviewDeps {
                db,
                project_path: &project_path,
                mcp_dir: &state.inner().mcp_dir,
                codebase_snapshot: &state.inner().codebase_snapshot,
                api_server_port: api_port,
            },
            args,
        )
    })
}

#[tauri::command]
pub async fn prompt_builder_config_list(state: State<'_, AppState>) -> RalphResult<Vec<String>> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_no_args(&rpc, "prompt_builder_config_list").await;
    }

    CommandContext::from_tauri_state(&state)
        .db(prompt_builder_configs_service::prompt_builder_config_list)
}

#[tauri::command]
pub async fn prompt_builder_config_get(
    state: State<'_, AppState>,
    args: PromptBuilderConfigGetArgs,
) -> RalphResult<Option<PromptBuilderConfigData>> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "prompt_builder_config_get", args).await;
    }

    CommandContext::from_tauri_state(&state)
        .db(|db| prompt_builder_configs_service::prompt_builder_config_get(db, args))
}

#[tauri::command]
pub async fn prompt_builder_config_save(
    state: State<'_, AppState>,
    args: PromptBuilderConfigSaveArgs,
) -> RalphResult<()> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "prompt_builder_config_save", args).await;
    }

    CommandContext::from_tauri_state(&state)
        .db(|db| prompt_builder_configs_service::prompt_builder_config_save(db, args))
}

#[tauri::command]
pub async fn prompt_builder_config_delete(
    state: State<'_, AppState>,
    args: PromptBuilderConfigDeleteArgs,
) -> RalphResult<()> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "prompt_builder_config_delete", args).await;
    }

    CommandContext::from_tauri_state(&state)
        .db(|db| prompt_builder_configs_service::prompt_builder_config_delete(db, args))
}
