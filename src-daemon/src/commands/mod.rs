use ralph_errors::{codes, err_string, RalphResult};

use crate::state::AppState;

pub mod agent_sessions;
pub mod project;
pub mod prompts;
pub mod protocol;
pub mod subsystems;
pub mod tasks;
pub mod terminal_bridge;

pub async fn handle_command(
    state: &AppState,
    command: &str,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    match command {
        "protocol_version_get" => protocol::protocol_version_get(payload),

        "project_validate_path" => project::project_validate_path(payload),
        "project_initialize" => project::project_initialize(payload),
        "project_lock_set" => project::project_lock_set(state, payload),
        "project_lock_get" => project::project_lock_get(state, payload),
        "project_recent_list" => project::project_recent_list(state, payload),
        "project_scan" => project::project_scan(payload),
        "project_info_get" => project::project_info_get(state, payload),
        "system_home_dir_get" => project::system_home_dir_get(payload),

        "execution_start" => project::execution_start(payload),
        "execution_pause" => project::execution_pause(payload),
        "execution_resume" => project::execution_resume(payload),
        "execution_stop" => project::execution_stop(payload),
        "execution_state_get" => project::execution_state_get(payload),

        "subsystems_list" => subsystems::subsystems_list(state, payload),
        "subsystems_create" => subsystems::subsystems_create(state, payload),
        "subsystems_update" => subsystems::subsystems_update(state, payload),
        "subsystems_delete" => subsystems::subsystems_delete(state, payload),
        "subsystems_comment_add" => subsystems::subsystems_comment_add(state, payload).await,
        "subsystems_comment_update" => subsystems::subsystems_comment_update(state, payload).await,
        "subsystems_comment_delete" => subsystems::subsystems_comment_delete(state, payload),

        "disciplines_list" => subsystems::disciplines_list(state, payload),
        "disciplines_create" => subsystems::disciplines_create(state, payload),
        "disciplines_update" => subsystems::disciplines_update(state, payload),
        "disciplines_delete" => subsystems::disciplines_delete(state, payload),
        "stacks_metadata_list" => subsystems::stacks_metadata_list(payload),
        "disciplines_image_data_get" => subsystems::disciplines_image_data_get(state, payload),
        "disciplines_cropped_image_get" => {
            subsystems::disciplines_cropped_image_get(state, payload)
        }

        "prompt_builder_config_list" => prompts::prompt_builder_config_list(state, payload),
        "prompt_builder_config_get" => prompts::prompt_builder_config_get(state, payload),
        "prompt_builder_config_save" => prompts::prompt_builder_config_save(state, payload),
        "prompt_builder_config_delete" => prompts::prompt_builder_config_delete(state, payload),
        "prompt_builder_preview" => prompts::prompt_builder_preview(state, payload),

        "terminal_start_session" => terminal_bridge::terminal_start_session(state, payload),
        "terminal_start_task_session" => {
            terminal_bridge::terminal_start_task_session(state, payload)
        }
        "terminal_resolve_task_launch_config" => {
            terminal_bridge::terminal_resolve_task_launch_config(state, payload)
        }
        "terminal_start_human_session" => {
            terminal_bridge::terminal_start_human_session(state, payload)
        }
        "terminal_list_model_form_tree" => terminal_bridge::terminal_list_model_form_tree(payload),
        "terminal_send_input" => terminal_bridge::terminal_send_input(state, payload),
        "terminal_resize" => terminal_bridge::terminal_resize(state, payload),
        "terminal_set_stream_mode" => terminal_bridge::terminal_set_stream_mode(state, payload),
        "terminal_replay_output" => terminal_bridge::terminal_replay_output(state, payload),
        "terminal_emit_system_message" => {
            terminal_bridge::terminal_emit_system_message(state, payload)
        }
        "terminal_terminate" => terminal_bridge::terminal_terminate(state, payload),

        "tasks_create" => tasks::tasks_create(state, payload),
        "tasks_update" => tasks::tasks_update(state, payload),
        "tasks_set_status" => tasks::tasks_set_status(state, payload),
        "tasks_delete" => tasks::tasks_delete(state, payload),
        "tasks_signal_add" => tasks::tasks_signal_add(state, payload),
        "tasks_signal_update" => tasks::tasks_signal_update(state, payload),
        "tasks_signal_delete" => tasks::tasks_signal_delete(state, payload),
        "tasks_list" => tasks::tasks_list(state, payload),
        "tasks_get" => tasks::tasks_get(state, payload),
        "tasks_list_items" => tasks::tasks_list_items(state, payload),
        "tasks_signal_summaries_get" => tasks::tasks_signal_summaries_get(state, payload),
        "tasks_ask_answer" => tasks::tasks_ask_answer(state, payload),
        "tasks_comment_reply_add" => tasks::tasks_comment_reply_add(state, payload),
        "tasks_signal_comment_add" => tasks::tasks_signal_comment_add(state, payload),
        "tasks_signal_comment_update" => tasks::tasks_signal_comment_update(state, payload),
        "tasks_signal_comment_delete" => tasks::tasks_signal_comment_delete(state, payload),
        "tasks_signal_comments_list" => tasks::tasks_signal_comments_list(state, payload),

        "agent_sessions_create_human" => {
            agent_sessions::agent_sessions_create_human(state, payload)
        }
        "agent_sessions_update_human" => {
            agent_sessions::agent_sessions_update_human(state, payload)
        }
        "agent_sessions_delete_human" => {
            agent_sessions::agent_sessions_delete_human(state, payload)
        }
        "agent_sessions_get" => agent_sessions::agent_sessions_get(state, payload),
        "agent_sessions_list_human" => agent_sessions::agent_sessions_list_human(state, payload),

        other => {
            ralph_backend::diagnostics::emit_warning(
                "ralphd",
                "unknown-command",
                &format!("Unknown command: {other}"),
            );
            Err(err_string(
                codes::INTERNAL,
                format!("Unknown command: {other}"),
            ))
        }
    }
}
