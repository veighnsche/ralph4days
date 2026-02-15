use crate::project::validate_project_path;
use ralph_errors::{codes, err_string, ralph_err, RalphResultExt, ToStringErr};
use ralph_macros::ipc_type;
use sqlite_db::SqliteDb;
use std::path::PathBuf;
use std::sync::Mutex;

#[ipc_type]
pub struct ProjectLockSetArgs {
    pub path: String,
}

pub fn with_db<T, F>(db: &Mutex<Option<SqliteDb>>, f: F) -> Result<T, String>
where
    F: FnOnce(&SqliteDb) -> Result<T, String>,
{
    let guard = db.lock().err_str(codes::INTERNAL)?;
    let db = guard
        .as_ref()
        .ok_or_else(|| err_string(codes::PROJECT_LOCK, "No project locked (database not open)"))?;
    f(db)
}

pub fn with_db_tx<T, F>(db: &Mutex<Option<SqliteDb>>, f: F) -> Result<T, String>
where
    F: FnOnce(&SqliteDb) -> Result<T, String>,
{
    with_db(db, |db| db.with_transaction(f))
}

pub fn locked_project_path(locked_project: &Mutex<Option<PathBuf>>) -> Result<PathBuf, String> {
    let guard = locked_project.lock().err_str(codes::INTERNAL)?;
    guard
        .as_ref()
        .cloned()
        .ok_or_else(|| err_string(codes::PROJECT_LOCK, "No project locked"))
}

pub fn maybe_locked_project_path(
    locked_project: &Mutex<Option<PathBuf>>,
) -> Result<Option<PathBuf>, String> {
    let guard = locked_project.lock().err_str(codes::INTERNAL)?;
    Ok(guard.as_ref().cloned())
}

/// Locks the project and opens the database.
///
/// Returns the canonical project path on success.
pub fn project_lock_set(
    locked_project: &Mutex<Option<PathBuf>>,
    db: &Mutex<Option<SqliteDb>>,
    args: ProjectLockSetArgs,
) -> Result<PathBuf, String> {
    validate_project_path(PathBuf::from(&args.path).as_path())?;

    let canonical_path = std::fs::canonicalize(&args.path)
        .ralph_err(codes::PROJECT_PATH, "Failed to resolve path")?;

    let mut locked = locked_project.lock().err_str(codes::INTERNAL)?;
    if locked.is_some() {
        return ralph_err!(
            codes::PROJECT_LOCK,
            "Project already locked for this session"
        );
    }

    let db_path = canonical_path.join(".ralph").join("db").join("ralph.db");
    let opened = SqliteDb::open(&db_path, None)?;

    let mut db_guard = db.lock().err_str(codes::INTERNAL)?;
    *db_guard = Some(opened);

    *locked = Some(canonical_path.clone());
    Ok(canonical_path)
}

pub fn project_lock_get(locked_project: &Mutex<Option<PathBuf>>) -> Result<Option<String>, String> {
    let locked = maybe_locked_project_path(locked_project)?;
    Ok(locked.as_ref().map(|p| p.to_string_lossy().to_string()))
}
