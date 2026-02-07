mod commands;
mod terminal;

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
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_cli::init())
        .manage(AppState::default())
        .setup(|app| {
            if let Ok(matches) = app.cli().matches() {
                if let Some(project_path) = matches.args.get("project") {
                    if let serde_json::Value::String(path_str) = &project_path.value {
                        if let Err(e) = commands::validate_project_path(path_str.clone()) {
                            eprintln!("Failed to lock project: {e}");
                            std::process::exit(1);
                        }

                        let state: tauri::State<AppState> = app.state();
                        if let Err(e) = commands::set_locked_project(state, path_str.clone()) {
                            eprintln!("Error: {e}");
                            std::process::exit(1);
                        }
                    }
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::project::start_loop,
            commands::project::pause_loop,
            commands::project::resume_loop,
            commands::project::stop_loop,
            commands::project::get_loop_state,
            commands::project::scan_for_ralph_projects,
            commands::project::get_current_dir,
            commands::project::validate_project_path,
            commands::project::initialize_ralph_project,
            commands::project::set_locked_project,
            commands::project::get_locked_project,
            commands::project::get_project_info,
            commands::tasks::create_task,
            commands::tasks::update_task,
            commands::tasks::set_task_status,
            commands::tasks::delete_task,
            commands::tasks::add_task_comment,
            commands::tasks::update_task_comment,
            commands::tasks::delete_task_comment,
            commands::tasks::get_tasks,
            commands::tasks::get_feature_stats,
            commands::tasks::get_discipline_stats,
            commands::tasks::get_project_progress,
            commands::tasks::get_all_tags,
            commands::features::get_disciplines_config,
            commands::features::get_features,
            commands::features::get_features_config,
            commands::features::create_feature,
            commands::features::update_feature,
            commands::features::delete_feature,
            commands::features::append_feature_learning,
            commands::features::remove_feature_learning,
            commands::features::add_feature_context_file,
            commands::features::create_discipline,
            commands::features::update_discipline,
            commands::features::delete_discipline,
            commands::prompts::preview_prompt,
            commands::prompts::get_default_instructions,
            commands::prompts::save_prompt_instructions,
            commands::prompts::load_prompt_instructions,
            commands::prompts::reset_prompt_instructions,
            commands::prompts::get_section_metadata,
            commands::prompts::get_recipe_sections,
            commands::prompts::preview_custom_recipe,
            commands::prompts::list_saved_recipes,
            commands::prompts::load_saved_recipe,
            commands::prompts::save_recipe,
            commands::prompts::delete_recipe,
            commands::terminal::create_pty_session,
            commands::terminal::send_terminal_input,
            commands::terminal::resize_pty,
            commands::terminal::terminate_pty_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
