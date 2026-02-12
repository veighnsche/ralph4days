use super::state::AppState;
use crate::terminal::{PtyOutputEvent, SessionConfig, TERMINAL_BRIDGE_OUTPUT_EVENT};
use base64::{engine::general_purpose::STANDARD, Engine};
use ralph_errors::{codes, ToStringErr};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};

fn locked_project_path(state: &AppState) -> Result<PathBuf, String> {
    let locked = state.locked_project.lock().err_str(codes::INTERNAL)?;
    locked
        .as_ref()
        .cloned()
        .ok_or_else(|| ralph_errors::err_string(codes::PROJECT_LOCK, "No project locked"))
}

fn build_system_message_event(session_id: String, text: String) -> PtyOutputEvent {
    PtyOutputEvent {
        session_id,
        data: STANDARD.encode(text.as_bytes()),
    }
}

#[tauri::command]
pub fn terminal_bridge_start_session(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    mcp_mode: Option<String>,
    model: Option<String>,
    thinking: Option<bool>,
) -> Result<(), String> {
    let project_path = locked_project_path(state.inner())?;

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
pub fn terminal_bridge_send_input(
    state: State<'_, AppState>,
    session_id: String,
    data: Vec<u8>,
) -> Result<(), String> {
    state.pty_manager.send_input(&session_id, &data)
}

#[tauri::command]
pub fn terminal_bridge_resize(
    state: State<'_, AppState>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    state.pty_manager.resize(&session_id, cols, rows)
}

#[tauri::command]
pub fn terminal_bridge_terminate(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    state.pty_manager.terminate(&session_id)
}

#[tauri::command]
pub fn terminal_bridge_start_task_session(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    task_id: u32,
    model: Option<String>,
    thinking: Option<bool>,
) -> Result<(), String> {
    let project_path = locked_project_path(state.inner())?;

    let mcp_config = Some(state.generate_mcp_config_for_task(task_id, &project_path)?);
    let config = SessionConfig { model, thinking };

    state
        .pty_manager
        .create_session(app, session_id, &project_path, mcp_config, config)
}

#[tauri::command]
pub fn terminal_bridge_emit_system_message(
    app: AppHandle,
    session_id: String,
    text: String,
) -> Result<(), String> {
    let event = build_system_message_event(session_id, text);
    app.emit(TERMINAL_BRIDGE_OUTPUT_EVENT, event)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::STANDARD;
    use std::path::PathBuf;

    #[test]
    fn locked_project_path_errors_when_unset() {
        let state = AppState::default();
        let err = locked_project_path(&state).unwrap_err();
        assert!(err.contains("No project locked"));
    }

    #[test]
    fn locked_project_path_returns_value_when_set() {
        let state = AppState::default();
        let expected = PathBuf::from("/tmp/ralph4days-test-project");
        *state.locked_project.lock().unwrap() = Some(expected.clone());
        let actual = locked_project_path(&state).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn build_system_message_event_encodes_text_as_base64() {
        let event =
            build_system_message_event("session-1".to_owned(), "[session started]\r\n".to_owned());
        assert_eq!(event.session_id, "session-1");
        let decoded = STANDARD.decode(event.data).unwrap();
        assert_eq!(String::from_utf8(decoded).unwrap(), "[session started]\r\n");
    }
}
