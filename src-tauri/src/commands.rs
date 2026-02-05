use crate::loop_engine::LoopEngine;
use crate::types::LoopStatus;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, State};

pub struct AppState {
    pub engine: Mutex<LoopEngine>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            engine: Mutex::new(LoopEngine::new()),
        }
    }
}

#[tauri::command]
pub fn start_loop(
    app: AppHandle,
    state: State<'_, AppState>,
    project_path: String,
    max_iterations: u32,
) -> Result<(), String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine
        .start(app, PathBuf::from(project_path), max_iterations)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn pause_loop(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.pause(&app).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn resume_loop(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.resume(&app).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn stop_loop(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.stop(&app).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_loop_state(state: State<'_, AppState>) -> Result<LoopStatus, String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    Ok(engine.get_status())
}
