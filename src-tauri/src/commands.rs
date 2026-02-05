use crate::loop_engine::LoopEngine;
use crate::types::LoopStatus;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, State};

// Recursive scan configuration
const MAX_SCAN_DEPTH: usize = 5; // Max 5 levels deep
const MAX_PROJECTS: usize = 100; // Max 100 projects returned
const EXCLUDED_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    "target",       // Rust builds
    "build",
    "dist",
    ".next",        // Next.js
    ".venv",        // Python venv
    "venv",
    "__pycache__",
    ".cache",
    "tmp",
    "temp",
    ".tmp",
    "vendor",       // Go/PHP dependencies
    ".idea",        // IDEs
    ".vscode",
    "Library",      // macOS system
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
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            engine: Mutex::new(LoopEngine::new()),
            locked_project: Mutex::new(None),
        }
    }
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
            "No .ralph/ folder. Create one:\n  mkdir -p {}/.ralph\n  # Then add prd.yaml inside",
            path.display()
        ));
    }
    if !ralph_dir.is_dir() {
        return Err(format!("{} exists but is not a directory", ralph_dir.display()));
    }

    // Check .ralph/prd.yaml exists
    let prd_path = ralph_dir.join("prd.yaml");
    if !prd_path.exists() {
        return Err(format!(
            "No prd.yaml found. Create one:\n  # See: .specs/035_PRD_FORMAT.md\n  # Or copy from: fixtures/single-task/.ralph/prd.yaml"
        ));
    }
    if !prd_path.is_file() {
        return Err(format!("{} exists but is not a file", prd_path.display()));
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

    // Create template prd.yaml
    let prd_path = ralph_dir.join("prd.yaml");
    let prd_template = format!(
        r#"schema_version: "1.0"
project:
  title: "{}"
  description: "Add project description here"
  created: "{}"

tasks:
  - id: "task-001"
    title: "Replace this with your first task"
    description: "Add task details here"
    status: "pending"
    priority: "medium"
    tags: []
    created: "{}"
"#,
        project_title,
        chrono::Utc::now().format("%Y-%m-%d"),
        chrono::Utc::now().format("%Y-%m-%d")
    );

    std::fs::write(&prd_path, prd_template)
        .map_err(|e| format!("Failed to create prd.yaml: {}", e))?;

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
    let canonical_path = std::fs::canonicalize(&path)
        .map_err(|e| format!("Failed to resolve path: {}", e))?;

    // Check if already locked
    let mut locked = state.locked_project.lock().map_err(|e| e.to_string())?;
    if locked.is_some() {
        return Err("Project already locked for this session".to_string());
    }

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
    let project_path = locked.as_ref()
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

                        let _ = scan_recursive(&entry_path, projects, depth + 1, max_depth, max_projects);

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

    // If we hit max results, add a note
    if projects.len() >= MAX_PROJECTS {
        return Err(format!(
            "Found {} projects (max limit). Scan stopped at {} levels deep. Consider narrowing search.",
            MAX_PROJECTS,
            MAX_SCAN_DEPTH
        ));
    }

    Ok(projects)
}

#[tauri::command]
pub fn get_current_dir() -> Result<String, String> {
    // Return the default scan location (home directory)
    let path = dirs::home_dir()
        .ok_or("Failed to get home directory")?;
    Ok(path.to_string_lossy().to_string())
}
