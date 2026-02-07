mod commands;
#[allow(dead_code)]
mod loop_engine;
mod terminal;
#[allow(dead_code)]
mod types;

use commands::AppState;
use tauri::Manager;
use tauri_plugin_cli::CliExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // WORKAROUND: WebKitGTK + NVIDIA + Wayland crash prevention
    // See: https://github.com/tauri-apps/tauri/issues/9394
    // __NV_DISABLE_EXPLICIT_SYNC=1 targets the specific NVIDIA sync issue
    // without disabling GPU acceleration entirely (unlike WEBKIT_DISABLE_DMABUF_RENDERER=1
    // which forces software rendering and causes massive input lag).
    #[cfg(target_os = "linux")]
    {
        std::env::set_var("__NV_DISABLE_EXPLICIT_SYNC", "1");
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
            commands::append_feature_learning,
            commands::remove_feature_learning,
            commands::add_feature_context_file,
            commands::create_discipline,
            commands::update_discipline,
            commands::delete_discipline,
            commands::add_task_comment,
            commands::update_task_comment,
            commands::delete_task_comment,
            commands::get_tasks,
            commands::get_feature_stats,
            commands::get_discipline_stats,
            commands::get_project_progress,
            commands::get_all_tags,
            commands::get_project_info,
            commands::preview_prompt,
            commands::get_default_instructions,
            commands::save_prompt_instructions,
            commands::load_prompt_instructions,
            commands::reset_prompt_instructions,
            commands::get_section_metadata,
            commands::get_recipe_sections,
            commands::preview_custom_recipe,
            commands::list_saved_recipes,
            commands::load_saved_recipe,
            commands::save_recipe,
            commands::delete_recipe,
            commands::create_pty_session,
            commands::send_terminal_input,
            commands::resize_pty,
            commands::terminate_pty_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
