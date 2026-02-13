use sqlite_db::{FixedClock, SqliteDb};
use std::path::{Path, PathBuf};

use crate::test_support::fixture_project;

pub(crate) fn initialize_project_for_fixture(
    path: PathBuf,
    project_title: String,
    use_undetect: bool,
) -> Result<(), String> {
    fixture_project::initialize_project_for_fixture(
        path,
        project_title,
        use_undetect,
        Some(fixed_clock()),
    )
}

pub(crate) fn fixed_clock() -> Box<dyn sqlite_db::Clock> {
    Box::new(FixedClock(
        chrono::NaiveDate::from_ymd_opt(2026, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc(),
    ))
}

pub(crate) fn open_fixture_db(fixture_path: &Path) -> SqliteDb {
    let db_path = fixture_path.join(".undetect-ralph/db/ralph.db");
    SqliteDb::open(&db_path, Some(fixed_clock())).unwrap()
}
