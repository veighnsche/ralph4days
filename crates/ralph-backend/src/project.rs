use ralph_errors::{codes, ralph_err};
use std::path::Path;

pub fn validate_project_path(path: &Path) -> Result<(), String> {
    tracing::debug!(path = %path.display(), "Validating project path");

    if !path.exists() {
        tracing::error!(path = %path.display(), "Directory not found");
        return ralph_err!(
            codes::PROJECT_PATH,
            "Directory not found: {}",
            path.display()
        );
    }
    if !path.is_dir() {
        return ralph_err!(codes::PROJECT_PATH, "Not a directory: {}", path.display());
    }

    let ralph_dir = path.join(".ralph");
    if !ralph_dir.exists() {
        return ralph_err!(
            codes::PROJECT_PATH,
            "No .ralph/ folder. Initialize with:\n  ralph --init \"{}\"",
            path.display()
        );
    }
    if !ralph_dir.is_dir() {
        return ralph_err!(
            codes::PROJECT_PATH,
            "{} exists but is not a directory",
            ralph_dir.display()
        );
    }

    let db_file = ralph_dir.join("db").join("ralph.db");
    if !db_file.exists() {
        tracing::error!(path = %path.display(), "No .ralph/db/ralph.db found");
        return ralph_err!(
            codes::PROJECT_PATH,
            "No .ralph/db/ralph.db found. Initialize with:\n  ralph --init \"{}\"",
            path.display()
        );
    }

    tracing::info!(path = %path.display(), "Project path validated successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn validate_project_path_errors_when_missing_directory() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("missing");
        let err = validate_project_path(&missing).unwrap_err();
        assert!(err.contains("[R-1000]"));
        assert!(err.contains("Directory not found"));
    }

    #[test]
    fn validate_project_path_errors_when_not_directory() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("file");
        std::fs::write(&file, "x").unwrap();
        let err = validate_project_path(&file).unwrap_err();
        assert!(err.contains("[R-1000]"));
        assert!(err.contains("Not a directory"));
    }

    #[test]
    fn validate_project_path_errors_when_missing_ralph_dir() {
        let dir = tempdir().unwrap();
        let err = validate_project_path(dir.path()).unwrap_err();
        assert!(err.contains("[R-1000]"));
        assert!(err.contains("No .ralph/ folder"));
    }

    #[test]
    fn validate_project_path_errors_when_missing_db_file() {
        let dir = tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".ralph")).unwrap();
        let err = validate_project_path(dir.path()).unwrap_err();
        assert!(err.contains("[R-1000]"));
        assert!(err.contains("No .ralph/db/ralph.db found"));
    }

    #[test]
    fn validate_project_path_ok_when_db_file_exists() {
        let dir = tempdir().unwrap();
        let db_dir = dir.path().join(".ralph").join("db");
        std::fs::create_dir_all(&db_dir).unwrap();
        std::fs::write(db_dir.join("ralph.db"), "").unwrap();
        validate_project_path(dir.path()).unwrap();
    }
}
