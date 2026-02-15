use crate::prompt_context::{build_prompt_context, PromptContextArgs};
use crate::session::with_db;
use prompt_builder::CodebaseSnapshot;
use ralph_errors::{codes, RalphResultExt};
use sqlite_db::SqliteDb;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub fn generate_mcp_config(
    db: &Mutex<Option<SqliteDb>>,
    codebase_snapshot: &Mutex<Option<CodebaseSnapshot>>,
    mcp_dir: &Path,
    api_server_port: Option<u16>,
    mode: &str,
    project_path: &Path,
) -> Result<PathBuf, String> {
    let prompt_type = match mode {
        "task_creation" => prompt_builder::PromptType::Braindump,
        _ => prompt_builder::PromptType::Discuss,
    };

    let mut overrides = HashMap::new();
    let override_path = project_path
        .join(".ralph")
        .join("prompts")
        .join(format!("{mode}_instructions.md"));
    if let Ok(text) = std::fs::read_to_string(&override_path) {
        let section_name = format!("{mode}_instructions");
        overrides.insert(section_name, text);
    }

    with_db(db, |db| {
        let recipe = prompt_builder::recipes::get(prompt_type);
        let ctx = build_prompt_context(PromptContextArgs {
            db,
            project_path,
            mcp_dir,
            codebase_snapshot,
            api_server_port,
            user_input: None,
            instruction_overrides: overrides,
            target_task_id: None,
        })?;

        let (scripts, config_json) =
            prompt_builder::mcp::generate(&ctx, recipe.mcp_mode, &recipe.mcp_tools);

        write_mcp_artifacts(mcp_dir, &scripts, &config_json, format!("mcp-{mode}.json"))
    })
}

pub fn generate_mcp_config_for_task(
    db: &Mutex<Option<SqliteDb>>,
    codebase_snapshot: &Mutex<Option<CodebaseSnapshot>>,
    mcp_dir: &Path,
    api_server_port: Option<u16>,
    task_id: u32,
    project_path: &Path,
) -> Result<PathBuf, String> {
    with_db(db, |db| {
        let ctx = build_prompt_context(PromptContextArgs {
            db,
            project_path,
            mcp_dir,
            codebase_snapshot,
            api_server_port,
            user_input: None,
            instruction_overrides: HashMap::new(),
            target_task_id: Some(task_id),
        })?;

        let recipe = prompt_builder::recipes::get(prompt_builder::PromptType::TaskExecution);
        let (scripts, config_json) =
            prompt_builder::mcp::generate(&ctx, recipe.mcp_mode, &recipe.mcp_tools);

        write_mcp_artifacts(
            mcp_dir,
            &scripts,
            &config_json,
            format!("mcp-task-{task_id}.json"),
        )
    })
}

fn write_mcp_artifacts(
    mcp_dir: &Path,
    scripts: &[prompt_builder::McpScript],
    config_json: &str,
    config_filename: String,
) -> Result<PathBuf, String> {
    std::fs::create_dir_all(mcp_dir).ralph_err(codes::FILESYSTEM, "Failed to create MCP dir")?;

    for script in scripts {
        let script_path = mcp_dir.join(&script.filename);
        std::fs::write(&script_path, &script.content)
            .ralph_err(codes::FILESYSTEM, "Failed to write MCP script")?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755))
                .ralph_err(codes::FILESYSTEM, "Failed to chmod MCP script")?;
        }
    }

    let config_path = mcp_dir.join(config_filename);
    std::fs::write(&config_path, config_json)
        .ralph_err(codes::FILESYSTEM, "Failed to write MCP config")?;

    Ok(config_path)
}
