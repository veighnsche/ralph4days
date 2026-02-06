//! CRUD operation tests for yaml-db crate
//!
//! Tests for Create, Read, Update, Delete operations on tasks.

use std::fs;
use tempfile::TempDir;
use yaml_db::{Priority, TaskInput, TaskStatus, YamlDatabase};

/// Helper to create a temporary database directory
fn create_temp_db() -> (TempDir, std::path::PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("db");
    fs::create_dir(&db_path).unwrap();
    (temp_dir, db_path)
}

// === CREATE tests (already covered in other files, but added for completeness) ===

#[test]
fn test_create_task() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("auth".to_string(), "Auth".to_string(), "AUTH".to_string(), None).unwrap();

    let task_id = db
        .create_task(TaskInput {
            feature: "auth".to_string(),
            discipline: "backend".to_string(),
            title: "Implement login".to_string(),
            description: Some("Login API".to_string()),
            priority: Some(Priority::High),
            tags: vec!["api".to_string()],
            depends_on: vec![],
            acceptance_criteria: Some(vec!["Works".to_string()]),
        })
        .unwrap();

    assert_eq!(task_id, 1);
    assert_eq!(db.get_tasks().len(), 1);
}

// === READ tests ===

#[test]
fn test_get_task_by_id_exists() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();

    let task_id = db
        .create_task(TaskInput {
            feature: "test".to_string(),
            discipline: "backend".to_string(),
            title: "Test Task".to_string(),
            description: None,
            priority: Some(Priority::Medium),
            tags: vec![],
            depends_on: vec![],
            acceptance_criteria: None,
        })
        .unwrap();

    let task = db.get_task_by_id(task_id);
    assert!(task.is_some());

    let task = task.unwrap();
    assert_eq!(task.id, task_id);
    assert_eq!(task.title, "Test Task");
    assert_eq!(task.priority, Some(Priority::Medium));
}

#[test]
fn test_get_task_by_id_not_exists() {
    let (_temp, db_path) = create_temp_db();
    let db = YamlDatabase::from_path(db_path).unwrap();

    let task = db.get_task_by_id(999);
    assert!(task.is_none());
}

// === UPDATE tests ===

#[test]
fn test_update_task() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("auth".to_string(), "Auth".to_string(), "AUTH".to_string(), None).unwrap();

    // Create initial task
    let task_id = db
        .create_task(TaskInput {
            feature: "auth".to_string(),
            discipline: "backend".to_string(),
            title: "Old Title".to_string(),
            description: Some("Old description".to_string()),
            priority: Some(Priority::Low),
            tags: vec!["old".to_string()],
            depends_on: vec![],
            acceptance_criteria: None,
        })
        .unwrap();

    // Update the task
    db.update_task(
        task_id,
        TaskInput {
            feature: "auth".to_string(),
            discipline: "backend".to_string(),
            title: "New Title".to_string(),
            description: Some("New description".to_string()),
            priority: Some(Priority::High),
            tags: vec!["new".to_string()],
            depends_on: vec![],
            acceptance_criteria: Some(vec!["Updated criteria".to_string()]),
        },
    )
    .unwrap();

    // Verify update
    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.title, "New Title");
    assert_eq!(task.description, Some("New description".to_string()));
    assert_eq!(task.priority, Some(Priority::High));
    assert_eq!(task.tags, vec!["new"]);
    assert_eq!(
        task.acceptance_criteria,
        vec!["Updated criteria".to_string()]
    );
    assert!(task.updated.is_some()); // Should have updated timestamp
    assert!(task.created.is_some()); // Should preserve created timestamp
}

#[test]
fn test_update_nonexistent_task() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();

    let result = db.update_task(
        999,
        TaskInput {
            feature: "test".to_string(),
            discipline: "backend".to_string(),
            title: "Title".to_string(),
            description: None,
            priority: None,
            tags: vec![],
            depends_on: vec![],
            acceptance_criteria: None,
        },
    );

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn test_update_task_with_invalid_dependency() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();

    let task_id = db
        .create_task(TaskInput {
            feature: "test".to_string(),
            discipline: "backend".to_string(),
            title: "Task".to_string(),
            description: None,
            priority: None,
            tags: vec![],
            depends_on: vec![],
            acceptance_criteria: None,
        })
        .unwrap();

    let result = db.update_task(
        task_id,
        TaskInput {
            feature: "test".to_string(),
            discipline: "backend".to_string(),
            title: "Task".to_string(),
            description: None,
            priority: None,
            tags: vec![],
            depends_on: vec![999], // Non-existent
            acceptance_criteria: None,
        },
    );

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn test_update_task_creates_self_referential_dependency() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();

    let task_id = db
        .create_task(TaskInput {
            feature: "test".to_string(),
            discipline: "backend".to_string(),
            title: "Task".to_string(),
            description: None,
            priority: None,
            tags: vec![],
            depends_on: vec![],
            acceptance_criteria: None,
        })
        .unwrap();

    let result = db.update_task(
        task_id,
        TaskInput {
            feature: "test".to_string(),
            discipline: "backend".to_string(),
            title: "Task".to_string(),
            description: None,
            priority: None,
            tags: vec![],
            depends_on: vec![task_id], // Self-reference!
            acceptance_criteria: None,
        },
    );

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot depend on itself"));
}

#[test]
fn test_update_task_creates_circular_dependency() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();

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

    // Create task B depending on A
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

    // Try to update A to depend on B (would create cycle: A->B->A)
    let result = db.update_task(
        task_a,
        TaskInput {
            feature: "test".to_string(),
            discipline: "backend".to_string(),
            title: "Task A".to_string(),
            description: None,
            priority: None,
            tags: vec![],
            depends_on: vec![task_b], // Creates cycle!
            acceptance_criteria: None,
        },
    );

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Circular dependency"));
}

#[test]
fn test_update_task_complex_circular_dependency() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();

    // Create chain: A -> B -> C
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

    let task_c = db
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

    // Try to update A to depend on C (would create cycle: A->B->C->A)
    let result = db.update_task(
        task_a,
        TaskInput {
            feature: "test".to_string(),
            discipline: "backend".to_string(),
            title: "Task A".to_string(),
            description: None,
            priority: None,
            tags: vec![],
            depends_on: vec![task_c], // Creates long cycle!
            acceptance_criteria: None,
        },
    );

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Circular dependency"));
}

// === DELETE tests ===

#[test]
fn test_delete_task() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();

    let task_id = db
        .create_task(TaskInput {
            feature: "test".to_string(),
            discipline: "backend".to_string(),
            title: "Task to delete".to_string(),
            description: None,
            priority: None,
            tags: vec![],
            depends_on: vec![],
            acceptance_criteria: None,
        })
        .unwrap();

    assert_eq!(db.get_tasks().len(), 1);

    db.delete_task(task_id).unwrap();

    assert_eq!(db.get_tasks().len(), 0);
    assert!(db.get_task_by_id(task_id).is_none());
}

#[test]
fn test_delete_nonexistent_task() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    let result = db.delete_task(999);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn test_delete_task_with_dependents() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();

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

    // Create task B depending on A
    let _task_b = db
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

    // Try to delete task A (should fail because B depends on it)
    let result = db.delete_task(task_a);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("depends on it"));
}

#[test]
fn test_delete_dependent_task_then_dependency() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();

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

    // Create task B depending on A
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

    assert_eq!(db.get_tasks().len(), 2);

    // Delete B first (should succeed)
    db.delete_task(task_b).unwrap();
    assert_eq!(db.get_tasks().len(), 1);

    // Now delete A (should succeed since B is gone)
    db.delete_task(task_a).unwrap();
    assert_eq!(db.get_tasks().len(), 0);
}

// === Integration tests ===

#[test]
fn test_crud_lifecycle() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("auth".to_string(), "Auth".to_string(), "AUTH".to_string(), None).unwrap();

    // CREATE
    let task_id = db
        .create_task(TaskInput {
            feature: "auth".to_string(),
            discipline: "backend".to_string(),
            title: "Initial Title".to_string(),
            description: Some("Initial description".to_string()),
            priority: Some(Priority::Low),
            tags: vec![],
            depends_on: vec![],
            acceptance_criteria: None,
        })
        .unwrap();

    // READ
    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.title, "Initial Title");
    assert_eq!(task.status, TaskStatus::Pending);

    // UPDATE
    db.update_task(
        task_id,
        TaskInput {
            feature: "auth".to_string(),
            discipline: "frontend".to_string(), // Change discipline
            title: "Updated Title".to_string(),
            description: Some("Updated description".to_string()),
            priority: Some(Priority::High),
            tags: vec!["updated".to_string()],
            depends_on: vec![],
            acceptance_criteria: None,
        },
    )
    .unwrap();

    // READ again
    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.title, "Updated Title");
    assert_eq!(task.discipline, "frontend");

    // DELETE
    db.delete_task(task_id).unwrap();
    assert!(db.get_task_by_id(task_id).is_none());
}
