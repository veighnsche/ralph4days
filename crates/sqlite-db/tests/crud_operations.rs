use sqlite_db::{CommentAuthor, Priority, SqliteDb, TaskInput, TaskStatus};

fn create_test_db() -> SqliteDb {
    let db = SqliteDb::open_in_memory().unwrap();
    db.seed_defaults().unwrap();
    db
}

// === CREATE tests ===

#[test]
fn test_create_task() {
    let db = create_test_db();
    db.create_feature("auth".into(), "Auth".into(), "AUTH".into(), None).unwrap();

    let task_id = db.create_task(TaskInput {
        feature: "auth".into(),
        discipline: "backend".into(),
        title: "Implement login".into(),
        description: Some("Login API".into()),
        priority: Some(Priority::High),
        tags: vec!["api".into()],
        depends_on: vec![],
        acceptance_criteria: Some(vec!["Works".into()]),
        ..Default::default()
    }).unwrap();

    assert_eq!(task_id, 1);
    assert_eq!(db.get_tasks().len(), 1);
}

#[test]
fn test_create_task_empty_feature_rejected() {
    let db = create_test_db();
    let result = db.create_task(TaskInput {
        feature: "   ".into(),
        discipline: "backend".into(),
        title: "Task".into(),
        ..Default::default()
    });
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Feature name cannot be empty"));
}

#[test]
fn test_create_task_empty_discipline_rejected() {
    let db = create_test_db();
    let result = db.create_task(TaskInput {
        feature: "test".into(),
        discipline: "\t\n ".into(),
        title: "Task".into(),
        ..Default::default()
    });
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Discipline name cannot be empty"));
}

#[test]
fn test_create_task_empty_title_rejected() {
    let db = create_test_db();
    let result = db.create_task(TaskInput {
        feature: "test".into(),
        discipline: "backend".into(),
        title: "     ".into(),
        ..Default::default()
    });
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Task title cannot be empty"));
}

#[test]
fn test_create_task_nonexistent_feature_rejected() {
    let db = create_test_db();
    let result = db.create_task(TaskInput {
        feature: "nope".into(),
        discipline: "backend".into(),
        title: "Task".into(),
        ..Default::default()
    });
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn test_create_task_nonexistent_discipline_rejected() {
    let db = create_test_db();
    db.create_feature("test".into(), "Test".into(), "TEST".into(), None).unwrap();
    let result = db.create_task(TaskInput {
        feature: "test".into(),
        discipline: "nope".into(),
        title: "Task".into(),
        ..Default::default()
    });
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

// === READ tests ===

#[test]
fn test_get_task_by_id() {
    let db = create_test_db();
    db.create_feature("test".into(), "Test".into(), "TEST".into(), None).unwrap();

    let task_id = db.create_task(TaskInput {
        feature: "test".into(),
        discipline: "backend".into(),
        title: "Test Task".into(),
        priority: Some(Priority::Medium),
        ..Default::default()
    }).unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.id, task_id);
    assert_eq!(task.title, "Test Task");
    assert_eq!(task.priority, Some(Priority::Medium));
}

#[test]
fn test_get_task_by_id_not_found() {
    let db = create_test_db();
    assert!(db.get_task_by_id(999).is_none());
}

// === UPDATE tests ===

#[test]
fn test_update_task() {
    let db = create_test_db();
    db.create_feature("auth".into(), "Auth".into(), "AUTH".into(), None).unwrap();

    let task_id = db.create_task(TaskInput {
        feature: "auth".into(),
        discipline: "backend".into(),
        title: "Old Title".into(),
        description: Some("Old description".into()),
        priority: Some(Priority::Low),
        tags: vec!["old".into()],
        ..Default::default()
    }).unwrap();

    db.update_task(task_id, TaskInput {
        feature: "auth".into(),
        discipline: "backend".into(),
        title: "New Title".into(),
        description: Some("New description".into()),
        priority: Some(Priority::High),
        tags: vec!["new".into()],
        acceptance_criteria: Some(vec!["Updated criteria".into()]),
        ..Default::default()
    }).unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.title, "New Title");
    assert_eq!(task.description, Some("New description".into()));
    assert_eq!(task.priority, Some(Priority::High));
    assert_eq!(task.tags, vec!["new"]);
    assert_eq!(task.acceptance_criteria, vec!["Updated criteria"]);
    assert!(task.updated.is_some());
    assert!(task.created.is_some()); // Preserved
}

#[test]
fn test_update_nonexistent_task() {
    let db = create_test_db();
    db.create_feature("test".into(), "Test".into(), "TEST".into(), None).unwrap();
    let result = db.update_task(999, TaskInput {
        feature: "test".into(),
        discipline: "backend".into(),
        title: "Title".into(),
        ..Default::default()
    });
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn test_update_task_self_dependency_rejected() {
    let db = create_test_db();
    db.create_feature("test".into(), "Test".into(), "TEST".into(), None).unwrap();

    let task_id = db.create_task(TaskInput {
        feature: "test".into(),
        discipline: "backend".into(),
        title: "Task".into(),
        ..Default::default()
    }).unwrap();

    let result = db.update_task(task_id, TaskInput {
        feature: "test".into(),
        discipline: "backend".into(),
        title: "Task".into(),
        depends_on: vec![task_id],
        ..Default::default()
    });
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot depend on itself"));
}

#[test]
fn test_update_task_circular_dependency_rejected() {
    let db = create_test_db();
    db.create_feature("test".into(), "Test".into(), "TEST".into(), None).unwrap();

    let a = db.create_task(TaskInput {
        feature: "test".into(), discipline: "backend".into(), title: "A".into(),
        ..Default::default()
    }).unwrap();

    let b = db.create_task(TaskInput {
        feature: "test".into(), discipline: "backend".into(), title: "B".into(),
        depends_on: vec![a],
        ..Default::default()
    }).unwrap();

    // Try to make A depend on B (cycle: A->B->A)
    let result = db.update_task(a, TaskInput {
        feature: "test".into(), discipline: "backend".into(), title: "A".into(),
        depends_on: vec![b],
        ..Default::default()
    });
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Circular dependency"));
}

#[test]
fn test_update_task_complex_circular_dependency() {
    let db = create_test_db();
    db.create_feature("test".into(), "Test".into(), "TEST".into(), None).unwrap();

    let a = db.create_task(TaskInput {
        feature: "test".into(), discipline: "backend".into(), title: "A".into(),
        ..Default::default()
    }).unwrap();
    let b = db.create_task(TaskInput {
        feature: "test".into(), discipline: "backend".into(), title: "B".into(),
        depends_on: vec![a],
        ..Default::default()
    }).unwrap();
    let c = db.create_task(TaskInput {
        feature: "test".into(), discipline: "backend".into(), title: "C".into(),
        depends_on: vec![b],
        ..Default::default()
    }).unwrap();

    // Try A->C (cycle: A->B->C->A)
    let result = db.update_task(a, TaskInput {
        feature: "test".into(), discipline: "backend".into(), title: "A".into(),
        depends_on: vec![c],
        ..Default::default()
    });
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Circular dependency"));
}

#[test]
fn test_update_task_preserves_status() {
    let db = create_test_db();
    db.create_feature("test".into(), "Test".into(), "TEST".into(), None).unwrap();

    let task_id = db.create_task(TaskInput {
        feature: "test".into(), discipline: "backend".into(), title: "Original".into(),
        ..Default::default()
    }).unwrap();

    // Status is "pending" by default
    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.status, TaskStatus::Pending);

    // Update should preserve status
    db.update_task(task_id, TaskInput {
        feature: "test".into(), discipline: "backend".into(), title: "Updated".into(),
        ..Default::default()
    }).unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.status, TaskStatus::Pending);
    assert_eq!(task.title, "Updated");
}

// === DELETE tests ===

#[test]
fn test_delete_task() {
    let db = create_test_db();
    db.create_feature("test".into(), "Test".into(), "TEST".into(), None).unwrap();

    let task_id = db.create_task(TaskInput {
        feature: "test".into(), discipline: "backend".into(), title: "To delete".into(),
        ..Default::default()
    }).unwrap();

    assert_eq!(db.get_tasks().len(), 1);
    db.delete_task(task_id).unwrap();
    assert_eq!(db.get_tasks().len(), 0);
    assert!(db.get_task_by_id(task_id).is_none());
}

#[test]
fn test_delete_nonexistent_task() {
    let db = create_test_db();
    let result = db.delete_task(999);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn test_delete_task_with_dependents_rejected() {
    let db = create_test_db();
    db.create_feature("test".into(), "Test".into(), "TEST".into(), None).unwrap();

    let a = db.create_task(TaskInput {
        feature: "test".into(), discipline: "backend".into(), title: "A".into(),
        ..Default::default()
    }).unwrap();
    db.create_task(TaskInput {
        feature: "test".into(), discipline: "backend".into(), title: "B".into(),
        depends_on: vec![a],
        ..Default::default()
    }).unwrap();

    let result = db.delete_task(a);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("depends on it"));
}

#[test]
fn test_delete_dependent_then_dependency() {
    let db = create_test_db();
    db.create_feature("test".into(), "Test".into(), "TEST".into(), None).unwrap();

    let a = db.create_task(TaskInput {
        feature: "test".into(), discipline: "backend".into(), title: "A".into(),
        ..Default::default()
    }).unwrap();
    let b = db.create_task(TaskInput {
        feature: "test".into(), discipline: "backend".into(), title: "B".into(),
        depends_on: vec![a],
        ..Default::default()
    }).unwrap();

    db.delete_task(b).unwrap();
    db.delete_task(a).unwrap();
    assert_eq!(db.get_tasks().len(), 0);
}

// === FEATURE tests ===

#[test]
fn test_create_feature() {
    let db = create_test_db();
    db.create_feature("auth".into(), "Auth".into(), "AUTH".into(), Some("Auth feature".into())).unwrap();
    let features = db.get_features();
    assert!(features.iter().any(|f| f.name == "auth"));
}

#[test]
fn test_create_duplicate_feature_rejected() {
    let db = create_test_db();
    db.create_feature("auth".into(), "Auth".into(), "AUTH".into(), None).unwrap();
    let result = db.create_feature("auth".into(), "Auth2".into(), "AUT2".into(), None);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already exists"));
}

#[test]
fn test_create_feature_duplicate_acronym_rejected() {
    let db = create_test_db();
    db.create_feature("auth".into(), "Auth".into(), "AUTH".into(), None).unwrap();
    let result = db.create_feature("other".into(), "Other".into(), "AUTH".into(), None);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already used"));
}

#[test]
fn test_update_feature() {
    let db = create_test_db();
    db.create_feature("auth".into(), "Auth".into(), "AUTH".into(), None).unwrap();
    db.update_feature("auth".into(), "Authentication".into(), "AUTH".into(), Some("Updated".into())).unwrap();

    let features = db.get_features();
    let f = features.iter().find(|f| f.name == "auth").unwrap();
    assert_eq!(f.display_name, "Authentication");
    assert_eq!(f.description, Some("Updated".into()));
    assert!(f.created.is_some()); // Preserved
}

#[test]
fn test_delete_feature() {
    let db = create_test_db();
    db.create_feature("auth".into(), "Auth".into(), "AUTH".into(), None).unwrap();
    let initial = db.get_features().len();
    db.delete_feature("auth".into()).unwrap();
    assert_eq!(db.get_features().len(), initial - 1);
}

#[test]
fn test_delete_feature_nonexistent() {
    let db = create_test_db();
    let result = db.delete_feature("nope".into());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn test_delete_feature_with_tasks_rejected() {
    let db = create_test_db();
    db.create_feature("auth".into(), "Auth".into(), "AUTH".into(), None).unwrap();
    db.create_task(TaskInput {
        feature: "auth".into(), discipline: "backend".into(), title: "Task".into(),
        ..Default::default()
    }).unwrap();

    let result = db.delete_feature("auth".into());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cannot delete feature"));
}

// === DISCIPLINE tests ===

#[test]
fn test_create_discipline() {
    let db = create_test_db();
    db.create_discipline("custom".into(), "Custom".into(), "CUST".into(), "Wrench".into(), "#ff0000".into()).unwrap();
    let disciplines = db.get_disciplines();
    assert!(disciplines.iter().any(|d| d.name == "custom"));
}

#[test]
fn test_create_duplicate_discipline_rejected() {
    let db = create_test_db();
    // "backend" already seeded
    let result = db.create_discipline("backend".into(), "Backend2".into(), "BAC2".into(), "Server".into(), "#000".into());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already exists"));
}

#[test]
fn test_update_discipline() {
    let db = create_test_db();
    db.create_discipline("custom".into(), "Custom".into(), "CUST".into(), "Wrench".into(), "#ff0000".into()).unwrap();
    db.update_discipline("custom".into(), "Custom Updated".into(), "CUST".into(), "Star".into(), "#00ff00".into()).unwrap();

    let disciplines = db.get_disciplines();
    let d = disciplines.iter().find(|d| d.name == "custom").unwrap();
    assert_eq!(d.display_name, "Custom Updated");
    assert_eq!(d.icon, "Star");
    assert_eq!(d.color, "#00ff00");
}

#[test]
fn test_delete_discipline() {
    let db = create_test_db();
    db.create_discipline("custom".into(), "Custom".into(), "CUST".into(), "Wrench".into(), "#ff0000".into()).unwrap();
    let initial = db.get_disciplines().len();
    db.delete_discipline("custom".into()).unwrap();
    assert_eq!(db.get_disciplines().len(), initial - 1);
}

#[test]
fn test_delete_discipline_nonexistent() {
    let db = create_test_db();
    let result = db.delete_discipline("nope".into());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn test_delete_discipline_with_tasks_rejected() {
    let db = create_test_db();
    db.create_feature("test".into(), "Test".into(), "TEST".into(), None).unwrap();
    db.create_task(TaskInput {
        feature: "test".into(), discipline: "backend".into(), title: "Task".into(),
        ..Default::default()
    }).unwrap();

    let result = db.delete_discipline("backend".into());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cannot delete discipline"));
}

// === COMMENT tests ===

#[test]
fn test_add_human_comment() {
    let db = create_test_db();
    db.create_feature("test".into(), "Test".into(), "TEST".into(), None).unwrap();
    let task_id = db.create_task(TaskInput {
        feature: "test".into(), discipline: "backend".into(), title: "Task".into(),
        ..Default::default()
    }).unwrap();

    db.add_comment(task_id, CommentAuthor::Human, None, "Use bcrypt".into()).unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.comments.len(), 1);
    assert_eq!(task.comments[0].author, CommentAuthor::Human);
    assert_eq!(task.comments[0].body, "Use bcrypt");
    assert!(task.comments[0].agent_task_id.is_none());
    assert!(task.comments[0].created.is_some());
}

#[test]
fn test_add_agent_comment() {
    let db = create_test_db();
    db.create_feature("test".into(), "Test".into(), "TEST".into(), None).unwrap();
    let task_id = db.create_task(TaskInput {
        feature: "test".into(), discipline: "backend".into(), title: "Task".into(),
        ..Default::default()
    }).unwrap();

    db.add_comment(task_id, CommentAuthor::Agent, Some(5), "Failed: missing .env".into()).unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.comments[0].author, CommentAuthor::Agent);
    assert_eq!(task.comments[0].agent_task_id, Some(5));
}

#[test]
fn test_add_comment_empty_body_rejected() {
    let db = create_test_db();
    db.create_feature("test".into(), "Test".into(), "TEST".into(), None).unwrap();
    let task_id = db.create_task(TaskInput {
        feature: "test".into(), discipline: "backend".into(), title: "Task".into(),
        ..Default::default()
    }).unwrap();

    let result = db.add_comment(task_id, CommentAuthor::Human, None, "   ".into());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot be empty"));
}

#[test]
fn test_add_comment_agent_missing_task_id_rejected() {
    let db = create_test_db();
    db.create_feature("test".into(), "Test".into(), "TEST".into(), None).unwrap();
    let task_id = db.create_task(TaskInput {
        feature: "test".into(), discipline: "backend".into(), title: "Task".into(),
        ..Default::default()
    }).unwrap();

    let result = db.add_comment(task_id, CommentAuthor::Agent, None, "Note".into());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("agent_task_id is required"));
}

#[test]
fn test_add_comment_human_with_task_id_rejected() {
    let db = create_test_db();
    db.create_feature("test".into(), "Test".into(), "TEST".into(), None).unwrap();
    let task_id = db.create_task(TaskInput {
        feature: "test".into(), discipline: "backend".into(), title: "Task".into(),
        ..Default::default()
    }).unwrap();

    let result = db.add_comment(task_id, CommentAuthor::Human, Some(1), "Note".into());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("must not be set"));
}

#[test]
fn test_add_comment_nonexistent_task() {
    let db = create_test_db();
    let result = db.add_comment(999, CommentAuthor::Human, None, "Hello".into());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn test_update_comment_by_id() {
    let db = create_test_db();
    db.create_feature("test".into(), "Test".into(), "TEST".into(), None).unwrap();
    let task_id = db.create_task(TaskInput {
        feature: "test".into(), discipline: "backend".into(), title: "Task".into(),
        ..Default::default()
    }).unwrap();

    db.add_comment(task_id, CommentAuthor::Human, None, "Original".into()).unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    let comment_id = task.comments[0].id;

    db.update_comment(task_id, comment_id, "Edited".into()).unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.comments[0].body, "Edited");
    assert_eq!(task.comments[0].author, CommentAuthor::Human); // Preserved
}

#[test]
fn test_delete_comment_by_id() {
    let db = create_test_db();
    db.create_feature("test".into(), "Test".into(), "TEST".into(), None).unwrap();
    let task_id = db.create_task(TaskInput {
        feature: "test".into(), discipline: "backend".into(), title: "Task".into(),
        ..Default::default()
    }).unwrap();

    db.add_comment(task_id, CommentAuthor::Human, None, "First".into()).unwrap();
    db.add_comment(task_id, CommentAuthor::Human, None, "Second".into()).unwrap();
    db.add_comment(task_id, CommentAuthor::Human, None, "Third".into()).unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    let second_id = task.comments[1].id;

    // Delete middle comment by stable ID
    db.delete_comment(task_id, second_id).unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.comments.len(), 2);
    assert_eq!(task.comments[0].body, "First");
    assert_eq!(task.comments[1].body, "Third");
}

#[test]
fn test_update_task_preserves_comments() {
    let db = create_test_db();
    db.create_feature("test".into(), "Test".into(), "TEST".into(), None).unwrap();
    let task_id = db.create_task(TaskInput {
        feature: "test".into(), discipline: "backend".into(), title: "Original".into(),
        ..Default::default()
    }).unwrap();

    db.add_comment(task_id, CommentAuthor::Human, None, "Keep me".into()).unwrap();

    db.update_task(task_id, TaskInput {
        feature: "test".into(), discipline: "backend".into(), title: "Updated".into(),
        ..Default::default()
    }).unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.title, "Updated");
    assert_eq!(task.comments.len(), 1);
    assert_eq!(task.comments[0].body, "Keep me");
}

// === ENRICHED TASKS tests ===

#[test]
fn test_enriched_tasks() {
    let db = create_test_db();
    db.create_feature("auth".into(), "Authentication".into(), "AUTH".into(), None).unwrap();
    db.create_task(TaskInput {
        feature: "auth".into(), discipline: "backend".into(), title: "Login".into(),
        ..Default::default()
    }).unwrap();

    let enriched = db.get_enriched_tasks();
    assert_eq!(enriched.len(), 1);
    assert_eq!(enriched[0].feature_display_name, "Authentication");
    assert_eq!(enriched[0].feature_acronym, "AUTH");
    assert_eq!(enriched[0].discipline_display_name, "Backend");
    assert_eq!(enriched[0].discipline_acronym, "BACK");
    assert_eq!(enriched[0].discipline_icon, "Server");
}

#[test]
fn test_enriched_tasks_comments_visible() {
    let db = create_test_db();
    db.create_feature("test".into(), "Test".into(), "TEST".into(), None).unwrap();
    db.create_task(TaskInput {
        feature: "test".into(), discipline: "backend".into(), title: "Task".into(),
        ..Default::default()
    }).unwrap();
    db.add_comment(1, CommentAuthor::Human, None, "Visible in enriched".into()).unwrap();

    let enriched = db.get_enriched_tasks();
    assert_eq!(enriched[0].comments.len(), 1);
    assert_eq!(enriched[0].comments[0].body, "Visible in enriched");
}

// === INFERRED STATUS tests ===

#[test]
fn test_inferred_status_ready() {
    let db = create_test_db();
    db.create_feature("test".into(), "Test".into(), "TEST".into(), None).unwrap();
    db.create_task(TaskInput {
        feature: "test".into(), discipline: "backend".into(), title: "Task".into(),
        ..Default::default()
    }).unwrap();

    let enriched = db.get_enriched_tasks();
    assert_eq!(enriched[0].inferred_status, sqlite_db::InferredTaskStatus::Ready);
}

#[test]
fn test_inferred_status_waiting_on_deps() {
    let db = create_test_db();
    db.create_feature("test".into(), "Test".into(), "TEST".into(), None).unwrap();

    let a = db.create_task(TaskInput {
        feature: "test".into(), discipline: "backend".into(), title: "A".into(),
        ..Default::default()
    }).unwrap();
    db.create_task(TaskInput {
        feature: "test".into(), discipline: "backend".into(), title: "B".into(),
        depends_on: vec![a],
        ..Default::default()
    }).unwrap();

    let enriched = db.get_enriched_tasks();
    let b_enriched = enriched.iter().find(|t| t.title == "B").unwrap();
    assert_eq!(b_enriched.inferred_status, sqlite_db::InferredTaskStatus::WaitingOnDeps);
}

// === STATS tests ===

#[test]
fn test_feature_stats() {
    let db = create_test_db();
    db.create_feature("auth".into(), "Auth".into(), "AUTH".into(), None).unwrap();
    db.create_feature("search".into(), "Search".into(), "SRCH".into(), None).unwrap();

    db.create_task(TaskInput {
        feature: "auth".into(), discipline: "backend".into(), title: "T1".into(),
        ..Default::default()
    }).unwrap();
    db.create_task(TaskInput {
        feature: "auth".into(), discipline: "backend".into(), title: "T2".into(),
        ..Default::default()
    }).unwrap();

    let stats = db.get_feature_stats();
    let auth_stats = stats.iter().find(|s| s.name == "auth").unwrap();
    assert_eq!(auth_stats.total, 2);
    assert_eq!(auth_stats.pending, 2);

    let search_stats = stats.iter().find(|s| s.name == "search").unwrap();
    assert_eq!(search_stats.total, 0);
}

#[test]
fn test_project_progress() {
    let db = create_test_db();
    assert_eq!(db.get_project_progress().total_tasks, 0);
    assert_eq!(db.get_project_progress().progress_percent, 0);
}

#[test]
fn test_get_all_tags() {
    let db = create_test_db();
    db.create_feature("test".into(), "Test".into(), "TEST".into(), None).unwrap();

    db.create_task(TaskInput {
        feature: "test".into(), discipline: "backend".into(), title: "T1".into(),
        tags: vec!["api".into(), "auth".into()],
        ..Default::default()
    }).unwrap();
    db.create_task(TaskInput {
        feature: "test".into(), discipline: "backend".into(), title: "T2".into(),
        tags: vec!["auth".into(), "db".into()],
        ..Default::default()
    }).unwrap();

    let tags = db.get_all_tags();
    assert_eq!(tags, vec!["api", "auth", "db"]); // Sorted, deduplicated
}

// === METADATA tests ===

#[test]
fn test_project_info() {
    let db = create_test_db();
    db.initialize_metadata("My Project".into(), Some("Description".into())).unwrap();

    let info = db.get_project_info();
    assert_eq!(info.title, "My Project");
    assert_eq!(info.description, Some("Description".into()));
    assert!(info.created.is_some());
}

// === EXPORT tests ===

#[test]
fn test_export_prd_yaml_deterministic() {
    let db = create_test_db();
    db.initialize_metadata("Test".into(), None).unwrap();
    db.create_feature("auth".into(), "Auth".into(), "AUTH".into(), None).unwrap();
    db.create_task(TaskInput {
        feature: "auth".into(), discipline: "backend".into(), title: "Login".into(),
        ..Default::default()
    }).unwrap();

    let export1 = db.export_prd_yaml().unwrap();
    let export2 = db.export_prd_yaml().unwrap();
    assert_eq!(export1, export2); // Deterministic
    assert!(export1.contains("auth"));
    assert!(export1.contains("Login"));
}

// === LIFECYCLE tests ===

#[test]
fn test_crud_lifecycle() {
    let db = create_test_db();
    db.create_feature("auth".into(), "Auth".into(), "AUTH".into(), None).unwrap();

    let task_id = db.create_task(TaskInput {
        feature: "auth".into(), discipline: "backend".into(), title: "Initial".into(),
        priority: Some(Priority::Low),
        ..Default::default()
    }).unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.title, "Initial");
    assert_eq!(task.status, TaskStatus::Pending);

    db.update_task(task_id, TaskInput {
        feature: "auth".into(), discipline: "frontend".into(), title: "Updated".into(),
        priority: Some(Priority::High),
        tags: vec!["updated".into()],
        ..Default::default()
    }).unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.title, "Updated");
    assert_eq!(task.discipline, "frontend");

    db.delete_task(task_id).unwrap();
    assert!(db.get_task_by_id(task_id).is_none());
}

// === SET TASK STATUS tests ===

#[test]
fn test_set_task_status() {
    let db = create_test_db();
    db.create_feature("auth".into(), "Auth".into(), "AUTH".into(), None).unwrap();
    let task_id = db.create_task(TaskInput {
        feature: "auth".into(),
        discipline: "backend".into(),
        title: "Login flow".into(),
        description: None,
        priority: None,
        tags: vec![],
        depends_on: vec![],
        acceptance_criteria: None,
        context_files: vec![], output_artifacts: vec![], hints: None, estimated_turns: None, provenance: None,
    }).unwrap();

    // Default is pending
    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.status, TaskStatus::Pending);
    assert!(task.completed.is_none());

    // Transition to in_progress
    db.set_task_status(task_id, TaskStatus::InProgress).unwrap();
    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.status, TaskStatus::InProgress);
    assert!(task.completed.is_none());
    assert!(task.updated.is_some());

    // Transition to done — should set completed timestamp
    db.set_task_status(task_id, TaskStatus::Done).unwrap();
    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.status, TaskStatus::Done);
    assert!(task.completed.is_some());
}

#[test]
fn test_set_task_status_nonexistent() {
    let db = create_test_db();
    let result = db.set_task_status(999, TaskStatus::Done);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

// === YAML EXPORT ESCAPING test ===

#[test]
fn test_export_yaml_escapes_special_chars() {
    let db = create_test_db();
    db.create_feature("test".into(), "Test \"Feature\"".into(), "TSTF".into(), Some("A description with \"quotes\" and\nnewlines".into())).unwrap();
    let task_id = db.create_task(TaskInput {
        feature: "test".into(),
        discipline: "backend".into(),
        title: "Fix the \"bug\" in code".into(),
        description: Some("Line 1\nLine 2\tTabbed".into()),
        priority: None,
        tags: vec!["tag with \"quotes\"".into()],
        depends_on: vec![],
        acceptance_criteria: Some(vec!["Check \"output\" is correct".into()]),
        context_files: vec![], output_artifacts: vec![], hints: None, estimated_turns: None, provenance: None,
    }).unwrap();
    db.add_comment(task_id, CommentAuthor::Human, None, "Comment with \"quotes\" and\nnewlines".into()).unwrap();

    let yaml = db.export_prd_yaml().unwrap();

    // Verify escaped quotes don't break the YAML structure
    assert!(yaml.contains(r#"display_name: "Test \"Feature\"""#));
    assert!(yaml.contains(r#"title: "Fix the \"bug\" in code""#));
    assert!(yaml.contains(r#"description: "Line 1\nLine 2\tTabbed""#));
    assert!(yaml.contains(r#"- "tag with \"quotes\"""#));
    assert!(yaml.contains(r#"body: "Comment with \"quotes\" and\nnewlines""#));
    // Verify no unescaped quotes mid-string
    assert!(!yaml.contains("\"bug\""));
}

// === SEED DEFAULTS test ===

#[test]
fn test_seed_defaults() {
    let db = create_test_db();
    let disciplines = db.get_disciplines();
    assert_eq!(disciplines.len(), 10);

    let names: Vec<&str> = disciplines.iter().map(|d| d.name.as_str()).collect();
    assert!(names.contains(&"frontend"));
    assert!(names.contains(&"backend"));
    assert!(names.contains(&"api"));
}
