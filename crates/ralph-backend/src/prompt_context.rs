use prompt_builder::{CodebaseSnapshot, PromptContext};
use ralph_errors::{codes, RalphResult, RalphResultExt};
use sqlite_db::SqliteDb;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

pub type InstructionOverrides = HashMap<String, String, RandomState>;

pub struct PromptContextArgs<'a> {
    pub db: &'a SqliteDb,
    pub project_path: &'a Path,
    pub mcp_dir: &'a Path,
    pub codebase_snapshot: &'a Mutex<Option<CodebaseSnapshot>>,
    pub api_server_port: Option<u16>,
    pub user_input: Option<String>,
    pub instruction_overrides: InstructionOverrides,
    pub target_task_id: Option<u32>,
}

pub fn build_prompt_context(args: PromptContextArgs<'_>) -> RalphResult<PromptContext> {
    let PromptContextArgs {
        db,
        project_path,
        mcp_dir,
        codebase_snapshot,
        api_server_port,
        user_input,
        instruction_overrides,
        target_task_id,
    } = args;

    let ralph_dir = project_path.join(".ralph");
    let db_path = ralph_dir.join("db").join("ralph.db");

    let snapshot = {
        let mut snap_guard = codebase_snapshot
            .lock()
            .ralph_err(codes::INTERNAL, "Codebase snapshot mutex poisoned")?;
        if snap_guard.is_none() {
            *snap_guard = Some(prompt_builder::snapshot::analyze(project_path));
        }
        snap_guard.clone()
    };

    Ok(PromptContext {
        features: db.get_subsystems()?,
        tasks: db.get_tasks()?,
        disciplines: db.get_disciplines()?,
        metadata: db.get_project_info()?,
        file_contents: HashMap::new(),
        progress_txt: None,
        learnings_txt: None,
        claude_ralph_md: None,
        project_path: project_path.to_string_lossy().to_string(),
        db_path: db_path.to_string_lossy().to_string(),
        script_dir: mcp_dir.to_string_lossy().to_string(),
        api_server_port,
        user_input,
        target_task_id,
        target_feature: None,
        codebase_snapshot: snapshot,
        instruction_overrides,
        relevant_comments: None,
    })
}
