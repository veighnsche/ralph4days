//! Transformation tests for yaml-db crate
//!
//! These tests verify that operations correctly transform database files from
//! one state to another. Each test:
//! 1. Copies a "before" snapshot to a temp directory
//! 2. Performs operations that transform the state
//! 3. Compares the result with an "after" snapshot
//! 4. Cleans up temporary files

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use yaml_db::{Priority, TaskInput, YamlDatabase};

/// Helper to set up test with before snapshots
fn setup_from_snapshot(snapshot_name: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("db");
    fs::create_dir_all(&db_path).unwrap();

    // Copy all before snapshots to temp directory
    let snapshot_dir = PathBuf::from("tests/snapshots/transformations");
    let before_dir = snapshot_dir.join(snapshot_name).join("before");

    if before_dir.exists() {
        for entry in fs::read_dir(&before_dir).unwrap() {
            let entry = entry.unwrap();
            let file_name = entry.file_name();
            let dest = db_path.join(&file_name);
            fs::copy(entry.path(), dest).unwrap();
        }
    }

    (temp_dir, db_path)
}

/// Helper to compare current state with after snapshot
fn assert_matches_snapshot(db_path: &PathBuf, snapshot_name: &str) {
    let snapshot_dir = PathBuf::from("tests/snapshots/transformations");
    let after_dir = snapshot_dir.join(snapshot_name).join("after");

    // Compare each file
    for file_name in &["tasks.yaml", "features.yaml", "disciplines.yaml", "metadata.yaml"] {
        let actual_path = db_path.join(file_name);
        let expected_path = after_dir.join(file_name);

        if expected_path.exists() {
            let actual = fs::read_to_string(&actual_path)
                .unwrap_or_else(|_| panic!("Failed to read {}", actual_path.display()));
            let expected = fs::read_to_string(&expected_path)
                .unwrap_or_else(|_| panic!("Failed to read {}", expected_path.display()));

            if actual.trim() != expected.trim() {
                eprintln!("=== ACTUAL {} ===", file_name);
                eprintln!("{}", actual);
                eprintln!("=== EXPECTED {} ===", file_name);
                eprintln!("{}", expected);
                panic!("File {} doesn't match snapshot", file_name);
            }
        }
    }
}

#[test]
fn test_transform_empty_db_to_first_task() {
    let (_temp, db_path) = setup_from_snapshot("empty_to_first_task");

    // Perform transformation: create first task
    let mut db = YamlDatabase::from_path(db_path.clone()).unwrap();
    db.create_feature("authentication".to_string(), "Authentication".to_string(), "AUTH".to_string(), None).unwrap();
    let task_id = db
        .create_task(TaskInput {
            feature: "authentication".to_string(),
            discipline: "backend".to_string(),
            title: "Implement login API".to_string(),
            description: Some("Create REST API endpoints for user authentication".to_string()),
            priority: Some(Priority::High),
            tags: vec!["api".to_string(), "security".to_string()],
            depends_on: vec![],
            acceptance_criteria: Some(vec![
                "POST /login endpoint works".to_string(),
                "Returns JWT token".to_string(),
            ]),
            ..Default::default()
        })
        .unwrap();

    assert_eq!(task_id, 1);

    // Compare with after snapshot
    assert_matches_snapshot(&db_path, "empty_to_first_task");
}

#[test]
fn test_transform_add_task_with_dependency() {
    let (_temp, db_path) = setup_from_snapshot("add_dependent_task");

    // Load existing database with one task (authentication feature already in before snapshot)
    let mut db = YamlDatabase::from_path(db_path.clone()).unwrap();

    // Add second task that depends on first
    let task_id = db
        .create_task(TaskInput {
            feature: "authentication".to_string(),
            discipline: "frontend".to_string(),
            title: "Build login form".to_string(),
            description: Some("Create UI for user login".to_string()),
            priority: Some(Priority::Medium),
            tags: vec!["ui".to_string()],
            depends_on: vec![1], // Depends on existing task 1
            acceptance_criteria: Some(vec!["Form validates input".to_string()]),
            ..Default::default()
        })
        .unwrap();

    assert_eq!(task_id, 2);

    // Compare with after snapshot
    assert_matches_snapshot(&db_path, "add_dependent_task");
}

#[test]
fn test_transform_multiple_tasks_multiple_features() {
    let (_temp, db_path) = setup_from_snapshot("multi_feature_expansion");

    // authentication feature already in before snapshot
    let mut db = YamlDatabase::from_path(db_path.clone()).unwrap();

    // Add tasks across multiple features
    db.create_task(TaskInput {
        feature: "authentication".to_string(),
        discipline: "backend".to_string(),
        title: "Add password reset".to_string(),
        description: None,
        priority: Some(Priority::Medium),
        tags: vec![],
        depends_on: vec![],
        acceptance_criteria: None,
        ..Default::default()
    })
    .unwrap();

    db.create_feature("user-profile".to_string(), "User Profile".to_string(), "USPR".to_string(), None).unwrap();

    db.create_task(TaskInput {
        feature: "user-profile".to_string(),
        discipline: "frontend".to_string(),
        title: "Create profile page".to_string(),
        description: None,
        priority: Some(Priority::Low),
        tags: vec![],
        depends_on: vec![],
        acceptance_criteria: None,
        ..Default::default()
    })
    .unwrap();

    db.create_task(TaskInput {
        feature: "user-profile".to_string(),
        discipline: "backend".to_string(),
        title: "Profile API endpoints".to_string(),
        description: None,
        priority: Some(Priority::Medium),
        tags: vec![],
        depends_on: vec![],
        acceptance_criteria: None,
        ..Default::default()
    })
    .unwrap();

    // Compare with after snapshot
    assert_matches_snapshot(&db_path, "multi_feature_expansion");
}

#[test]
fn test_transform_counter_rebuild() {
    let (_temp, db_path) = setup_from_snapshot("counter_rebuild");

    let mut db = YamlDatabase::from_path(db_path.clone()).unwrap();

    // Counters should be wrong in before snapshot
    // Adding new tasks and rebuilding should fix them
    db.rebuild_counters();
    db.save_all().unwrap();

    // Compare with after snapshot (should have correct counters)
    assert_matches_snapshot(&db_path, "counter_rebuild");
}

#[test]
fn test_transform_custom_discipline_addition() {
    let (_temp, db_path) = setup_from_snapshot("custom_discipline");

    let mut db = YamlDatabase::from_path(db_path.clone()).unwrap();

    // Create feature and custom discipline before creating the task
    db.create_feature("ml-pipeline".to_string(), "ML Pipeline".to_string(), "MLPP".to_string(), None).unwrap();
    db.create_discipline(
        "machine-learning".to_string(),
        "Machine Learning".to_string(),
        "MLRN".to_string(),
        "Brain".to_string(),
        "violet".to_string(),
    ).unwrap();

    db.create_task(TaskInput {
        feature: "ml-pipeline".to_string(),
        discipline: "machine-learning".to_string(),
        title: "Train model".to_string(),
        description: Some("Train ML model on dataset".to_string()),
        priority: Some(Priority::High),
        tags: vec!["ml".to_string(), "training".to_string()],
        depends_on: vec![],
        acceptance_criteria: None,
        ..Default::default()
    })
    .unwrap();

    // Compare with after snapshot (should include new discipline)
    assert_matches_snapshot(&db_path, "custom_discipline");
}

#[test]
fn test_transform_reload_and_modify() {
    let (_temp, db_path) = setup_from_snapshot("reload_modify");

    // First operation: create a task
    {
        let mut db = YamlDatabase::from_path(db_path.clone()).unwrap();
        db.create_feature("search".to_string(), "Search".to_string(), "SRCH".to_string(), None).unwrap();
        db.create_task(TaskInput {
            feature: "search".to_string(),
            discipline: "backend".to_string(),
            title: "Implement search API".to_string(),
            description: None,
            priority: Some(Priority::High),
            tags: vec![],
            depends_on: vec![],
            acceptance_criteria: None,
            ..Default::default()
        })
        .unwrap();
    }

    // Second operation: reload and create another task
    {
        let mut db = YamlDatabase::from_path(db_path.clone()).unwrap();
        db.create_task(TaskInput {
            feature: "search".to_string(),
            discipline: "frontend".to_string(),
            title: "Search UI component".to_string(),
            description: None,
            priority: Some(Priority::Medium),
            tags: vec![],
            depends_on: vec![1],
            acceptance_criteria: None,
            ..Default::default()
        })
        .unwrap();
    }

    // Compare with after snapshot
    assert_matches_snapshot(&db_path, "reload_modify");
}
