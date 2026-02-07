use crate::loop_engine::LoopEngine;
use crate::mcp_generator::MCPGenerator;
use crate::terminal::{PTYManager, SessionConfig};
use crate::types::LoopStatus;
use sqlite_db::{Priority, SqliteDb, TaskInput};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, State};

// Recursive scan configuration
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
    pub engine: Mutex<LoopEngine>,
    pub locked_project: Mutex<Option<PathBuf>>,
    pub db: Mutex<Option<SqliteDb>>,
    pub pty_manager: PTYManager,
    pub mcp_generator: MCPGenerator,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            engine: Mutex::new(LoopEngine::new()),
            locked_project: Mutex::new(None),
            db: Mutex::new(None),
            pty_manager: PTYManager::new(),
            mcp_generator: MCPGenerator::new(),
        }
    }
}

/// Get a reference to the opened database. Fails if no project locked.
fn get_db<'a>(state: &'a State<'a, AppState>) -> Result<std::sync::MutexGuard<'a, Option<SqliteDb>>, String> {
    let guard = state.db.lock().map_err(|e| e.to_string())?;
    if guard.is_none() {
        return Err("No project locked (database not open)".to_string());
    }
    Ok(guard)
}

#[tauri::command]
pub fn validate_project_path(path: String) -> Result<(), String> {
    let path = PathBuf::from(&path);

    // Check path exists and is directory
    if !path.exists() {
        return Err(format!("Directory not found: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("Not a directory: {}", path.display()));
    }

    // Check .ralph/ directory exists
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

    // Check .ralph/db/ralph.db exists
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

    // Check path exists and is directory
    if !path.exists() {
        return Err(format!("Directory not found: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("Not a directory: {}", path.display()));
    }

    // Create .ralph/ directory
    let ralph_dir = path.join(".ralph");
    if ralph_dir.exists() {
        return Err(format!(".ralph/ already exists at {}", path.display()));
    }

    std::fs::create_dir(&ralph_dir)
        .map_err(|e| format!("Failed to create .ralph/ directory: {}", e))?;

    // Create .ralph/db/ directory
    let db_dir = ralph_dir.join("db");
    std::fs::create_dir(&db_dir)
        .map_err(|e| format!("Failed to create .ralph/db/ directory: {}", e))?;

    // Create and initialize the SQLite database
    let db_path = db_dir.join("ralph.db");
    let db = SqliteDb::open(&db_path)?;
    db.seed_defaults()?;
    db.initialize_metadata(
        project_title.clone(),
        Some("Add project description here".to_string()),
    )?;

    // Create optional CLAUDE.RALPH.md template
    let claude_path = ralph_dir.join("CLAUDE.RALPH.md");
    let claude_template = format!(
        r#"# {} - Ralph Context

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
"#,
        project_title
    );

    std::fs::write(&claude_path, claude_template)
        .map_err(|e| format!("Failed to create CLAUDE.RALPH.md: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn set_locked_project(state: State<'_, AppState>, path: String) -> Result<(), String> {
    // Validate the project path first
    validate_project_path(path.clone())?;

    // Canonicalize path (resolve symlinks)
    let canonical_path =
        std::fs::canonicalize(&path).map_err(|e| format!("Failed to resolve path: {}", e))?;

    // Check if already locked
    let mut locked = state.locked_project.lock().map_err(|e| e.to_string())?;
    if locked.is_some() {
        return Err("Project already locked for this session".to_string());
    }

    // Open the SQLite database
    let db_path = canonical_path.join(".ralph").join("db").join("ralph.db");
    let db = SqliteDb::open(&db_path)?;

    // Store the database connection
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
pub fn start_loop(
    app: AppHandle,
    state: State<'_, AppState>,
    max_iterations: u32,
) -> Result<(), String> {
    // Get locked project from state
    let locked = state.locked_project.lock().map_err(|e| e.to_string())?;
    let project_path = locked
        .as_ref()
        .ok_or("No project locked (bug, restart app)")?
        .clone();
    drop(locked);

    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine
        .start(app, project_path, max_iterations)
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
    ) -> std::io::Result<()> {
        // Early return if limits hit
        if depth > max_depth || projects.len() >= max_projects {
            return Ok(());
        }

        if !path.is_dir() {
            return Ok(());
        }

        // Check if this directory has a .ralph folder
        let ralph_dir = path.join(".ralph");
        if ralph_dir.exists() && ralph_dir.is_dir() {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string();

            projects.push(RalphProject {
                name,
                path: path.to_string_lossy().to_string(),
            });

            // Early return if we hit max projects
            if projects.len() >= max_projects {
                return Ok(());
            }
        }

        // Recursively scan subdirectories
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        let entry_path = entry.path();

                        // Check if directory should be excluded
                        if let Some(dir_name) = entry_path.file_name().and_then(|n| n.to_str()) {
                            if EXCLUDED_DIRS.contains(&dir_name) {
                                continue; // Skip this directory
                            }
                        }

                        let _ = scan_recursive(
                            &entry_path,
                            projects,
                            depth + 1,
                            max_depth,
                            max_projects,
                        );

                        // Early return if we hit max projects
                        if projects.len() >= max_projects {
                            return Ok(());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    scan_recursive(&scan_path, &mut projects, 0, MAX_SCAN_DEPTH, MAX_PROJECTS)
        .map_err(|e| e.to_string())?;

    Ok(projects)
}

#[tauri::command]
pub fn get_current_dir() -> Result<String, String> {
    // Return the default scan location (home directory)
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

/// Normalize feature name to lowercase with hyphens, reject invalid chars
fn normalize_feature_name(name: &str) -> Result<String, String> {
    // Reject slashes, colons, and other special chars
    if name.contains('/') || name.contains(':') || name.contains('\\') {
        return Err("Feature name cannot contain /, :, or \\".to_string());
    }

    // Normalize: lowercase, replace whitespace with hyphens
    Ok(name.to_lowercase().trim().replace(char::is_whitespace, "-"))
}

/// Parse priority string to Priority enum
fn parse_priority(priority: Option<&str>) -> Option<Priority> {
    priority.and_then(|p| match p {
        "low" => Some(Priority::Low),
        "medium" => Some(Priority::Medium),
        "high" => Some(Priority::High),
        "critical" => Some(Priority::Critical),
        _ => None,
    })
}

/// Parse provenance string to TaskProvenance enum
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
pub struct FeatureData {
    pub name: String,
    pub display_name: String,
    pub acronym: String,
    pub description: Option<String>,
    pub created: Option<String>,
    pub knowledge_paths: Vec<String>,
    pub context_files: Vec<String>,
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
        })
        .collect())
}

#[tauri::command]
pub fn create_feature(
    state: State<'_, AppState>,
    name: String,
    display_name: String,
    acronym: String,
    description: Option<String>,
) -> Result<(), String> {
    let guard = get_db(&state)?;
    let db = guard.as_ref().unwrap();

    // Normalize name (lowercase with hyphens)
    let normalized_name = normalize_feature_name(&name)?;

    db.create_feature(normalized_name, display_name, acronym, description)
}

#[tauri::command]
pub fn update_feature(
    state: State<'_, AppState>,
    name: String,
    display_name: String,
    acronym: String,
    description: Option<String>,
) -> Result<(), String> {
    let guard = get_db(&state)?;
    let db = guard.as_ref().unwrap();
    db.update_feature(name, display_name, acronym, description)
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

    // Normalize name (lowercase with hyphens)
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
pub fn set_task_status(
    state: State<'_, AppState>,
    id: u32,
    status: String,
) -> Result<(), String> {
    let guard = get_db(&state)?;
    let db = guard.as_ref().unwrap();
    let status = sqlite_db::TaskStatus::parse(&status)
        .ok_or_else(|| format!("Invalid status: {}", status))?;
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
        _ => return Err(format!("Invalid author: {}", author)),
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

// --- Enriched Query Commands ---

#[tauri::command]
pub fn get_enriched_tasks(
    state: State<'_, AppState>,
) -> Result<Vec<sqlite_db::EnrichedTask>, String> {
    let guard = get_db(&state)?;
    let db = guard.as_ref().unwrap();
    Ok(db.get_enriched_tasks())
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
        created: info.created.clone(),
    })
}

// --- PTY Commands ---

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
        let ralph_db = project_path.join(".ralph").join("db");
        Some(state.mcp_generator.generate(&mode, &ralph_db)?)
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
