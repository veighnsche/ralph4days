use super::state::{AppState, ToStringErr};
use crate::errors::codes;
use crate::terminal::SessionConfig;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn create_pty_session(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    mcp_mode: Option<String>,
    model: Option<String>,
    thinking: Option<bool>,
) -> Result<(), String> {
    let locked = state.locked_project.lock().err_str(codes::INTERNAL)?;
    let project_path = locked
        .as_ref()
        .ok_or_else(|| {
            crate::errors::RalphError {
                code: codes::PROJECT_LOCK,
                message: "No project locked".to_owned(),
            }
            .to_string()
        })?
        .clone();
    drop(locked);

    let mcp_config = if let Some(mode) = mcp_mode {
        Some(state.generate_mcp_config(&mode, &project_path)?)
    } else {
        None
    };

    let config = SessionConfig { model, thinking };

    state
        .pty_manager
        .create_session(app, session_id, &project_path, mcp_config, config)
}

#[tauri::command]
pub fn send_terminal_input(
    state: State<'_, AppState>,
    session_id: String,
    data: Vec<u8>,
) -> Result<(), String> {
    state.pty_manager.send_input(&session_id, &data)
}

#[tauri::command]
pub fn resize_pty(
    state: State<'_, AppState>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    state.pty_manager.resize(&session_id, cols, rows)
}

#[tauri::command]
pub fn terminate_pty_session(state: State<'_, AppState>, session_id: String) -> Result<(), String> {
    state.pty_manager.terminate(&session_id)
}

#[tauri::command]
pub fn create_pty_session_for_task(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    task_id: u32,
    model: Option<String>,
    thinking: Option<bool>,
) -> Result<(), String> {
    let locked = state.locked_project.lock().err_str(codes::INTERNAL)?;
    let project_path = locked
        .as_ref()
        .ok_or_else(|| {
            crate::errors::RalphError {
                code: codes::PROJECT_LOCK,
                message: "No project locked".to_owned(),
            }
            .to_string()
        })?
        .clone();
    drop(locked);

    let mcp_config = Some(state.generate_mcp_config_for_task(task_id, &project_path)?);

    let config = SessionConfig { model, thinking };

    state
        .pty_manager
        .create_session(app, session_id, &project_path, mcp_config, config)
}
