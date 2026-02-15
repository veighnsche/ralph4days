use super::remote_proxy::{remote_invoke_args, remote_invoke_no_args};
use super::state::{AppState, CommandContext};
use ralph_backend::prompt_builder_preview::{
    PromptBuilderPreviewArgs, PromptBuilderPreviewDeps, PromptPreview,
};
use ralph_errors::{codes, ToStringErr};
use ralph_macros::ipc_type;
use serde::Deserialize;
use sqlite_db::{PromptBuilderConfigData, PromptBuilderConfigInput};
use tauri::State;

#[tauri::command]
pub async fn prompt_builder_preview(
    state: State<'_, AppState>,
    args: PromptBuilderPreviewArgs,
) -> Result<PromptPreview, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "prompt_builder_preview", args).await;
    }

    let ctx = CommandContext::from_tauri_state(&state);
    let project_path = ctx.locked_project_path()?;

    let api_port = *state
        .inner()
        .api_server_port
        .lock()
        .err_str(codes::INTERNAL)?;

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
pub async fn prompt_builder_config_list(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_no_args(&rpc, "prompt_builder_config_list").await;
    }

    CommandContext::from_tauri_state(&state).db(sqlite_db::SqliteDb::list_prompt_builder_configs)
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptBuilderConfigGetArgs {
    pub name: String,
}

#[tauri::command]
pub async fn prompt_builder_config_get(
    state: State<'_, AppState>,
    args: PromptBuilderConfigGetArgs,
) -> Result<Option<PromptBuilderConfigData>, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "prompt_builder_config_get", args).await;
    }

    CommandContext::from_tauri_state(&state).db(|db| db.get_prompt_builder_config(&args.name))
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptBuilderConfigSaveArgs {
    pub config: PromptBuilderConfigInput,
}

#[tauri::command]
pub async fn prompt_builder_config_save(
    state: State<'_, AppState>,
    args: PromptBuilderConfigSaveArgs,
) -> Result<(), String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "prompt_builder_config_save", args).await;
    }

    CommandContext::from_tauri_state(&state).db(|db| db.save_prompt_builder_config(args.config))
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptBuilderConfigDeleteArgs {
    pub name: String,
}

#[tauri::command]
pub async fn prompt_builder_config_delete(
    state: State<'_, AppState>,
    args: PromptBuilderConfigDeleteArgs,
) -> Result<(), String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "prompt_builder_config_delete", args).await;
    }

    CommandContext::from_tauri_state(&state).db(|db| db.delete_prompt_builder_config(&args.name))
}
