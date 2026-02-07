use crate::terminal::{PTYManager, SessionConfig};
use prompt_builder::{CodebaseSnapshot, PromptContext};
use sqlite_db::{FeatureInput, Priority, SqliteDb, TaskInput};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, State};

const MAX_SCAN_DEPTH: usize = 5; // Max 5 levels deep
const MAX_PROJECTS: usize = 100; // Max 100 projects returned
const EXCLUDED_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    "target", // Rust builds
    "build",
    "dist",
    ".next", // Next.js
    ".venv", // Python venv
    "venv",
    "__pycache__",
    ".cache",
    "tmp",
    "temp",
    ".tmp",
    "vendor", // Go/PHP dependencies
    ".idea",  // IDEs
    ".vscode",
    "Library", // macOS system
    "Applications",
];

#[derive(Debug, Clone, serde::Serialize)]
pub struct RalphProject {
    pub name: String,
    pub path: String,
}

pub struct AppState {
    pub locked_project: Mutex<Option<PathBuf>>,
    pub db: Mutex<Option<SqliteDb>>,
    pub codebase_snapshot: Mutex<Option<CodebaseSnapshot>>,
    pub pty_manager: PTYManager,
    mcp_dir: PathBuf,
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
    fn build_prompt_context(
        &self,
        project_path: &std::path::Path,
        user_input: Option<String>,
        instruction_overrides: std::collections::HashMap<String, String>,
    ) -> Result<PromptContext, String> {
        let ralph_dir = project_path.join(".ralph");
        let db_path = ralph_dir.join("db").join("ralph.db");

        let db_guard = self.db.lock().map_err(|e| e.to_string())?;
        let db = db_guard
            .as_ref()
            .ok_or_else(|| "No project locked (database not open)".to_owned())?;

        let snapshot = self
            .codebase_snapshot
            .lock()
            .map_err(|e| e.to_string())?
            .clone();

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
            target_task_id: None,
            target_feature: None,
            codebase_snapshot: snapshot,
            instruction_overrides,
        })
    }

    fn generate_mcp_config(
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
        let ctx = self.build_prompt_context(project_path, None, overrides)?;

        let (scripts, config_json) = prompt_builder::mcp::generate(&ctx, &recipe.mcp_tools);

        std::fs::create_dir_all(&self.mcp_dir)
            .map_err(|e| format!("Failed to create MCP dir: {e}"))?;

        for script in &scripts {
            let script_path = self.mcp_dir.join(&script.filename);
            std::fs::write(&script_path, &script.content)
                .map_err(|e| format!("Failed to write MCP script: {e}"))?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755))
                    .map_err(|e| format!("Failed to chmod MCP script: {e}"))?;
            }
        }

        let config_path = self.mcp_dir.join(format!("mcp-{mode}.json"));
        std::fs::write(&config_path, &config_json)
            .map_err(|e| format!("Failed to write MCP config: {e}"))?;

        Ok(config_path)
    }
}

fn get_db<'a>(
    state: &'a State<'a, AppState>,
) -> Result<std::sync::MutexGuard<'a, Option<SqliteDb>>, String> {
    let guard = state.db.lock().map_err(|e| e.to_string())?;
    if guard.is_none() {
        return Err("No project locked (database not open)".to_owned());
    }
    Ok(guard)
}

#[tauri::command]
pub fn validate_project_path(path: String) -> Result<(), String> {
    let path = PathBuf::from(&path);

    if !path.exists() {
        return Err(format!("Directory not found: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("Not a directory: {}", path.display()));
    }

    let ralph_dir = path.join(".ralph");
    if !ralph_dir.exists() {
        return Err(format!(
            "No .ralph/ folder. Initialize with:\n  ralph --init \"{}\"",
            path.display()
        ));
    }
    if !ralph_dir.is_dir() {
        return Err(format!(
            "{} exists but is not a directory",
            ralph_dir.display()
        ));
    }

    let db_file = ralph_dir.join("db").join("ralph.db");
    if !db_file.exists() {
        return Err(format!(
            "No .ralph/db/ralph.db found. Initialize with:\n  ralph --init \"{}\"",
            path.display()
        ));
    }

    Ok(())
}

#[tauri::command]
pub fn initialize_ralph_project(path: String, project_title: String) -> Result<(), String> {
    let path = PathBuf::from(&path);

    if !path.exists() {
        return Err(format!("Directory not found: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("Not a directory: {}", path.display()));
    }

    let ralph_dir = path.join(".ralph");
    if ralph_dir.exists() {
        return Err(format!(".ralph/ already exists at {}", path.display()));
    }

    std::fs::create_dir(&ralph_dir)
        .map_err(|e| format!("Failed to create .ralph/ directory: {e}"))?;

    let db_dir = ralph_dir.join("db");
    std::fs::create_dir(&db_dir)
        .map_err(|e| format!("Failed to create .ralph/db/ directory: {e}"))?;

    let db_path = db_dir.join("ralph.db");
    let db = SqliteDb::open(&db_path)?;
    db.seed_defaults()?;
    db.initialize_metadata(
        project_title.clone(),
        Some("Add project description here".to_owned()),
    )?;
    let claude_path = ralph_dir.join("CLAUDE.RALPH.md");
    let claude_template = format!(
        "# {project_title} - Ralph Context

## Project Overview

Add context about this project that Claude should know when working on it.

## Architecture

Describe the architecture, tech stack, and key components.

## Coding Standards

- List any coding conventions
- Style guides
- Best practices

## Important Notes

- Any gotchas or things to watch out for
- Known issues or limitations
- Dependencies or external services
"
    );

    std::fs::write(&claude_path, claude_template)
        .map_err(|e| format!("Failed to create CLAUDE.RALPH.md: {e}"))?;

    Ok(())
}

#[tauri::command]
pub fn set_locked_project(state: State<'_, AppState>, path: String) -> Result<(), String> {
    validate_project_path(path.clone())?;

    let canonical_path =
        std::fs::canonicalize(&path).map_err(|e| format!("Failed to resolve path: {e}"))?;

    let mut locked = state.locked_project.lock().map_err(|e| e.to_string())?;
    if locked.is_some() {
        return Err("Project already locked for this session".to_owned());
    }

    let db_path = canonical_path.join(".ralph").join("db").join("ralph.db");
    let db = SqliteDb::open(&db_path)?;

    let snapshot = prompt_builder::snapshot::analyze(&canonical_path);
    let mut snap_guard = state.codebase_snapshot.lock().map_err(|e| e.to_string())?;
    *snap_guard = Some(snapshot);

    let mut db_guard = state.db.lock().map_err(|e| e.to_string())?;
    *db_guard = Some(db);

    *locked = Some(canonical_path);
    Ok(())
}

#[tauri::command]
pub fn get_locked_project(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let locked = state.locked_project.lock().map_err(|e| e.to_string())?;
    Ok(locked.as_ref().map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
pub fn start_loop() -> Result<(), String> {
    Err("Not implemented".to_owned())
}

#[tauri::command]
pub fn pause_loop() -> Result<(), String> {
    Err("Not implemented".to_owned())
}

#[tauri::command]
pub fn resume_loop() -> Result<(), String> {
    Err("Not implemented".to_owned())
}

#[tauri::command]
pub fn stop_loop() -> Result<(), String> {
    Err("Not implemented".to_owned())
}

#[tauri::command]
pub fn get_loop_state() -> Result<(), String> {
    Err("Not implemented".to_owned())
}

#[tauri::command]
pub fn scan_for_ralph_projects(root_dir: Option<String>) -> Result<Vec<RalphProject>, String> {
    let scan_path = if let Some(dir) = root_dir {
        PathBuf::from(dir)
    } else {
        // Default to user's home directory for a sane, predictable scan location
        dirs::home_dir().ok_or("Failed to get home directory")?
    };

    let mut projects = Vec::new();

    fn scan_recursive(
        path: &PathBuf,
        projects: &mut Vec<RalphProject>,
        depth: usize,
        max_depth: usize,
        max_projects: usize,
    ) {
        if depth > max_depth || projects.len() >= max_projects {
            return;
        }

        if !path.is_dir() {
            return;
        }

        let ralph_dir = path.join(".ralph");
        if ralph_dir.exists() && ralph_dir.is_dir() {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_owned();

            projects.push(RalphProject {
                name,
                path: path.to_string_lossy().to_string(),
            });

            if projects.len() >= max_projects {
                return;
            }
        }

        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        let entry_path = entry.path();

                        if let Some(dir_name) = entry_path.file_name().and_then(|n| n.to_str()) {
                            if EXCLUDED_DIRS.contains(&dir_name) {
                                continue;
                            }
                        }

                        scan_recursive(&entry_path, projects, depth + 1, max_depth, max_projects);

                        if projects.len() >= max_projects {
                            return;
                        }
                    }
                }
            }
        }
    }

    scan_recursive(&scan_path, &mut projects, 0, MAX_SCAN_DEPTH, MAX_PROJECTS);

    Ok(projects)
}

#[tauri::command]
pub fn get_current_dir() -> Result<String, String> {
    let path = dirs::home_dir().ok_or("Failed to get home directory")?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn create_task(
    state: State<'_, AppState>,
    feature: String,
    discipline: String,
    title: String,
    description: Option<String>,
    priority: Option<String>,
    tags: Vec<String>,
    depends_on: Option<Vec<u32>>,
    acceptance_criteria: Option<Vec<String>>,
    context_files: Option<Vec<String>>,
    output_artifacts: Option<Vec<String>>,
    hints: Option<String>,
    estimated_turns: Option<u32>,
    provenance: Option<String>,
) -> Result<String, String> {
    let guard = get_db(&state)?;
    let db = guard.as_ref().unwrap();

    let task_input = TaskInput {
        feature: normalize_feature_name(&feature)?,
        discipline,
        title,
        description,
        priority: parse_priority(priority.as_deref()),
        tags,
        depends_on: depends_on.unwrap_or_default(),
        acceptance_criteria,
        context_files: context_files.unwrap_or_default(),
        output_artifacts: output_artifacts.unwrap_or_default(),
        hints,
        estimated_turns,
        provenance: parse_provenance(provenance.as_deref()),
    };

    let task_id = db.create_task(task_input)?;
    Ok(task_id.to_string())
}

fn normalize_feature_name(name: &str) -> Result<String, String> {
    if name.contains('/') || name.contains(':') || name.contains('\\') {
        return Err("Feature name cannot contain /, :, or \\".to_owned());
    }

    Ok(name.to_lowercase().trim().replace(char::is_whitespace, "-"))
}

fn parse_priority(priority: Option<&str>) -> Option<Priority> {
    priority.and_then(|p| match p {
        "low" => Some(Priority::Low),
        "medium" => Some(Priority::Medium),
        "high" => Some(Priority::High),
        "critical" => Some(Priority::Critical),
        _ => None,
    })
}

fn parse_provenance(provenance: Option<&str>) -> Option<sqlite_db::TaskProvenance> {
    provenance.and_then(|p| match p {
        "agent" => Some(sqlite_db::TaskProvenance::Agent),
        "human" => Some(sqlite_db::TaskProvenance::Human),
        "system" => Some(sqlite_db::TaskProvenance::System),
        _ => None,
    })
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerConfigData {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisciplineConfig {
    pub name: String,
    pub display_name: String,
    pub icon: String,
    pub color: String,
    pub acronym: String,
    pub system_prompt: Option<String>,
    pub skills: Vec<String>,
    pub conventions: Option<String>,
    pub mcp_servers: Vec<McpServerConfigData>,
}

#[tauri::command]
pub fn get_disciplines_config(state: State<'_, AppState>) -> Result<Vec<DisciplineConfig>, String> {
    let guard = get_db(&state)?;
    let db = guard.as_ref().unwrap();
    Ok(db
        .get_disciplines()
        .iter()
        .map(|d| DisciplineConfig {
            name: d.name.clone(),
            display_name: d.display_name.clone(),
            icon: d.icon.clone(),
            color: d.color.clone(),
            acronym: d.acronym.clone(),
            system_prompt: d.system_prompt.clone(),
            skills: d.skills.clone(),
            conventions: d.conventions.clone(),
            mcp_servers: d
                .mcp_servers
                .iter()
                .map(|m| McpServerConfigData {
                    name: m.name.clone(),
                    command: m.command.clone(),
                    args: m.args.clone(),
                    env: m.env.clone(),
                })
                .collect(),
        })
        .collect())
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureConfig {
    pub name: String,
    pub display_name: String,
    pub acronym: String,
}

#[tauri::command]
pub fn get_features_config(state: State<'_, AppState>) -> Result<Vec<FeatureConfig>, String> {
    let guard = get_db(&state)?;
    let db = guard.as_ref().unwrap();
    Ok(db
        .get_features()
        .iter()
        .map(|f| FeatureConfig {
            name: f.name.clone(),
            display_name: f.display_name.clone(),
            acronym: f.acronym.clone(),
        })
        .collect())
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureLearningData {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iteration: Option<u32>,
    pub created: String,
    pub hit_count: u32,
    pub reviewed: bool,
    pub review_count: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureData {
    pub name: String,
    pub display_name: String,
    pub acronym: String,
    pub description: Option<String>,
    pub created: Option<String>,
    pub knowledge_paths: Vec<String>,
    pub context_files: Vec<String>,
    pub architecture: Option<String>,
    pub boundaries: Option<String>,
    pub learnings: Vec<FeatureLearningData>,
    pub dependencies: Vec<String>,
}

fn learning_source_str(source: sqlite_db::LearningSource) -> String {
    match source {
        sqlite_db::LearningSource::Auto => "auto".into(),
        sqlite_db::LearningSource::Agent => "agent".into(),
        sqlite_db::LearningSource::Human => "human".into(),
        sqlite_db::LearningSource::OpusReviewed => "opus_reviewed".into(),
    }
}

fn to_learning_data(l: &sqlite_db::FeatureLearning) -> FeatureLearningData {
    FeatureLearningData {
        text: l.text.clone(),
        reason: l.reason.clone(),
        source: learning_source_str(l.source),
        task_id: l.task_id,
        iteration: l.iteration,
        created: l.created.clone(),
        hit_count: l.hit_count,
        reviewed: l.reviewed,
        review_count: l.review_count,
    }
}

#[tauri::command]
pub fn get_features(state: State<'_, AppState>) -> Result<Vec<FeatureData>, String> {
    let guard = get_db(&state)?;
    let db = guard.as_ref().unwrap();
    Ok(db
        .get_features()
        .iter()
        .map(|f| FeatureData {
            name: f.name.clone(),
            display_name: f.display_name.clone(),
            acronym: f.acronym.clone(),
            description: f.description.clone(),
            created: f.created.clone(),
            knowledge_paths: f.knowledge_paths.clone(),
            context_files: f.context_files.clone(),
            architecture: f.architecture.clone(),
            boundaries: f.boundaries.clone(),
            learnings: f.learnings.iter().map(to_learning_data).collect(),
            dependencies: f.dependencies.clone(),
        })
        .collect())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn create_feature(
    state: State<'_, AppState>,
    name: String,
    display_name: String,
    acronym: String,
    description: Option<String>,
    architecture: Option<String>,
    boundaries: Option<String>,
    knowledge_paths: Option<Vec<String>>,
    context_files: Option<Vec<String>>,
    dependencies: Option<Vec<String>>,
) -> Result<(), String> {
    let guard = get_db(&state)?;
    let db = guard.as_ref().unwrap();

    let normalized_name = normalize_feature_name(&name)?;

    db.create_feature(FeatureInput {
        name: normalized_name,
        display_name,
        acronym,
        description,
        architecture,
        boundaries,
        knowledge_paths: knowledge_paths.unwrap_or_default(),
        context_files: context_files.unwrap_or_default(),
        dependencies: dependencies.unwrap_or_default(),
    })
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn update_feature(
    state: State<'_, AppState>,
    name: String,
    display_name: String,
    acronym: String,
    description: Option<String>,
    architecture: Option<String>,
    boundaries: Option<String>,
    knowledge_paths: Option<Vec<String>>,
    context_files: Option<Vec<String>>,
    dependencies: Option<Vec<String>>,
) -> Result<(), String> {
    let guard = get_db(&state)?;
    let db = guard.as_ref().unwrap();
    db.update_feature(FeatureInput {
        name,
        display_name,
        acronym,
        description,
        architecture,
        boundaries,
        knowledge_paths: knowledge_paths.unwrap_or_default(),
        context_files: context_files.unwrap_or_default(),
        dependencies: dependencies.unwrap_or_default(),
    })
}

#[tauri::command]
pub fn create_discipline(
    state: State<'_, AppState>,
    name: String,
    display_name: String,
    acronym: String,
    icon: String,
    color: String,
) -> Result<(), String> {
    let guard = get_db(&state)?;
    let db = guard.as_ref().unwrap();

    let normalized_name = name.to_lowercase().trim().replace(char::is_whitespace, "-");

    db.create_discipline(normalized_name, display_name, acronym, icon, color)
}

#[tauri::command]
pub fn update_discipline(
    state: State<'_, AppState>,
    name: String,
    display_name: String,
    acronym: String,
    icon: String,
    color: String,
) -> Result<(), String> {
    let guard = get_db(&state)?;
    let db = guard.as_ref().unwrap();
    db.update_discipline(name, display_name, acronym, icon, color)
}

#[tauri::command]
pub fn append_feature_learning(
    state: State<'_, AppState>,
    feature_name: String,
    text: String,
    reason: Option<String>,
    source: Option<String>,
    task_id: Option<u32>,
    iteration: Option<u32>,
) -> Result<bool, String> {
    let guard = get_db(&state)?;
    let db = guard.as_ref().unwrap();

    let learning = match source.as_deref() {
        Some("human") => sqlite_db::FeatureLearning::from_human(text, reason),
        Some("agent") => sqlite_db::FeatureLearning::from_agent(text, reason, task_id),
        Some("auto") | None => {
            sqlite_db::FeatureLearning::auto_extracted(text, iteration.unwrap_or(0), task_id)
        }
        Some(other) => return Err(format!("Invalid learning source: {other}")),
    };

    db.append_feature_learning(&feature_name, learning, 50)
}

#[tauri::command]
pub fn remove_feature_learning(
    state: State<'_, AppState>,
    feature_name: String,
    index: usize,
) -> Result<(), String> {
    let guard = get_db(&state)?;
    let db = guard.as_ref().unwrap();
    db.remove_feature_learning(&feature_name, index)
}

#[tauri::command]
pub fn add_feature_context_file(
    state: State<'_, AppState>,
    feature_name: String,
    file_path: String,
) -> Result<bool, String> {
    let guard = get_db(&state)?;
    let db = guard.as_ref().unwrap();
    db.add_feature_context_file(&feature_name, &file_path, 100)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn update_task(
    state: State<'_, AppState>,
    id: u32,
    feature: String,
    discipline: String,
    title: String,
    description: Option<String>,
    priority: Option<String>,
    tags: Vec<String>,
    depends_on: Option<Vec<u32>>,
    acceptance_criteria: Option<Vec<String>>,
    context_files: Option<Vec<String>>,
    output_artifacts: Option<Vec<String>>,
    hints: Option<String>,
    estimated_turns: Option<u32>,
    provenance: Option<String>,
) -> Result<(), String> {
    let guard = get_db(&state)?;
    let db = guard.as_ref().unwrap();

    let task_input = TaskInput {
        feature: normalize_feature_name(&feature)?,
        discipline,
        title,
        description,
        priority: parse_priority(priority.as_deref()),
        tags,
        depends_on: depends_on.unwrap_or_default(),
        acceptance_criteria,
        context_files: context_files.unwrap_or_default(),
        output_artifacts: output_artifacts.unwrap_or_default(),
        hints,
        estimated_turns,
        provenance: parse_provenance(provenance.as_deref()),
    };

    db.update_task(id, task_input)
}

#[tauri::command]
pub fn set_task_status(state: State<'_, AppState>, id: u32, status: String) -> Result<(), String> {
    let guard = get_db(&state)?;
    let db = guard.as_ref().unwrap();
    let status =
        sqlite_db::TaskStatus::parse(&status).ok_or_else(|| format!("Invalid status: {status}"))?;
    db.set_task_status(id, status)
}

#[tauri::command]
pub fn delete_task(state: State<'_, AppState>, id: u32) -> Result<(), String> {
    let guard = get_db(&state)?;
    let db = guard.as_ref().unwrap();
    db.delete_task(id)
}

#[tauri::command]
pub fn delete_feature(state: State<'_, AppState>, name: String) -> Result<(), String> {
    let guard = get_db(&state)?;
    let db = guard.as_ref().unwrap();
    db.delete_feature(name)
}

#[tauri::command]
pub fn delete_discipline(state: State<'_, AppState>, name: String) -> Result<(), String> {
    let guard = get_db(&state)?;
    let db = guard.as_ref().unwrap();
    db.delete_discipline(name)
}

#[tauri::command]
pub fn add_task_comment(
    state: State<'_, AppState>,
    task_id: u32,
    author: String,
    agent_task_id: Option<u32>,
    body: String,
) -> Result<(), String> {
    let guard = get_db(&state)?;
    let db = guard.as_ref().unwrap();
    let comment_author = match author.as_str() {
        "human" => sqlite_db::CommentAuthor::Human,
        "agent" => sqlite_db::CommentAuthor::Agent,
        _ => return Err(format!("Invalid author: {author}")),
    };
    db.add_comment(task_id, comment_author, agent_task_id, body)
}

#[tauri::command]
pub fn update_task_comment(
    state: State<'_, AppState>,
    task_id: u32,
    comment_id: u32,
    body: String,
) -> Result<(), String> {
    let guard = get_db(&state)?;
    let db = guard.as_ref().unwrap();
    db.update_comment(task_id, comment_id, body)
}

#[tauri::command]
pub fn delete_task_comment(
    state: State<'_, AppState>,
    task_id: u32,
    comment_id: u32,
) -> Result<(), String> {
    let guard = get_db(&state)?;
    let db = guard.as_ref().unwrap();
    db.delete_comment(task_id, comment_id)
}


#[tauri::command]
pub fn get_tasks(state: State<'_, AppState>) -> Result<Vec<sqlite_db::Task>, String> {
    let guard = get_db(&state)?;
    let db = guard.as_ref().unwrap();
    Ok(db.get_tasks())
}

#[tauri::command]
pub fn get_feature_stats(state: State<'_, AppState>) -> Result<Vec<sqlite_db::GroupStats>, String> {
    let guard = get_db(&state)?;
    let db = guard.as_ref().unwrap();
    Ok(db.get_feature_stats())
}

#[tauri::command]
pub fn get_discipline_stats(
    state: State<'_, AppState>,
) -> Result<Vec<sqlite_db::GroupStats>, String> {
    let guard = get_db(&state)?;
    let db = guard.as_ref().unwrap();
    Ok(db.get_discipline_stats())
}

#[tauri::command]
pub fn get_project_progress(
    state: State<'_, AppState>,
) -> Result<sqlite_db::ProjectProgress, String> {
    let guard = get_db(&state)?;
    let db = guard.as_ref().unwrap();
    Ok(db.get_project_progress())
}

#[tauri::command]
pub fn get_all_tags(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let guard = get_db(&state)?;
    let db = guard.as_ref().unwrap();
    Ok(db.get_all_tags())
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfo {
    pub title: String,
    pub description: Option<String>,
    pub created: Option<String>,
}

#[tauri::command]
pub fn get_project_info(state: State<'_, AppState>) -> Result<ProjectInfo, String> {
    let guard = get_db(&state)?;
    let db = guard.as_ref().unwrap();
    let info = db.get_project_info();
    Ok(ProjectInfo {
        title: info.title.clone(),
        description: info.description.clone(),
        created: info.created,
    })
}


fn get_locked_project_path(state: &State<'_, AppState>) -> Result<PathBuf, String> {
    let locked = state.locked_project.lock().map_err(|e| e.to_string())?;
    locked
        .as_ref()
        .cloned()
        .ok_or_else(|| "No project locked".to_owned())
}

fn parse_prompt_type(s: &str) -> Result<prompt_builder::PromptType, String> {
    prompt_builder::PromptType::parse(s).ok_or_else(|| format!("Unknown prompt type: {s}"))
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptPreviewSection {
    pub name: String,
    pub content: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptPreview {
    pub sections: Vec<PromptPreviewSection>,
    pub full_prompt: String,
}

#[tauri::command]
pub fn preview_prompt(
    state: State<'_, AppState>,
    prompt_type: String,
    instruction_override: Option<String>,
    user_input: Option<String>,
) -> Result<PromptPreview, String> {
    let project_path = get_locked_project_path(&state)?;
    let pt = parse_prompt_type(&prompt_type)?;
    // Convert single override to per-section HashMap
    let overrides = instruction_override.map_or_else(std::collections::HashMap::new, |text| {
        let section_name = format!("{prompt_type}_instructions");
        let mut map = std::collections::HashMap::new();
        map.insert(section_name, text);
        map
    });
    let ctx = state.build_prompt_context(&project_path, user_input, overrides)?;

    let sections: Vec<PromptPreviewSection> = prompt_builder::build_sections(pt, &ctx)
        .into_iter()
        .map(|s| PromptPreviewSection {
            name: s.name,
            content: s.content,
        })
        .collect();

    let full_prompt = sections
        .iter()
        .map(|s| s.content.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");

    Ok(PromptPreview {
        sections,
        full_prompt,
    })
}

#[tauri::command]
pub fn get_default_instructions(prompt_type: String) -> Result<String, String> {
    let pt = parse_prompt_type(&prompt_type)?;
    Ok(prompt_builder::default_instructions(pt))
}

#[tauri::command]
pub fn save_prompt_instructions(
    state: State<'_, AppState>,
    prompt_type: String,
    text: String,
) -> Result<(), String> {
    let project_path = get_locked_project_path(&state)?;
    let prompts_dir = project_path.join(".ralph").join("prompts");
    std::fs::create_dir_all(&prompts_dir)
        .map_err(|e| format!("Failed to create prompts dir: {e}"))?;
    let file_path = prompts_dir.join(format!("{prompt_type}_instructions.md"));
    std::fs::write(&file_path, &text).map_err(|e| format!("Failed to save instructions: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn load_prompt_instructions(
    state: State<'_, AppState>,
    prompt_type: String,
) -> Result<Option<String>, String> {
    let project_path = get_locked_project_path(&state)?;
    let file_path = project_path
        .join(".ralph")
        .join("prompts")
        .join(format!("{prompt_type}_instructions.md"));
    match std::fs::read_to_string(&file_path) {
        Ok(text) => Ok(Some(text)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("Failed to read instructions: {e}")),
    }
}

#[tauri::command]
pub fn reset_prompt_instructions(
    state: State<'_, AppState>,
    prompt_type: String,
) -> Result<(), String> {
    let project_path = get_locked_project_path(&state)?;
    let file_path = project_path
        .join(".ralph")
        .join("prompts")
        .join(format!("{prompt_type}_instructions.md"));
    match std::fs::remove_file(&file_path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("Failed to delete instructions: {e}")),
    }
}


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionConfig {
    pub name: String,
    pub enabled: bool,
    pub instruction_override: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomRecipe {
    pub name: String,
    pub base_recipe: Option<String>,
    pub sections: Vec<SectionConfig>,
}

#[tauri::command]
pub fn get_section_metadata() -> Vec<prompt_builder::SectionInfo> {
    prompt_builder::sections::metadata::all_sections()
}

#[tauri::command]
pub fn get_recipe_sections(prompt_type: String) -> Result<Vec<SectionConfig>, String> {
    let pt = parse_prompt_type(&prompt_type)?;
    let names = prompt_builder::get_recipe_section_names(pt);
    let all_meta = prompt_builder::sections::metadata::all_sections();

    Ok(names
        .iter()
        .map(|name| SectionConfig {
            name: (*name).to_owned(),
            enabled: true,
            instruction_override: None,
        })
        .chain(
            all_meta
                .iter()
                .filter(|info| !names.contains(&info.name))
                .map(|info| SectionConfig {
                    name: info.name.to_owned(),
                    enabled: false,
                    instruction_override: None,
                }),
        )
        .collect())
}

#[tauri::command]
pub fn preview_custom_recipe(
    state: State<'_, AppState>,
    sections: Vec<SectionConfig>,
    user_input: Option<String>,
) -> Result<PromptPreview, String> {
    let project_path = get_locked_project_path(&state)?;

    let overrides: std::collections::HashMap<String, String> = sections
        .iter()
        .filter(|s| s.enabled && s.instruction_override.is_some())
        .map(|s| (s.name.clone(), s.instruction_override.clone().unwrap()))
        .collect();

    let ctx = state.build_prompt_context(&project_path, user_input, overrides)?;

    let enabled_names: Vec<&str> = sections
        .iter()
        .filter(|s| s.enabled)
        .map(|s| s.name.as_str())
        .collect();

    let built_sections: Vec<PromptPreviewSection> =
        prompt_builder::build_custom_sections(&enabled_names, &ctx)
            .into_iter()
            .map(|s| PromptPreviewSection {
                name: s.name,
                content: s.content,
            })
            .collect();

    let full_prompt = built_sections
        .iter()
        .map(|s| s.content.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");

    Ok(PromptPreview {
        sections: built_sections,
        full_prompt,
    })
}

#[tauri::command]
pub fn list_saved_recipes(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let project_path = get_locked_project_path(&state)?;
    let prompts_dir = project_path.join(".ralph").join("prompts");

    if !prompts_dir.exists() {
        return Ok(vec![]);
    }

    let mut names = Vec::new();
    let entries =
        std::fs::read_dir(&prompts_dir).map_err(|e| format!("Failed to read prompts dir: {e}"))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                names.push(stem.to_owned());
            }
        }
    }

    names.sort();
    Ok(names)
}

#[tauri::command]
pub fn load_saved_recipe(state: State<'_, AppState>, name: String) -> Result<CustomRecipe, String> {
    let project_path = get_locked_project_path(&state)?;
    let file_path = project_path
        .join(".ralph")
        .join("prompts")
        .join(format!("{name}.json"));

    let content =
        std::fs::read_to_string(&file_path).map_err(|e| format!("Failed to read recipe: {e}"))?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse recipe: {e}"))
}

#[tauri::command]
pub fn save_recipe(state: State<'_, AppState>, recipe: CustomRecipe) -> Result<(), String> {
    let project_path = get_locked_project_path(&state)?;
    let prompts_dir = project_path.join(".ralph").join("prompts");
    std::fs::create_dir_all(&prompts_dir)
        .map_err(|e| format!("Failed to create prompts dir: {e}"))?;

    let file_path = prompts_dir.join(format!("{}.json", recipe.name));
    let content =
        serde_json::to_string_pretty(&recipe).map_err(|e| format!("Failed to serialize: {e}"))?;

    std::fs::write(&file_path, content).map_err(|e| format!("Failed to write recipe: {e}"))
}

#[tauri::command]
pub fn delete_recipe(state: State<'_, AppState>, name: String) -> Result<(), String> {
    let project_path = get_locked_project_path(&state)?;
    let file_path = project_path
        .join(".ralph")
        .join("prompts")
        .join(format!("{name}.json"));

    match std::fs::remove_file(&file_path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("Failed to delete recipe: {e}")),
    }
}


#[tauri::command]
pub fn create_pty_session(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    mcp_mode: Option<String>,
    model: Option<String>,
    thinking: Option<bool>,
) -> Result<(), String> {
    let locked = state.locked_project.lock().map_err(|e| e.to_string())?;
    let project_path = locked.as_ref().ok_or("No project locked")?.clone();
    drop(locked);

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
pub fn send_terminal_input(
    state: State<'_, AppState>,
    session_id: String,
    data: Vec<u8>,
) -> Result<(), String> {
    state.pty_manager.send_input(&session_id, &data)
}

#[tauri::command]
pub fn resize_pty(
    state: State<'_, AppState>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    state.pty_manager.resize(&session_id, cols, rows)
}

#[tauri::command]
pub fn terminate_pty_session(state: State<'_, AppState>, session_id: String) -> Result<(), String> {
    state.pty_manager.terminate(&session_id)
}
