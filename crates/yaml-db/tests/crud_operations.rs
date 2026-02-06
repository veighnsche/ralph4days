//! CRUD operation tests for yaml-db crate
//!
//! Tests for Create, Read, Update, Delete operations on tasks.

use std::fs;
use tempfile::TempDir;
use yaml_db::{CommentAuthor, Priority, TaskInput, TaskStatus, YamlDatabase};

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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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

// === COMMENT tests ===

#[test]
fn test_add_human_comment() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();
    let task_id = db
        .create_task(TaskInput {
            feature: "test".to_string(),
            discipline: "backend".to_string(),
            title: "Task".to_string(),
            ..Default::default()
        })
        .unwrap();

    db.add_comment(task_id, CommentAuthor::Human, None, "Use bcrypt".to_string())
        .unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.comments.len(), 1);
    assert_eq!(task.comments[0].author, CommentAuthor::Human);
    assert_eq!(task.comments[0].body, "Use bcrypt");
    assert!(task.comments[0].agent_task_id.is_none());
    assert!(task.comments[0].created.is_some());
}

#[test]
fn test_add_agent_comment() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();
    let task_id = db
        .create_task(TaskInput {
            feature: "test".to_string(),
            discipline: "backend".to_string(),
            title: "Task".to_string(),
            ..Default::default()
        })
        .unwrap();

    db.add_comment(task_id, CommentAuthor::Agent, Some(5), "Failed: missing .env".to_string())
        .unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.comments.len(), 1);
    assert_eq!(task.comments[0].author, CommentAuthor::Agent);
    assert_eq!(task.comments[0].agent_task_id, Some(5));
    assert_eq!(task.comments[0].body, "Failed: missing .env");
}

#[test]
fn test_add_comment_empty_body_rejected() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();
    let task_id = db
        .create_task(TaskInput {
            feature: "test".to_string(),
            discipline: "backend".to_string(),
            title: "Task".to_string(),
            ..Default::default()
        })
        .unwrap();

    let result = db.add_comment(task_id, CommentAuthor::Human, None, "   ".to_string());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot be empty"));
}

#[test]
fn test_add_comment_agent_missing_task_id_rejected() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();
    let task_id = db
        .create_task(TaskInput {
            feature: "test".to_string(),
            discipline: "backend".to_string(),
            title: "Task".to_string(),
            ..Default::default()
        })
        .unwrap();

    let result = db.add_comment(task_id, CommentAuthor::Agent, None, "Note".to_string());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("agent_task_id is required"));
}

#[test]
fn test_add_comment_human_with_task_id_rejected() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();
    let task_id = db
        .create_task(TaskInput {
            feature: "test".to_string(),
            discipline: "backend".to_string(),
            title: "Task".to_string(),
            ..Default::default()
        })
        .unwrap();

    let result = db.add_comment(task_id, CommentAuthor::Human, Some(1), "Note".to_string());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("must not be set"));
}

#[test]
fn test_add_comment_nonexistent_task() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    let result = db.add_comment(999, CommentAuthor::Human, None, "Hello".to_string());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn test_add_multiple_comments() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();
    let task_id = db
        .create_task(TaskInput {
            feature: "test".to_string(),
            discipline: "backend".to_string(),
            title: "Task".to_string(),
            ..Default::default()
        })
        .unwrap();

    db.add_comment(task_id, CommentAuthor::Human, None, "First".to_string()).unwrap();
    db.add_comment(task_id, CommentAuthor::Agent, Some(1), "Second".to_string()).unwrap();
    db.add_comment(task_id, CommentAuthor::Human, None, "Third".to_string()).unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.comments.len(), 3);
    assert_eq!(task.comments[0].body, "First");
    assert_eq!(task.comments[1].body, "Second");
    assert_eq!(task.comments[2].body, "Third");
}

#[test]
fn test_comments_persist_through_reload() {
    let (_temp, db_path) = create_temp_db();

    // Add comment and drop db
    {
        let mut db = YamlDatabase::from_path(db_path.clone()).unwrap();
        db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();
        db.create_task(TaskInput {
            feature: "test".to_string(),
            discipline: "backend".to_string(),
            title: "Task".to_string(),
            ..Default::default()
        })
        .unwrap();
        db.add_comment(1, CommentAuthor::Human, None, "Persisted".to_string()).unwrap();
    }

    // Reload from disk
    let db = YamlDatabase::from_path(db_path).unwrap();
    let task = db.get_task_by_id(1).unwrap();
    assert_eq!(task.comments.len(), 1);
    assert_eq!(task.comments[0].body, "Persisted");
    assert_eq!(task.comments[0].author, CommentAuthor::Human);
    assert!(task.comments[0].agent_task_id.is_none());
    assert!(task.comments[0].created.is_some());
}

#[test]
fn test_comments_in_enriched_tasks() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();
    db.create_task(TaskInput {
        feature: "test".to_string(),
        discipline: "backend".to_string(),
        title: "Task".to_string(),
        ..Default::default()
    })
    .unwrap();
    db.add_comment(1, CommentAuthor::Human, None, "Visible in enriched".to_string()).unwrap();

    let enriched = db.get_enriched_tasks();
    assert_eq!(enriched.len(), 1);
    assert_eq!(enriched[0].comments.len(), 1);
    assert_eq!(enriched[0].comments[0].body, "Visible in enriched");
}

#[test]
fn test_update_task_preserves_comments() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();
    let task_id = db
        .create_task(TaskInput {
            feature: "test".to_string(),
            discipline: "backend".to_string(),
            title: "Original".to_string(),
            ..Default::default()
        })
        .unwrap();

    db.add_comment(task_id, CommentAuthor::Human, None, "Keep me".to_string()).unwrap();

    db.update_task(
        task_id,
        TaskInput {
            feature: "test".to_string(),
            discipline: "backend".to_string(),
            title: "Updated".to_string(),
            ..Default::default()
        },
    )
    .unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.title, "Updated");
    assert_eq!(task.comments.len(), 1);
    assert_eq!(task.comments[0].body, "Keep me");
}

// === FEATURE DELETE tests ===

#[test]
fn test_delete_feature() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("auth".to_string(), "Auth".to_string(), "AUTH".to_string(), None).unwrap();
    assert_eq!(db.get_features().len(), 1);

    db.delete_feature("auth".to_string()).unwrap();
    assert_eq!(db.get_features().len(), 0);
}

#[test]
fn test_delete_feature_nonexistent() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    let result = db.delete_feature("nope".to_string());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn test_delete_feature_with_tasks_rejected() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("auth".to_string(), "Auth".to_string(), "AUTH".to_string(), None).unwrap();
    db.create_task(TaskInput {
        feature: "auth".to_string(),
        discipline: "backend".to_string(),
        title: "Task".to_string(),
        ..Default::default()
    })
    .unwrap();

    let result = db.delete_feature("auth".to_string());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cannot delete feature"));
}

// === DISCIPLINE DELETE tests ===

#[test]
fn test_delete_discipline() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_discipline(
        "custom".to_string(), "Custom".to_string(), "CUST".to_string(),
        "Wrench".to_string(), "#ff0000".to_string(),
    ).unwrap();

    let initial_count = db.get_disciplines().len();
    db.delete_discipline("custom".to_string()).unwrap();
    assert_eq!(db.get_disciplines().len(), initial_count - 1);
}

#[test]
fn test_delete_discipline_nonexistent() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    let result = db.delete_discipline("nope".to_string());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn test_delete_discipline_with_tasks_rejected() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();
    db.create_task(TaskInput {
        feature: "test".to_string(),
        discipline: "backend".to_string(),
        title: "Task".to_string(),
        ..Default::default()
    })
    .unwrap();

    let result = db.delete_discipline("backend".to_string());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cannot delete discipline"));
}

// === COMMENT UPDATE/DELETE tests ===

#[test]
fn test_update_comment() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();
    let task_id = db.create_task(TaskInput {
        feature: "test".to_string(),
        discipline: "backend".to_string(),
        title: "Task".to_string(),
        ..Default::default()
    }).unwrap();

    db.add_comment(task_id, CommentAuthor::Human, None, "Original".to_string()).unwrap();
    db.update_comment(task_id, 0, "Edited".to_string()).unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.comments[0].body, "Edited");
    // Author and created should be preserved
    assert_eq!(task.comments[0].author, CommentAuthor::Human);
    assert!(task.comments[0].created.is_some());
}

#[test]
fn test_update_comment_empty_body_rejected() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();
    let task_id = db.create_task(TaskInput {
        feature: "test".to_string(),
        discipline: "backend".to_string(),
        title: "Task".to_string(),
        ..Default::default()
    }).unwrap();

    db.add_comment(task_id, CommentAuthor::Human, None, "Hello".to_string()).unwrap();

    let result = db.update_comment(task_id, 0, "   ".to_string());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot be empty"));
}

#[test]
fn test_update_comment_out_of_bounds() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();
    let task_id = db.create_task(TaskInput {
        feature: "test".to_string(),
        discipline: "backend".to_string(),
        title: "Task".to_string(),
        ..Default::default()
    }).unwrap();

    let result = db.update_comment(task_id, 0, "Hello".to_string());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("out of bounds"));
}

#[test]
fn test_delete_comment() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();
    let task_id = db.create_task(TaskInput {
        feature: "test".to_string(),
        discipline: "backend".to_string(),
        title: "Task".to_string(),
        ..Default::default()
    }).unwrap();

    db.add_comment(task_id, CommentAuthor::Human, None, "First".to_string()).unwrap();
    db.add_comment(task_id, CommentAuthor::Human, None, "Second".to_string()).unwrap();
    db.add_comment(task_id, CommentAuthor::Human, None, "Third".to_string()).unwrap();

    // Delete middle comment
    db.delete_comment(task_id, 1).unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.comments.len(), 2);
    assert_eq!(task.comments[0].body, "First");
    assert_eq!(task.comments[1].body, "Third");
}

#[test]
fn test_delete_comment_out_of_bounds() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    db.create_feature("test".to_string(), "Test".to_string(), "TEST".to_string(), None).unwrap();
    let task_id = db.create_task(TaskInput {
        feature: "test".to_string(),
        discipline: "backend".to_string(),
        title: "Task".to_string(),
        ..Default::default()
    }).unwrap();

    let result = db.delete_comment(task_id, 0);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("out of bounds"));
}

#[test]
fn test_delete_comment_nonexistent_task() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path).unwrap();

    let result = db.delete_comment(999, 0);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}
