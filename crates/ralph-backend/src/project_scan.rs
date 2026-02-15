use crate::project_contract::{ProjectInfo, ProjectScanArgs, RalphProject, RecentProject};
use ralph_errors::{codes, RalphResult, RalphResultExt};
use sqlite_db::SqliteDb;
use std::path::{Path, PathBuf};

const RECENT_PROJECTS_FILENAME: &str = "recent_projects.json";
const MAX_RECENT_PROJECTS: usize = 20;

const MAX_SCAN_DEPTH: usize = 5;
const MAX_PROJECTS: usize = 100;
const EXCLUDED_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    "target",
    "build",
    "dist",
    ".next",
    ".venv",
    "venv",
    "__pycache__",
    ".cache",
    "tmp",
    "temp",
    ".tmp",
    "vendor",
    ".idea",
    ".vscode",
    "Library",
    "Applications",
];

pub fn recents_load(data_dir: &Path) -> RalphResult<Vec<RecentProject>> {
    let file = data_dir.join(RECENT_PROJECTS_FILENAME);
    if !file.exists() {
        return Ok(Vec::new());
    }

    let contents = std::fs::read_to_string(&file)
        .ralph_err(codes::FILESYSTEM, "Failed to read recent projects")?;
    let projects: Vec<RecentProject> = serde_json::from_str(&contents)
        .ralph_err(codes::FILESYSTEM, "Failed to parse recent projects")?;
    Ok(projects)
}

pub fn recents_add(data_dir: &Path, path: String, name: String) -> RalphResult<()> {
    std::fs::create_dir_all(data_dir).ralph_err(
        codes::FILESYSTEM,
        "Failed to create recent projects directory",
    )?;

    let mut projects = recents_load(data_dir)?;
    projects.retain(|p| p.path != path);

    let now = chrono::Utc::now().to_rfc3339();
    projects.insert(
        0,
        RecentProject {
            path,
            name,
            last_opened: now,
        },
    );
    projects.truncate(MAX_RECENT_PROJECTS);

    let file = data_dir.join(RECENT_PROJECTS_FILENAME);
    let json = serde_json::to_string_pretty(&projects)
        .ralph_err(codes::FILESYSTEM, "Failed to serialize recent projects")?;
    std::fs::write(&file, json).ralph_err(codes::FILESYSTEM, "Failed to write recent projects")?;
    Ok(())
}

pub fn project_scan(args: ProjectScanArgs) -> RalphResult<Vec<RalphProject>> {
    let scan_path = if let Some(dir) = args.root_dir {
        PathBuf::from(dir)
    } else {
        dirs::home_dir().ok_or_else(|| {
            ralph_errors::err_string(codes::FILESYSTEM, "Failed to get home directory")
        })?
    };

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
            return;
        }

        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                if projects.len() >= max_projects {
                    return;
                }
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    if let Some(name) = entry_path.file_name().and_then(|n| n.to_str()) {
                        if EXCLUDED_DIRS.contains(&name) {
                            continue;
                        }
                    }
                    scan_recursive(&entry_path, projects, depth + 1, max_depth, max_projects);
                }
            }
        }
    }

    let mut projects = Vec::new();
    scan_recursive(&scan_path, &mut projects, 0, MAX_SCAN_DEPTH, MAX_PROJECTS);
    Ok(projects)
}

pub fn project_info_get(db: &SqliteDb) -> RalphResult<ProjectInfo> {
    let info = db.get_project_info()?;
    Ok(ProjectInfo {
        title: info.title.clone(),
        description: info.description.clone(),
        created: info.created,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn recent_projects_round_trip() {
        let dir = TempDir::new().unwrap();
        let data_dir = dir.path();

        recents_add(data_dir, "/tmp/a".to_owned(), "a".to_owned()).unwrap();
        let projects = recents_load(data_dir).unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].path, "/tmp/a");
    }

    #[test]
    fn project_scan_finds_ralph_dirs() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        let project_a = root.join("a");
        std::fs::create_dir_all(project_a.join(".ralph")).unwrap();

        let result = project_scan(ProjectScanArgs {
            root_dir: Some(root.to_string_lossy().to_string()),
        })
        .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "a");
    }
}
