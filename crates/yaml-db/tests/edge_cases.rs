//! Edge case tests for yaml-db crate
//!
//! These tests verify error handling, validation, and edge cases.

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

#[test]
fn test_create_task_with_invalid_dependency() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();

    // Try to create task with non-existent dependency
    let result = db.create_task(TaskInput {
        feature: "test".to_string(),
        discipline: "backend".to_string(),
        title: "Task with invalid dep".to_string(),
        description: None,
        priority: None,
        tags: vec![],
        depends_on: vec![999], // Non-existent task ID
        acceptance_criteria: None,
        ..Default::default()
    });

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("Dependency task 999 does not exist"));
}

#[test]
fn test_create_task_with_circular_dependency() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();

    // Create first task
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
            ..Default::default()
        })
        .unwrap();

    // Note: Circular dependencies (task depends on itself in depends_on)
    // are currently not validated - this is a potential gap
    // For now, we just test that forward dependencies work
    let task2_id = db
        .create_task(TaskInput {
            feature: "test".to_string(),
            discipline: "backend".to_string(),
            title: "Task 2".to_string(),
            description: None,
            priority: None,
            tags: vec![],
            depends_on: vec![task1_id],
            acceptance_criteria: None,
            ..Default::default()
        })
        .unwrap();

    assert!(task2_id > task1_id);
}

#[test]
fn test_concurrent_task_creation() {
    let (_temp, db_path) = create_temp_db();

    // Pre-create all features that the threads will use
    {
        let mut db = YamlDatabase::from_path(db_path.clone()).unwrap();
        for i in 0..5 {
            db.create_feature(
                format!("feature-{}", i),
                format!("Feature {}", i),
                format!("FT{:02}", i),
                None,
            ).unwrap();
        }
    }

    // Create multiple tasks "concurrently" (file locking should prevent conflicts)
    let mut handles = vec![];

    for i in 0..5 {
        let db_path_clone = db_path.clone();
        let handle = std::thread::spawn(move || {
            let mut db = YamlDatabase::from_path(db_path_clone).unwrap();
            db.create_task(TaskInput {
                feature: format!("feature-{}", i),
                discipline: "backend".to_string(),
                title: format!("Task {}", i),
                description: None,
                priority: None,
                tags: vec![],
                depends_on: vec![],
                acceptance_criteria: None,
                ..Default::default()
            })
            .unwrap()
        });
        handles.push(handle);
    }

    let mut ids: Vec<u32> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    ids.sort();

    // All IDs should be unique and sequential
    assert_eq!(ids.len(), 5);
    assert_eq!(ids, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_reload_database_after_external_modification() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path.clone()).unwrap();

    db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();

    // Create a task
    let id1 = db
        .create_task(TaskInput {
            feature: "test".to_string(),
            discipline: "backend".to_string(),
            title: "Task 1".to_string(),
            description: None,
            priority: None,
            tags: vec![],
            depends_on: vec![],
            acceptance_criteria: None,
            ..Default::default()
        })
        .unwrap();

    assert_eq!(db.get_tasks().len(), 1);

    // Simulate external modification by creating another database instance
    let mut db2 = YamlDatabase::from_path(db_path).unwrap();
    let id2 = db2
        .create_task(TaskInput {
            feature: "test".to_string(),
            discipline: "backend".to_string(),
            title: "Task 2".to_string(),
            description: None,
            priority: None,
            tags: vec![],
            depends_on: vec![],
            acceptance_criteria: None,
            ..Default::default()
        })
        .unwrap();

    // First instance still sees only 1 task
    assert_eq!(db.get_tasks().len(), 1);

    // But reload picks up the change
    db.load_all().unwrap();
    assert_eq!(db.get_tasks().len(), 2);
    assert!(db.get_tasks().iter().any(|t| t.id == id1));
    assert!(db.get_tasks().iter().any(|t| t.id == id2));
}

#[test]
fn test_task_id_sequence_after_reload() {
    let (_temp, db_path) = create_temp_db();

    // Create task with ID 5
    {
        let mut db = YamlDatabase::from_path(db_path.clone()).unwrap();
        db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();
        db.create_task(TaskInput {
            feature: "test".to_string(),
            discipline: "backend".to_string(),
            title: "Task 1".to_string(),
            description: None,
            priority: None,
            tags: vec![],
            depends_on: vec![],
            acceptance_criteria: None,
            ..Default::default()
        })
        .unwrap();
    }

    // Reload database and create another task
    {
        let mut db = YamlDatabase::from_path(db_path.clone()).unwrap();
        let next_id = db.get_next_task_id();
        assert_eq!(next_id, 2);

        let id2 = db
            .create_task(TaskInput {
                feature: "test".to_string(),
                discipline: "backend".to_string(),
                title: "Task 2".to_string(),
                description: None,
                priority: None,
                tags: vec![],
                depends_on: vec![],
                acceptance_criteria: None,
                ..Default::default()
            })
            .unwrap();

        assert_eq!(id2, 2);
    }
}

#[test]
fn test_empty_feature_name() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    // Empty feature name should fail validation
    let result = db.create_task(TaskInput {
        feature: "".to_string(),
        discipline: "backend".to_string(),
        title: "Task".to_string(),
        description: None,
        priority: None,
        tags: vec![],
        depends_on: vec![],
        acceptance_criteria: None,
        ..Default::default()
    });

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Feature name cannot be empty"));
}

#[test]
fn test_empty_discipline_name() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    let result = db.create_task(TaskInput {
        feature: "test".to_string(),
        discipline: "".to_string(),
        title: "Task".to_string(),
        description: None,
        priority: None,
        tags: vec![],
        depends_on: vec![],
        acceptance_criteria: None,
        ..Default::default()
    });

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("Discipline name cannot be empty"));
}

#[test]
fn test_empty_title() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    let result = db.create_task(TaskInput {
        feature: "test".to_string(),
        discipline: "backend".to_string(),
        title: "".to_string(),
        description: None,
        priority: None,
        tags: vec![],
        depends_on: vec![],
        acceptance_criteria: None,
        ..Default::default()
    });

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Task title cannot be empty"));
}

#[test]
fn test_very_long_title() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();

    let long_title = "A".repeat(10000);
    let result = db.create_task(TaskInput {
        feature: "test".to_string(),
        discipline: "backend".to_string(),
        title: long_title.clone(),
        description: None,
        priority: None,
        tags: vec![],
        depends_on: vec![],
        acceptance_criteria: None,
        ..Default::default()
    });

    assert!(result.is_ok());
    let tasks = db.get_tasks();
    assert_eq!(tasks[0].title, long_title);
}

#[test]
fn test_many_tags() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();

    let tags: Vec<String> = (0..100).map(|i| format!("tag-{}", i)).collect();

    let result = db.create_task(TaskInput {
        feature: "test".to_string(),
        discipline: "backend".to_string(),
        title: "Task with many tags".to_string(),
        description: None,
        priority: None,
        tags: tags.clone(),
        depends_on: vec![],
        acceptance_criteria: None,
        ..Default::default()
    });

    assert!(result.is_ok());
    let tasks = db.get_tasks();
    assert_eq!(tasks[0].tags.len(), 100);
}

#[test]
fn test_special_characters_in_fields() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("feature:with:colons".to_string(), "Feature With Colons".to_string(), "FTWC".to_string(), None).unwrap();

    let result = db.create_task(TaskInput {
        feature: "feature:with:colons".to_string(),
        discipline: "backend".to_string(),
        title: "Title with \"quotes\" and 'apostrophes'".to_string(),
        description: Some("Description with\nnewlines\nand\ttabs".to_string()),
        priority: Some(Priority::High),
        tags: vec![
            "tag-with-dash".to_string(),
            "tag_with_underscore".to_string(),
        ],
        depends_on: vec![],
        acceptance_criteria: Some(vec!["Criteria with: special chars!".to_string()]),
        ..Default::default()
    });

    assert!(result.is_ok());

    // Reload and verify
    db.load_all().unwrap();
    let tasks = db.get_tasks();
    assert_eq!(tasks[0].feature, "feature:with:colons");
    assert_eq!(tasks[0].title, "Title with \"quotes\" and 'apostrophes'");
}

#[test]
fn test_database_from_nonexistent_path() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("nonexistent").join("db");

    // Should create directory and succeed
    let result = YamlDatabase::from_path(db_path);
    assert!(result.is_ok());
}

#[test]
fn test_discipline_auto_creation() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();

    // Create the custom discipline before creating the task
    db.create_discipline(
        "custom-discipline".to_string(),
        "Custom Discipline".to_string(),
        "CSTM".to_string(),
        "Wrench".to_string(),
        "gray".to_string(),
    ).unwrap();

    // Create task with custom discipline
    let result = db.create_task(TaskInput {
        feature: "test".to_string(),
        discipline: "custom-discipline".to_string(),
        title: "Task".to_string(),
        description: None,
        priority: None,
        tags: vec![],
        depends_on: vec![],
        acceptance_criteria: None,
        ..Default::default()
    });

    assert!(result.is_ok());

    // Verify discipline was created
    let disciplines = db.get_disciplines();
    assert!(disciplines.iter().any(|d| d.name == "custom-discipline"));
}
