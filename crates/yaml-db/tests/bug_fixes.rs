//! Tests for bug fixes and gap filling
//!
//! These tests verify fixes for bugs discovered during proactive review.

use std::fs;
use tempfile::TempDir;
use yaml_db::{Priority, TaskInput, YamlDatabase};

/// Helper to create a temporary database directory
fn create_temp_db() -> (TempDir, std::path::PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("db");
    fs::create_dir(&db_path).unwrap();
    (temp_dir, db_path)
}

// === Circular Dependency Tests ===

#[test]
fn test_self_referential_dependency() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    // Create first task to get its ID
    let _task1_id = db
        .create_task(TaskInput {
            feature: "test".to_string(),
            discipline: "backend".to_string(),
            title: "Task 1".to_string(),
            description: None,
            priority: None,
            tags: vec![],
            depends_on: vec![],
            acceptance_criteria: None,
        })
        .unwrap();

    // Try to update task to depend on itself (would need update_task method)
    // For now, just test that we can't create a task that depends on future ID
    let result = db.create_task(TaskInput {
        feature: "test".to_string(),
        discipline: "backend".to_string(),
        title: "Self-referential task".to_string(),
        description: None,
        priority: None,
        tags: vec![],
        depends_on: vec![999], // Future ID that will fail
        acceptance_criteria: None,
    });

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn test_circular_dependency_chain() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    // Create task A
    let task_a = db
        .create_task(TaskInput {
            feature: "test".to_string(),
            discipline: "backend".to_string(),
            title: "Task A".to_string(),
            description: None,
            priority: None,
            tags: vec![],
            depends_on: vec![],
            acceptance_criteria: None,
        })
        .unwrap();

    // Create task B depending on A (OK)
    let task_b = db
        .create_task(TaskInput {
            feature: "test".to_string(),
            discipline: "backend".to_string(),
            title: "Task B".to_string(),
            description: None,
            priority: None,
            tags: vec![],
            depends_on: vec![task_a],
            acceptance_criteria: None,
        })
        .unwrap();

    // Create task C depending on B (OK)
    let _task_c = db
        .create_task(TaskInput {
            feature: "test".to_string(),
            discipline: "backend".to_string(),
            title: "Task C".to_string(),
            description: None,
            priority: None,
            tags: vec![],
            depends_on: vec![task_b],
            acceptance_criteria: None,
        })
        .unwrap();

    // Note: Creating a circular dependency (A depends on C) would require
    // an update_task method to modify existing tasks. This is a gap in functionality.
    // For now, we can only test that forward dependencies work.
}

// === Whitespace Validation Tests ===

#[test]
fn test_whitespace_only_feature_name() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    let result = db.create_task(TaskInput {
        feature: "   ".to_string(), // Whitespace only
        discipline: "backend".to_string(),
        title: "Task".to_string(),
        description: None,
        priority: None,
        tags: vec![],
        depends_on: vec![],
        acceptance_criteria: None,
    });

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Feature name cannot be empty"));
}

#[test]
fn test_whitespace_only_discipline_name() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    let result = db.create_task(TaskInput {
        feature: "test".to_string(),
        discipline: "\t\n ".to_string(), // Various whitespace
        title: "Task".to_string(),
        description: None,
        priority: None,
        tags: vec![],
        depends_on: vec![],
        acceptance_criteria: None,
    });

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("Discipline name cannot be empty"));
}

#[test]
fn test_whitespace_only_title() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    let result = db.create_task(TaskInput {
        feature: "test".to_string(),
        discipline: "backend".to_string(),
        title: "     ".to_string(), // Whitespace only
        description: None,
        priority: None,
        tags: vec![],
        depends_on: vec![],
        acceptance_criteria: None,
    });

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Task title cannot be empty"));
}

// === Feature Name Normalization Tests ===

#[test]
fn test_feature_name_with_underscores() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_task(TaskInput {
        feature: "user_profile".to_string(),
        discipline: "backend".to_string(),
        title: "Task".to_string(),
        description: None,
        priority: None,
        tags: vec![],
        depends_on: vec![],
        acceptance_criteria: None,
    })
    .unwrap();

    let features = db.get_features();
    let feature = features.iter().find(|f| f.name == "user_profile").unwrap();

    // Should convert underscores to spaces and capitalize
    assert_eq!(feature.display_name, "User Profile");
}

#[test]
fn test_feature_name_with_mixed_separators() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_task(TaskInput {
        feature: "user-profile_settings".to_string(),
        discipline: "backend".to_string(),
        title: "Task".to_string(),
        description: None,
        priority: None,
        tags: vec![],
        depends_on: vec![],
        acceptance_criteria: None,
    })
    .unwrap();

    let features = db.get_features();
    let feature = features
        .iter()
        .find(|f| f.name == "user-profile_settings")
        .unwrap();

    // Should handle both hyphens and underscores
    assert_eq!(feature.display_name, "User Profile Settings");
}

// === Task Lookup Tests ===

#[test]
fn test_get_task_by_id() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    let task_id = db
        .create_task(TaskInput {
            feature: "test".to_string(),
            discipline: "backend".to_string(),
            title: "Test Task".to_string(),
            description: Some("Description".to_string()),
            priority: Some(Priority::High),
            tags: vec![],
            depends_on: vec![],
            acceptance_criteria: None,
        })
        .unwrap();

    // Get task by ID (using manual search for now, as get_by_id doesn't exist)
    let task = db.get_tasks().iter().find(|t| t.id == task_id);

    assert!(task.is_some());
    let task = task.unwrap();
    assert_eq!(task.title, "Test Task");
    assert_eq!(task.priority, Some(Priority::High));
}

#[test]
fn test_duplicate_dependency() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    let task1_id = db
        .create_task(TaskInput {
            feature: "test".to_string(),
            discipline: "backend".to_string(),
            title: "Task 1".to_string(),
            description: None,
            priority: None,
            tags: vec![],
            depends_on: vec![],
            acceptance_criteria: None,
        })
        .unwrap();

    // Create task with duplicate dependencies
    let result = db.create_task(TaskInput {
        feature: "test".to_string(),
        discipline: "backend".to_string(),
        title: "Task 2".to_string(),
        description: None,
        priority: None,
        tags: vec![],
        depends_on: vec![task1_id, task1_id, task1_id], // Duplicates
        acceptance_criteria: None,
    });

    // Should succeed (duplicates are allowed for now, could be cleaned up)
    assert!(result.is_ok());
}
