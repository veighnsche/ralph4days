use crate::project::validate_project_path;
use crate::project_scan;
pub use core_contracts::session::ProjectLockSetArgs;
use service_runtime::diagnostics;
use core_errors::{codes, err_string, ralph_err, RalphResult, RalphResultExt};
use data_sqlite::SqliteDb;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;

pub fn with_db<T, F>(db: &Mutex<Option<SqliteDb>>, f: F) -> RalphResult<T>
where
    F: FnOnce(&SqliteDb) -> RalphResult<T>,
{
    let guard = db
        .lock()
        .ralph_err(codes::INTERNAL, "Database mutex poisoned")?;
    let db = guard
        .as_ref()
        .ok_or_else(|| err_string(codes::PROJECT_LOCK, "No project locked (database not open)"))?;
    f(db)
}

pub fn with_db_tx<T, F>(db: &Mutex<Option<SqliteDb>>, f: F) -> RalphResult<T>
where
    F: FnOnce(&SqliteDb) -> RalphResult<T>,
{
    with_db(db, |db| db.with_transaction(f))
}

pub fn locked_project_path(locked_project: &Mutex<Option<PathBuf>>) -> RalphResult<PathBuf> {
    let guard = locked_project
        .lock()
        .ralph_err(codes::INTERNAL, "Locked project mutex poisoned")?;
    guard
        .as_ref()
        .cloned()
        .ok_or_else(|| err_string(codes::PROJECT_LOCK, "No project locked"))
}

pub fn maybe_locked_project_path(
    locked_project: &Mutex<Option<PathBuf>>,
) -> RalphResult<Option<PathBuf>> {
    let guard = locked_project
        .lock()
        .ralph_err(codes::INTERNAL, "Locked project mutex poisoned")?;
    Ok(guard.as_ref().cloned())
}

/// Locks the project and opens the database.
///
/// Returns the canonical project path on success.
pub fn project_lock_set(
    locked_project: &Mutex<Option<PathBuf>>,
    db: &Mutex<Option<SqliteDb>>,
    args: ProjectLockSetArgs,
) -> RalphResult<PathBuf> {
    validate_project_path(PathBuf::from(&args.path).as_path())?;

    let canonical_path = std::fs::canonicalize(&args.path)
        .ralph_err(codes::PROJECT_PATH, "Failed to resolve path")?;

    let mut locked = locked_project
        .lock()
        .ralph_err(codes::INTERNAL, "Locked project mutex poisoned")?;
    if locked.is_some() {
        return ralph_err!(
            codes::PROJECT_LOCK,
            "Project already locked for this session"
        );
    }

    let db_path = canonical_path.join(".ralph").join("db").join("ralph.db");
    let opened = SqliteDb::open(&db_path, None)?;

    let mut db_guard = db.lock().ralph_err(codes::INTERNAL, "DB mutex poisoned")?;
    *db_guard = Some(opened);

    *locked = Some(canonical_path.clone());
    Ok(canonical_path)
}

pub fn project_lock_set_and_record_recent(
    locked_project: &Mutex<Option<PathBuf>>,
    db: &Mutex<Option<SqliteDb>>,
    data_dir: &Path,
    args: ProjectLockSetArgs,
) -> RalphResult<PathBuf> {
    let canonical_path = project_lock_set(locked_project, db, args)?;

    let project_name = canonical_path
        .file_name()
        .map_or_else(|| "Unknown".to_owned(), |n| n.to_string_lossy().to_string());

    if let Err(error) = project_scan::recents_add(
        data_dir,
        canonical_path.to_string_lossy().to_string(),
        project_name,
    ) {
        diagnostics::emit_warning(
            "recent-projects",
            "write-failed",
            &format!("Failed to persist recent projects: {error}"),
        );
    }

    Ok(canonical_path)
}

pub fn project_lock_get(locked_project: &Mutex<Option<PathBuf>>) -> RalphResult<Option<String>> {
    let locked = maybe_locked_project_path(locked_project)?;
    Ok(locked.as_ref().map(|p| p.to_string_lossy().to_string()))
}
