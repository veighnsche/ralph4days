mod claude_client;
mod commands;
mod loop_engine;
mod terminal;
mod types;

use commands::AppState;
use tauri::Manager;
use tauri_plugin_cli::CliExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // WORKAROUND: WebKitGTK + NVIDIA + Wayland is broken
    // See: https://github.com/tauri-apps/tauri/issues/10702
    // Trade-off: Prevents crash but may cause higher CPU usage
    #[cfg(target_os = "linux")]
    {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_cli::init())
        .manage(AppState::default())
        .setup(|app| {
            // Parse CLI arguments
            if let Ok(matches) = app.cli().matches() {
                if let Some(project_path) = matches.args.get("project") {
                    if let serde_json::Value::String(path_str) = &project_path.value {
                        // Validate and lock project from CLI arg
                        if let Err(e) = commands::validate_project_path(path_str.to_string()) {
                            eprintln!("Failed to lock project: {}", e);
                            std::process::exit(1);
                        }

                        // Lock the project
                        let state: tauri::State<AppState> = app.state();
                        if let Err(e) = commands::set_locked_project(state, path_str.to_string()) {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::start_loop,
            commands::pause_loop,
            commands::resume_loop,
            commands::stop_loop,
            commands::get_loop_state,
            commands::scan_for_ralph_projects,
            commands::get_current_dir,
            commands::validate_project_path,
            commands::initialize_ralph_project,
            commands::set_locked_project,
            commands::get_locked_project,
            commands::create_task,
            commands::update_task,
            commands::set_task_status,
            commands::delete_task,
            commands::get_disciplines_config,
            commands::get_features,
            commands::get_features_config,
            commands::create_feature,
            commands::update_feature,
            commands::delete_feature,
            commands::create_discipline,
            commands::update_discipline,
            commands::delete_discipline,
            commands::add_task_comment,
            commands::update_task_comment,
            commands::delete_task_comment,
            commands::get_enriched_tasks,
            commands::get_feature_stats,
            commands::get_discipline_stats,
            commands::get_project_progress,
            commands::get_all_tags,
            commands::get_project_info,
            commands::create_pty_session,
            commands::send_terminal_input,
            commands::resize_pty,
            commands::terminate_pty_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
