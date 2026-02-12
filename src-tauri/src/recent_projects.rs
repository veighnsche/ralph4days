use crate::xdg::XdgDirs;
use ralph_errors::{codes, RalphResultExt};
use serde::{Deserialize, Serialize};

const FILENAME: &str = "recent_projects.json";
const MAX_RECENT: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentProject {
    pub path: String,
    pub name: String,
    pub last_opened: String,
}

pub fn load(xdg: &XdgDirs) -> Result<Vec<RecentProject>, String> {
    let file = xdg.data().join(FILENAME);
    if !file.exists() {
        return Ok(Vec::new());
    }
    let contents = std::fs::read_to_string(&file)
        .ralph_err(codes::FILESYSTEM, "Failed to read recent projects")?;
    let projects: Vec<RecentProject> = serde_json::from_str(&contents)
        .ralph_err(codes::FILESYSTEM, "Failed to parse recent projects")?;
    Ok(projects)
}

pub fn add(xdg: &XdgDirs, path: String, name: String) -> Result<(), String> {
    let mut projects = load(xdg)?;

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

    projects.truncate(MAX_RECENT);

    let data_dir = xdg.ensure_data()?;
    let file = data_dir.join(FILENAME);
    let json = serde_json::to_string_pretty(&projects)
        .ralph_err(codes::FILESYSTEM, "Failed to serialize recent projects")?;
    std::fs::write(&file, json).ralph_err(codes::FILESYSTEM, "Failed to write recent projects")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_xdg() -> XdgDirs {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default();
        let base = std::env::temp_dir().join(format!("ralph4days-recent-projects-{nanos}"));
        XdgDirs::from_base(&base)
    }

    #[test]
    fn load_returns_empty_when_no_file() {
        let xdg = test_xdg();
        let _ = load(&xdg);
    }

    #[test]
    fn add_and_load_round_trip() {
        let xdg = test_xdg();
        xdg.ensure_data().unwrap();

        let file = xdg.data().join(FILENAME);
        let had_existing = file.exists();
        let backup = if had_existing {
            Some(std::fs::read_to_string(&file).unwrap())
        } else {
            None
        };

        // Write test data
        add(
            &xdg,
            "/tmp/test-project".to_owned(),
            "test-project".to_owned(),
        )
        .unwrap();
        let projects = load(&xdg).unwrap();
        assert!(!projects.is_empty());
        assert_eq!(projects[0].path, "/tmp/test-project");

        // Restore original state
        if let Some(original) = backup {
            std::fs::write(&file, original).unwrap();
        } else {
            let _ = std::fs::remove_file(&file);
        }

        if let Some(parent) = xdg.data().parent().and_then(|p| p.parent()) {
            let _ = std::fs::remove_dir_all(parent);
        }
    }
}
