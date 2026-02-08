use crate::terminal::PTYManager;
use prompt_builder::{CodebaseSnapshot, PromptContext};
use ralph_errors::{codes, ralph_err, ToStringErr};
use sqlite_db::SqliteDb;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub locked_project: Mutex<Option<PathBuf>>,
    pub db: Mutex<Option<SqliteDb>>,
    pub codebase_snapshot: Mutex<Option<CodebaseSnapshot>>,
    pub pty_manager: PTYManager,
    pub(super) mcp_dir: PathBuf,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            locked_project: Mutex::new(None),
            db: Mutex::new(None),
            codebase_snapshot: Mutex::new(None),
            pty_manager: PTYManager::new(),
            mcp_dir: std::env::temp_dir().join(format!("ralph-mcp-{}", std::process::id())),
        }
    }
}

impl Drop for AppState {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.mcp_dir);
    }
}

impl AppState {
    pub(super) fn build_prompt_context(
        &self,
        project_path: &std::path::Path,
        user_input: Option<String>,
        instruction_overrides: std::collections::HashMap<String, String>,
        target_task_id: Option<u32>,
    ) -> Result<PromptContext, String> {
        let ralph_dir = project_path.join(".ralph");
        let db_path = ralph_dir.join("db").join("ralph.db");

        let db_guard = self.db.lock().err_str(codes::INTERNAL)?;
        let db = db_guard.as_ref().ok_or_else(|| {
            ralph_errors::RalphError {
                code: codes::PROJECT_LOCK,
                message: "No project locked (database not open)".to_owned(),
            }
            .to_string()
        })?;

        let snapshot = {
            let mut snap_guard = self.codebase_snapshot.lock().err_str(codes::INTERNAL)?;
            if snap_guard.is_none() {
                *snap_guard = Some(prompt_builder::snapshot::analyze(project_path));
            }
            snap_guard.clone()
        };

        Ok(PromptContext {
            features: db.get_features(),
            tasks: db.get_tasks(),
            disciplines: db.get_disciplines(),
            metadata: db.get_project_info(),
            file_contents: std::collections::HashMap::new(),
            progress_txt: None,
            learnings_txt: None,
            claude_ralph_md: None,
            project_path: project_path.to_string_lossy().to_string(),
            db_path: db_path.to_string_lossy().to_string(),
            script_dir: self.mcp_dir.to_string_lossy().to_string(),
            user_input,
            target_task_id,
            target_feature: None,
            codebase_snapshot: snapshot,
            instruction_overrides,
        })
    }

    pub(super) fn generate_mcp_config(
        &self,
        mode: &str,
        project_path: &std::path::Path,
    ) -> Result<PathBuf, String> {
        let prompt_type = match mode {
            "task_creation" => prompt_builder::PromptType::Braindump,
            _ => prompt_builder::PromptType::Discuss,
        };

        let mut overrides = std::collections::HashMap::new();
        let override_path = project_path
            .join(".ralph")
            .join("prompts")
            .join(format!("{mode}_instructions.md"));
        if let Ok(text) = std::fs::read_to_string(&override_path) {
            let section_name = format!("{mode}_instructions");
            overrides.insert(section_name, text);
        }

        let recipe = prompt_builder::recipes::get(prompt_type);
        let ctx = self.build_prompt_context(project_path, None, overrides, None)?;

        let (scripts, config_json) = prompt_builder::mcp::generate(&ctx, &recipe.mcp_tools);

        std::fs::create_dir_all(&self.mcp_dir).map_err(|e| {
            ralph_errors::RalphError {
                code: codes::FILESYSTEM,
                message: format!("Failed to create MCP dir: {e}"),
            }
            .to_string()
        })?;

        for script in &scripts {
            let script_path = self.mcp_dir.join(&script.filename);
            std::fs::write(&script_path, &script.content).map_err(|e| {
                ralph_errors::RalphError {
                    code: codes::FILESYSTEM,
                    message: format!("Failed to write MCP script: {e}"),
                }
                .to_string()
            })?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755))
                    .map_err(|e| {
                        ralph_errors::RalphError {
                            code: codes::FILESYSTEM,
                            message: format!("Failed to chmod MCP script: {e}"),
                        }
                        .to_string()
                    })?;
            }
        }

        let config_path = self.mcp_dir.join(format!("mcp-{mode}.json"));
        std::fs::write(&config_path, &config_json).map_err(|e| {
            ralph_errors::RalphError {
                code: codes::FILESYSTEM,
                message: format!("Failed to write MCP config: {e}"),
            }
            .to_string()
        })?;

        Ok(config_path)
    }

    pub(super) fn generate_mcp_config_for_task(
        &self,
        task_id: u32,
        project_path: &std::path::Path,
    ) -> Result<PathBuf, String> {
        let ctx = self.build_prompt_context(
            project_path,
            None,
            std::collections::HashMap::new(),
            Some(task_id),
        )?;

        let recipe = prompt_builder::recipes::get(prompt_builder::PromptType::TaskExecution);
        let (scripts, config_json) = prompt_builder::mcp::generate(&ctx, &recipe.mcp_tools);

        std::fs::create_dir_all(&self.mcp_dir).map_err(|e| {
            ralph_errors::RalphError {
                code: codes::FILESYSTEM,
                message: format!("Failed to create MCP dir: {e}"),
            }
            .to_string()
        })?;

        for script in &scripts {
            let script_path = self.mcp_dir.join(&script.filename);
            std::fs::write(&script_path, &script.content).map_err(|e| {
                ralph_errors::RalphError {
                    code: codes::FILESYSTEM,
                    message: format!("Failed to write MCP script: {e}"),
                }
                .to_string()
            })?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755))
                    .map_err(|e| {
                        ralph_errors::RalphError {
                            code: codes::FILESYSTEM,
                            message: format!("Failed to chmod MCP script: {e}"),
                        }
                        .to_string()
                    })?;
            }
        }

        let config_path = self.mcp_dir.join(format!("mcp-task-{task_id}.json"));
        std::fs::write(&config_path, &config_json).map_err(|e| {
            ralph_errors::RalphError {
                code: codes::FILESYSTEM,
                message: format!("Failed to write MCP config: {e}"),
            }
            .to_string()
        })?;

        Ok(config_path)
    }
}

pub(super) struct DbGuard<'a>(std::sync::MutexGuard<'a, Option<SqliteDb>>);

impl std::ops::Deref for DbGuard<'_> {
    type Target = SqliteDb;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

pub(super) fn get_db<'a>(state: &'a State<'a, AppState>) -> Result<DbGuard<'a>, String> {
    let guard = state.db.lock().err_str(codes::INTERNAL)?;
    if guard.is_none() {
        return ralph_err!(codes::PROJECT_LOCK, "No project locked (database not open)");
    }
    Ok(DbGuard(guard))
}

pub(super) fn get_locked_project_path(state: &State<'_, AppState>) -> Result<PathBuf, String> {
    let locked = state.locked_project.lock().err_str(codes::INTERNAL)?;
    locked.as_ref().cloned().ok_or_else(|| {
        ralph_errors::RalphError {
            code: codes::PROJECT_LOCK,
            message: "No project locked".to_owned(),
        }
        .to_string()
    })
}
