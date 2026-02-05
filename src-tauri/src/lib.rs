mod claude_client;
mod commands;
mod loop_engine;
mod prompt_builder;
mod types;

use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::start_loop,
            commands::pause_loop,
            commands::resume_loop,
            commands::stop_loop,
            commands::get_loop_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
