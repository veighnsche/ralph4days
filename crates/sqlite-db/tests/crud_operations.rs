use sqlite_db::{
    FeatureInput, FeatureLearning, FixedClock, Priority, SqliteDb, TaskInput, TaskStatus,
};

fn create_test_db() -> SqliteDb {
    let clock = Box::new(FixedClock(
        chrono::NaiveDate::from_ymd_opt(2026, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc(),
    ));
    let db = SqliteDb::open_in_memory_with_clock(clock).unwrap();
    seed_test_disciplines(&db);
    db
}

fn seed_test_disciplines(db: &SqliteDb) {
    let disciplines = [
        ("frontend", "Frontend", "FRNT", "Monitor", "#3b82f6"),
        ("backend", "Backend", "BACK", "Server", "#8b5cf6"),
        ("data", "Data", "DATA", "Database", "#10b981"),
        ("platform", "Platform", "PLTF", "Cloud", "#6366f1"),
        ("quality", "Quality", "QLTY", "FlaskConical", "#f59e0b"),
        ("security", "Security", "SECR", "Shield", "#ef4444"),
        ("integration", "Integration", "INTG", "Cable", "#06b6d4"),
        (
            "documentation",
            "Documentation",
            "DOCS",
            "BookOpen",
            "#14b8a6",
        ),
    ];
    for (name, display, acronym, icon, color) in disciplines {
        db.create_discipline(sqlite_db::DisciplineInput {
            name: name.to_owned(),
            display_name: display.to_owned(),
            acronym: acronym.to_owned(),
            icon: icon.to_owned(),
            color: color.to_owned(),
            system_prompt: Some("Test prompt".to_owned()),
            skills: "[]".to_owned(),
            conventions: Some("Test conventions".to_owned()),
            mcp_servers: "[]".to_owned(),
            image_path: None,
            crops: None,
            description: None,
            image_prompt: None,
        })
        .unwrap();
    }
}

fn feature(name: &str, display_name: &str, acronym: &str) -> FeatureInput {
    FeatureInput {
        name: name.into(),
        display_name: display_name.into(),
        acronym: acronym.into(),
        ..Default::default()
    }
}

// === CREATE tests ===

#[test]
fn test_create_task() {
    let db = create_test_db();
    db.create_feature(feature("auth", "Auth", "AUTH")).unwrap();

    let task_id = db
        .create_task(TaskInput {
            feature: "auth".into(),
            discipline: "backend".into(),
            title: "Implement login".into(),
            description: Some("Login API".into()),
            priority: Some(Priority::High),
            tags: vec!["api".into()],
            depends_on: vec![],
            acceptance_criteria: Some(vec!["Works".into()]),
            ..Default::default()
        })
        .unwrap();

    assert_eq!(task_id, 1);
    let tasks = db.get_tasks();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].created, Some("2026-01-01".into()));
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
    assert!(result
        .unwrap_err()
        .contains("Discipline name cannot be empty"));
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
    db.create_feature(feature("test", "Test", "TEST")).unwrap();
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
    db.create_feature(feature("test", "Test", "TEST")).unwrap();

    let task_id = db
        .create_task(TaskInput {
            feature: "test".into(),
            discipline: "backend".into(),
            title: "Test Task".into(),
            priority: Some(Priority::Medium),
            ..Default::default()
        })
        .unwrap();

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
    db.create_feature(feature("auth", "Auth", "AUTH")).unwrap();

    let task_id = db
        .create_task(TaskInput {
            feature: "auth".into(),
            discipline: "backend".into(),
            title: "Old Title".into(),
            description: Some("Old description".into()),
            priority: Some(Priority::Low),
            tags: vec!["old".into()],
            ..Default::default()
        })
        .unwrap();

    db.update_task(
        task_id,
        TaskInput {
            feature: "auth".into(),
            discipline: "backend".into(),
            title: "New Title".into(),
            description: Some("New description".into()),
            priority: Some(Priority::High),
            tags: vec!["new".into()],
            acceptance_criteria: Some(vec!["Updated criteria".into()]),
            ..Default::default()
        },
    )
    .unwrap();

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
    db.create_feature(feature("test", "Test", "TEST")).unwrap();
    let result = db.update_task(
        999,
        TaskInput {
            feature: "test".into(),
            discipline: "backend".into(),
            title: "Title".into(),
            ..Default::default()
        },
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn test_update_task_self_dependency_rejected() {
    let db = create_test_db();
    db.create_feature(feature("test", "Test", "TEST")).unwrap();

    let task_id = db
        .create_task(TaskInput {
            feature: "test".into(),
            discipline: "backend".into(),
            title: "Task".into(),
            ..Default::default()
        })
        .unwrap();

    let result = db.update_task(
        task_id,
        TaskInput {
            feature: "test".into(),
            discipline: "backend".into(),
            title: "Task".into(),
            depends_on: vec![task_id],
            ..Default::default()
        },
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot depend on itself"));
}

#[test]
fn test_update_task_circular_dependency_rejected() {
    let db = create_test_db();
    db.create_feature(feature("test", "Test", "TEST")).unwrap();

    let a = db
        .create_task(TaskInput {
            feature: "test".into(),
            discipline: "backend".into(),
            title: "A".into(),
            ..Default::default()
        })
        .unwrap();

    let b = db
        .create_task(TaskInput {
            feature: "test".into(),
            discipline: "backend".into(),
            title: "B".into(),
            depends_on: vec![a],
            ..Default::default()
        })
        .unwrap();

    // Try to make A depend on B (cycle: A->B->A)
    let result = db.update_task(
        a,
        TaskInput {
            feature: "test".into(),
            discipline: "backend".into(),
            title: "A".into(),
            depends_on: vec![b],
            ..Default::default()
        },
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Circular dependency"));
}

#[test]
fn test_update_task_complex_circular_dependency() {
    let db = create_test_db();
    db.create_feature(feature("test", "Test", "TEST")).unwrap();

    let a = db
        .create_task(TaskInput {
            feature: "test".into(),
            discipline: "backend".into(),
            title: "A".into(),
            ..Default::default()
        })
        .unwrap();
    let b = db
        .create_task(TaskInput {
            feature: "test".into(),
            discipline: "backend".into(),
            title: "B".into(),
            depends_on: vec![a],
            ..Default::default()
        })
        .unwrap();
    let c = db
        .create_task(TaskInput {
            feature: "test".into(),
            discipline: "backend".into(),
            title: "C".into(),
            depends_on: vec![b],
            ..Default::default()
        })
        .unwrap();

    // Try A->C (cycle: A->B->C->A)
    let result = db.update_task(
        a,
        TaskInput {
            feature: "test".into(),
            discipline: "backend".into(),
            title: "A".into(),
            depends_on: vec![c],
            ..Default::default()
        },
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Circular dependency"));
}

#[test]
fn test_update_task_preserves_status() {
    let db = create_test_db();
    db.create_feature(feature("test", "Test", "TEST")).unwrap();

    let task_id = db
        .create_task(TaskInput {
            feature: "test".into(),
            discipline: "backend".into(),
            title: "Original".into(),
            ..Default::default()
        })
        .unwrap();

    // Status is "pending" by default
    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.status, TaskStatus::Pending);

    // Update should preserve status
    db.update_task(
        task_id,
        TaskInput {
            feature: "test".into(),
            discipline: "backend".into(),
            title: "Updated".into(),
            ..Default::default()
        },
    )
    .unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.status, TaskStatus::Pending);
    assert_eq!(task.title, "Updated");
}

// === DELETE tests ===

#[test]
fn test_delete_task() {
    let db = create_test_db();
    db.create_feature(feature("test", "Test", "TEST")).unwrap();

    let task_id = db
        .create_task(TaskInput {
            feature: "test".into(),
            discipline: "backend".into(),
            title: "To delete".into(),
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
    let db = create_test_db();
    let result = db.delete_task(999);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn test_delete_task_with_dependents_rejected() {
    let db = create_test_db();
    db.create_feature(feature("test", "Test", "TEST")).unwrap();

    let a = db
        .create_task(TaskInput {
            feature: "test".into(),
            discipline: "backend".into(),
            title: "A".into(),
            ..Default::default()
        })
        .unwrap();
    db.create_task(TaskInput {
        feature: "test".into(),
        discipline: "backend".into(),
        title: "B".into(),
        depends_on: vec![a],
        ..Default::default()
    })
    .unwrap();

    let result = db.delete_task(a);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("depends on it"));
}

#[test]
fn test_delete_dependent_then_dependency() {
    let db = create_test_db();
    db.create_feature(feature("test", "Test", "TEST")).unwrap();

    let a = db
        .create_task(TaskInput {
            feature: "test".into(),
            discipline: "backend".into(),
            title: "A".into(),
            ..Default::default()
        })
        .unwrap();
    let b = db
        .create_task(TaskInput {
            feature: "test".into(),
            discipline: "backend".into(),
            title: "B".into(),
            depends_on: vec![a],
            ..Default::default()
        })
        .unwrap();

    db.delete_task(b).unwrap();
    db.delete_task(a).unwrap();
    assert_eq!(db.get_tasks().len(), 0);
}

// === FEATURE tests ===

#[test]
fn test_create_feature() {
    let db = create_test_db();
    db.create_feature(FeatureInput {
        name: "auth".into(),
        display_name: "Auth".into(),
        acronym: "AUTH".into(),
        description: Some("Auth feature".into()),
        ..Default::default()
    })
    .unwrap();
    let features = db.get_features();
    let f = features.iter().find(|f| f.name == "auth").unwrap();
    assert_eq!(f.created, Some("2026-01-01".into()));
}

#[test]
fn test_create_duplicate_feature_rejected() {
    let db = create_test_db();
    db.create_feature(feature("auth", "Auth", "AUTH")).unwrap();
    let result = db.create_feature(feature("auth", "Auth2", "AUT2"));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already exists"));
}

#[test]
fn test_create_feature_duplicate_acronym_rejected() {
    let db = create_test_db();
    db.create_feature(feature("auth", "Auth", "AUTH")).unwrap();
    let result = db.create_feature(feature("other", "Other", "AUTH"));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already used"));
}

#[test]
fn test_update_feature() {
    let db = create_test_db();
    db.create_feature(feature("auth", "Auth", "AUTH")).unwrap();
    db.update_feature(FeatureInput {
        name: "auth".into(),
        display_name: "Authentication".into(),
        acronym: "AUTH".into(),
        description: Some("Updated".into()),
        ..Default::default()
    })
    .unwrap();

    let features = db.get_features();
    let f = features.iter().find(|f| f.name == "auth").unwrap();
    assert_eq!(f.display_name, "Authentication");
    assert_eq!(f.description, Some("Updated".into()));
    assert!(f.created.is_some()); // Preserved
}

#[test]
fn test_delete_feature() {
    let db = create_test_db();
    db.create_feature(feature("auth", "Auth", "AUTH")).unwrap();
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
    db.create_feature(feature("auth", "Auth", "AUTH")).unwrap();
    db.create_task(TaskInput {
        feature: "auth".into(),
        discipline: "backend".into(),
        title: "Task".into(),
        ..Default::default()
    })
    .unwrap();

    let result = db.delete_feature("auth".into());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cannot delete feature"));
}

// === FEATURE RAG FIELDS tests ===

#[test]
fn test_create_feature_with_rag_fields() {
    let db = create_test_db();
    db.create_feature(FeatureInput {
        name: "auth".into(),
        display_name: "Auth".into(),
        acronym: "AUTH".into(),
        description: Some("Authentication feature".into()),
        architecture: Some("OAuth2 + JWT".into()),
        boundaries: Some("No direct DB access".into()),
        dependencies: vec!["user-profile".into()],
        ..Default::default()
    })
    .unwrap();

    let features = db.get_features();
    let f = features.iter().find(|f| f.name == "auth").unwrap();
    assert_eq!(f.architecture, Some("OAuth2 + JWT".into()));
    assert_eq!(f.boundaries, Some("No direct DB access".into()));
    assert_eq!(f.dependencies, vec!["user-profile"]);
    assert!(f.learnings.is_empty()); // Always empty on create
}

#[test]
fn test_update_feature_rag_fields() {
    let db = create_test_db();
    db.create_feature(feature("auth", "Auth", "AUTH")).unwrap();

    db.update_feature(FeatureInput {
        name: "auth".into(),
        display_name: "Auth".into(),
        acronym: "AUTH".into(),
        architecture: Some("Session-based".into()),
        boundaries: Some("Frontend only".into()),
        dependencies: vec!["settings".into()],
        ..Default::default()
    })
    .unwrap();

    let features = db.get_features();
    let f = features.iter().find(|f| f.name == "auth").unwrap();
    assert_eq!(f.architecture, Some("Session-based".into()));
    assert_eq!(f.boundaries, Some("Frontend only".into()));
    assert_eq!(f.dependencies, vec!["settings"]);
}

#[test]
fn test_update_feature_preserves_learnings() {
    let db = create_test_db();
    db.create_feature(feature("auth", "Auth", "AUTH")).unwrap();

    // Add a learning
    let learning = FeatureLearning::from_human("Use bcrypt not SHA256".into(), None);
    db.append_feature_learning("auth", learning, 50).unwrap();

    // Update feature (should NOT touch learnings)
    db.update_feature(FeatureInput {
        name: "auth".into(),
        display_name: "Authentication".into(),
        acronym: "AUTH".into(),
        architecture: Some("New arch".into()),
        ..Default::default()
    })
    .unwrap();

    let features = db.get_features();
    let f = features.iter().find(|f| f.name == "auth").unwrap();
    assert_eq!(f.display_name, "Authentication");
    assert_eq!(f.architecture, Some("New arch".into()));
    assert_eq!(f.learnings.len(), 1);
    assert_eq!(f.learnings[0].text, "Use bcrypt not SHA256");
}

// === FEATURE LEARNING tests ===

#[test]
fn test_append_feature_learning_basic() {
    let db = create_test_db();
    db.create_feature(feature("auth", "Auth", "AUTH")).unwrap();

    let learning = FeatureLearning::auto_extracted("Auth expects User object".into(), 7, Some(42));
    assert!(learning.created.is_empty());

    let added = db.append_feature_learning("auth", learning, 50).unwrap();
    assert!(added);

    let features = db.get_features();
    let f = features.iter().find(|f| f.name == "auth").unwrap();
    assert_eq!(f.learnings.len(), 1);
    assert_eq!(f.learnings[0].text, "Auth expects User object");
    assert_eq!(f.learnings[0].iteration, Some(7));
    assert_eq!(f.learnings[0].task_id, Some(42));
    assert_eq!(f.learnings[0].created, "2026-01-01T00:00:00+00:00");
}

#[test]
fn test_append_feature_learning_dedup() {
    let db = create_test_db();
    db.create_feature(feature("auth", "Auth", "AUTH")).unwrap();

    let learning1 = FeatureLearning::auto_extracted(
        "Auth middleware expects User object not userId string".into(),
        5,
        None,
    );
    db.append_feature_learning("auth", learning1, 50).unwrap();

    // Near-duplicate → should dedup (return false, increment hit_count)
    let learning2 = FeatureLearning::auto_extracted(
        "Auth middleware expects User object instead of userId string".into(),
        8,
        None,
    );
    let added = db.append_feature_learning("auth", learning2, 50).unwrap();
    assert!(!added); // Deduped

    let features = db.get_features();
    let f = features.iter().find(|f| f.name == "auth").unwrap();
    assert_eq!(f.learnings.len(), 1); // Still 1
    assert_eq!(f.learnings[0].hit_count, 2); // Incremented
}

#[test]
fn test_append_feature_learning_nonexistent_feature() {
    let db = create_test_db();
    let learning = FeatureLearning::from_human("test".into(), None);
    let result = db.append_feature_learning("nope", learning, 50);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

// === REMOVE LEARNING tests ===

#[test]
fn test_remove_feature_learning_basic() {
    let db = create_test_db();
    db.create_feature(feature("auth", "Auth", "AUTH")).unwrap();

    db.append_feature_learning(
        "auth",
        FeatureLearning::from_human("First".into(), None),
        50,
    )
    .unwrap();
    db.append_feature_learning(
        "auth",
        FeatureLearning::from_human("Second".into(), None),
        50,
    )
    .unwrap();

    db.remove_feature_learning("auth", 0).unwrap();

    let features = db.get_features();
    let f = features.iter().find(|f| f.name == "auth").unwrap();
    assert_eq!(f.learnings.len(), 1);
    assert_eq!(f.learnings[0].text, "Second");
}

#[test]
fn test_remove_feature_learning_out_of_range() {
    let db = create_test_db();
    db.create_feature(feature("auth", "Auth", "AUTH")).unwrap();

    let result = db.remove_feature_learning("auth", 0);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("out of range"));
}

// === CONTEXT FILE tests ===

#[test]
fn test_add_feature_context_file_basic() {
    let db = create_test_db();
    db.create_feature(feature("auth", "Auth", "AUTH")).unwrap();

    let added = db
        .add_feature_context_file("auth", "src/auth/mod.rs", 100)
        .unwrap();
    assert!(added);

    let features = db.get_features();
    let f = features.iter().find(|f| f.name == "auth").unwrap();
    assert_eq!(f.context_files, vec!["src/auth/mod.rs"]);
}

#[test]
fn test_add_feature_context_file_idempotent() {
    let db = create_test_db();
    db.create_feature(feature("auth", "Auth", "AUTH")).unwrap();

    db.add_feature_context_file("auth", "src/auth/mod.rs", 100)
        .unwrap();
    let added = db
        .add_feature_context_file("auth", "src/auth/mod.rs", 100)
        .unwrap();
    assert!(!added); // Already present

    let features = db.get_features();
    let f = features.iter().find(|f| f.name == "auth").unwrap();
    assert_eq!(f.context_files.len(), 1); // Still 1
}

#[test]
fn test_add_feature_context_file_cap() {
    let db = create_test_db();
    db.create_feature(feature("auth", "Auth", "AUTH")).unwrap();

    // Fill to cap of 2
    db.add_feature_context_file("auth", "a.rs", 2).unwrap();
    db.add_feature_context_file("auth", "b.rs", 2).unwrap();

    let result = db.add_feature_context_file("auth", "c.rs", 2);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("limit"));
}

#[test]
fn test_add_feature_context_file_rejects_absolute_path() {
    let db = create_test_db();
    db.create_feature(feature("auth", "Auth", "AUTH")).unwrap();

    let result = db.add_feature_context_file("auth", "/etc/passwd", 100);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("relative"));
}

#[test]
fn test_add_feature_context_file_rejects_parent_traversal() {
    let db = create_test_db();
    db.create_feature(feature("auth", "Auth", "AUTH")).unwrap();

    let result = db.add_feature_context_file("auth", "../secret.txt", 100);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains(".."));
}

// === DISCIPLINE tests ===

#[test]
fn test_create_discipline() {
    let db = create_test_db();
    db.create_discipline(sqlite_db::DisciplineInput {
        name: "custom".to_owned(),
        display_name: "Custom".to_owned(),
        acronym: "CUST".to_owned(),
        icon: "Wrench".to_owned(),
        color: "#ff0000".to_owned(),
        system_prompt: None,
        skills: "[]".to_owned(),
        conventions: None,
        mcp_servers: "[]".to_owned(),
        image_path: None,
        crops: None,
        description: None,
        image_prompt: None,
    })
    .unwrap();
    let disciplines = db.get_disciplines();
    assert!(disciplines.iter().any(|d| d.name == "custom"));
}

#[test]
fn test_create_duplicate_discipline_rejected() {
    let db = create_test_db();
    // "backend" already seeded
    let result = db.create_discipline(sqlite_db::DisciplineInput {
        name: "backend".to_owned(),
        display_name: "Backend2".to_owned(),
        acronym: "BAC2".to_owned(),
        icon: "Server".to_owned(),
        color: "#000".to_owned(),
        system_prompt: None,
        skills: "[]".to_owned(),
        conventions: None,
        mcp_servers: "[]".to_owned(),
        image_path: None,
        crops: None,
        description: None,
        image_prompt: None,
    });
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already exists"));
}

#[test]
fn test_update_discipline() {
    let db = create_test_db();
    db.create_discipline(sqlite_db::DisciplineInput {
        name: "custom".to_owned(),
        display_name: "Custom".to_owned(),
        acronym: "CUST".to_owned(),
        icon: "Wrench".to_owned(),
        color: "#ff0000".to_owned(),
        system_prompt: None,
        skills: "[]".to_owned(),
        conventions: None,
        mcp_servers: "[]".to_owned(),
        image_path: None,
        crops: None,
        description: None,
        image_prompt: None,
    })
    .unwrap();
    db.update_discipline(sqlite_db::DisciplineInput {
        name: "custom".to_owned(),
        display_name: "Custom Updated".to_owned(),
        acronym: "CUST".to_owned(),
        icon: "Star".to_owned(),
        color: "#00ff00".to_owned(),
        system_prompt: None,
        skills: "[]".to_owned(),
        conventions: None,
        mcp_servers: "[]".to_owned(),
        image_path: None,
        crops: None,
        description: None,
        image_prompt: None,
    })
    .unwrap();

    let disciplines = db.get_disciplines();
    let d = disciplines.iter().find(|d| d.name == "custom").unwrap();
    assert_eq!(d.display_name, "Custom Updated");
    assert_eq!(d.icon, "Star");
    assert_eq!(d.color, "#00ff00");
}

#[test]
fn test_delete_discipline() {
    let db = create_test_db();
    db.create_discipline(sqlite_db::DisciplineInput {
        name: "custom".to_owned(),
        display_name: "Custom".to_owned(),
        acronym: "CUST".to_owned(),
        icon: "Wrench".to_owned(),
        color: "#ff0000".to_owned(),
        system_prompt: None,
        skills: "[]".to_owned(),
        conventions: None,
        mcp_servers: "[]".to_owned(),
        image_path: None,
        crops: None,
        description: None,
        image_prompt: None,
    })
    .unwrap();
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
    db.create_feature(feature("test", "Test", "TEST")).unwrap();
    db.create_task(TaskInput {
        feature: "test".into(),
        discipline: "backend".into(),
        title: "Task".into(),
        ..Default::default()
    })
    .unwrap();

    let result = db.delete_discipline("backend".into());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cannot delete discipline"));
}

// === COMMENT tests ===

#[test]
fn test_add_human_comment() {
    let db = create_test_db();
    db.create_feature(feature("test", "Test", "TEST")).unwrap();
    let task_id = db
        .create_task(TaskInput {
            feature: "test".into(),
            discipline: "backend".into(),
            title: "Task".into(),
            ..Default::default()
        })
        .unwrap();

    db.add_comment(
        task_id,
        "human".to_owned(),
        None,
        None,
        None,
        "Use bcrypt".into(),
    )
    .unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.comments.len(), 1);
    assert_eq!(task.comments[0].author, "human".to_owned());
    assert_eq!(task.comments[0].body, "Use bcrypt");
    assert!(task.comments[0].discipline.is_none());
    assert!(task.comments[0].agent_task_id.is_none());
    assert_eq!(
        task.comments[0].created,
        Some("2026-01-01T00:00:00Z".into())
    );
}

#[test]
fn test_add_agent_comment() {
    let db = create_test_db();
    db.create_feature(feature("test", "Test", "TEST")).unwrap();
    let task_id = db
        .create_task(TaskInput {
            feature: "test".into(),
            discipline: "backend".into(),
            title: "Task".into(),
            ..Default::default()
        })
        .unwrap();

    db.add_comment(
        task_id,
        "agent".to_owned(),
        None,
        Some(5),
        None,
        "Failed: missing .env".into(),
    )
    .unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.comments[0].author, "agent".to_owned());
    assert_eq!(task.comments[0].agent_task_id, Some(5));
}

#[test]
fn test_add_comment_empty_body_rejected() {
    let db = create_test_db();
    db.create_feature(feature("test", "Test", "TEST")).unwrap();
    let task_id = db
        .create_task(TaskInput {
            feature: "test".into(),
            discipline: "backend".into(),
            title: "Task".into(),
            ..Default::default()
        })
        .unwrap();

    let result = db.add_comment(task_id, "human".to_owned(), None, None, None, "   ".into());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot be empty"));
}

#[test]
fn test_add_comment_empty_author_rejected() {
    let db = create_test_db();
    db.create_feature(feature("test", "Test", "TEST")).unwrap();
    let task_id = db
        .create_task(TaskInput {
            feature: "test".into(),
            discipline: "backend".into(),
            title: "Task".into(),
            ..Default::default()
        })
        .unwrap();

    let result = db.add_comment(task_id, "   ".to_owned(), None, None, None, "Note".into());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("author cannot be empty"));
}

#[test]
fn test_add_comment_discipline_author() {
    let db = create_test_db();
    db.create_feature(feature("test", "Test", "TEST")).unwrap();
    let task_id = db
        .create_task(TaskInput {
            feature: "test".into(),
            discipline: "backend".into(),
            title: "Task".into(),
            ..Default::default()
        })
        .unwrap();

    db.add_comment(
        task_id,
        "backend".to_owned(),
        Some("backend".to_owned()),
        Some(1),
        None,
        "Note from backend".into(),
    )
    .unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.comments[0].author, "backend");
    assert_eq!(task.comments[0].discipline, Some("backend".to_owned()));
}

#[test]
fn test_add_comment_nonexistent_task() {
    let db = create_test_db();
    let result = db.add_comment(999, "human".to_owned(), None, None, None, "Hello".into());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn test_update_comment_by_id() {
    let db = create_test_db();
    db.create_feature(feature("test", "Test", "TEST")).unwrap();
    let task_id = db
        .create_task(TaskInput {
            feature: "test".into(),
            discipline: "backend".into(),
            title: "Task".into(),
            ..Default::default()
        })
        .unwrap();

    db.add_comment(
        task_id,
        "human".to_owned(),
        None,
        None,
        None,
        "Original".into(),
    )
    .unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    let comment_id = task.comments[0].id;

    db.update_comment(task_id, comment_id, "Edited".into())
        .unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.comments[0].body, "Edited");
    assert_eq!(task.comments[0].author, "human".to_owned()); // Preserved
}

#[test]
fn test_delete_comment_by_id() {
    let db = create_test_db();
    db.create_feature(feature("test", "Test", "TEST")).unwrap();
    let task_id = db
        .create_task(TaskInput {
            feature: "test".into(),
            discipline: "backend".into(),
            title: "Task".into(),
            ..Default::default()
        })
        .unwrap();

    db.add_comment(
        task_id,
        "human".to_owned(),
        None,
        None,
        None,
        "First".into(),
    )
    .unwrap();
    db.add_comment(
        task_id,
        "human".to_owned(),
        None,
        None,
        None,
        "Second".into(),
    )
    .unwrap();
    db.add_comment(
        task_id,
        "human".to_owned(),
        None,
        None,
        None,
        "Third".into(),
    )
    .unwrap();

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
    db.create_feature(feature("test", "Test", "TEST")).unwrap();
    let task_id = db
        .create_task(TaskInput {
            feature: "test".into(),
            discipline: "backend".into(),
            title: "Original".into(),
            ..Default::default()
        })
        .unwrap();

    db.add_comment(
        task_id,
        "human".to_owned(),
        None,
        None,
        None,
        "Keep me".into(),
    )
    .unwrap();

    db.update_task(
        task_id,
        TaskInput {
            feature: "test".into(),
            discipline: "backend".into(),
            title: "Updated".into(),
            ..Default::default()
        },
    )
    .unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.title, "Updated");
    assert_eq!(task.comments.len(), 1);
    assert_eq!(task.comments[0].body, "Keep me");
}

// === ENRICHED TASKS tests ===

#[test]
fn test_enriched_tasks() {
    let db = create_test_db();
    db.create_feature(FeatureInput {
        name: "auth".into(),
        display_name: "Authentication".into(),
        acronym: "AUTH".into(),
        ..Default::default()
    })
    .unwrap();
    db.create_task(TaskInput {
        feature: "auth".into(),
        discipline: "backend".into(),
        title: "Login".into(),
        ..Default::default()
    })
    .unwrap();

    let enriched = db.get_tasks();
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
    db.create_feature(feature("test", "Test", "TEST")).unwrap();
    db.create_task(TaskInput {
        feature: "test".into(),
        discipline: "backend".into(),
        title: "Task".into(),
        ..Default::default()
    })
    .unwrap();
    db.add_comment(
        1,
        "human".to_owned(),
        None,
        None,
        None,
        "Visible in enriched".into(),
    )
    .unwrap();

    let enriched = db.get_tasks();
    assert_eq!(enriched[0].comments.len(), 1);
    assert_eq!(enriched[0].comments[0].body, "Visible in enriched");
}

// === INFERRED STATUS tests ===

#[test]
fn test_inferred_status_ready() {
    let db = create_test_db();
    db.create_feature(feature("test", "Test", "TEST")).unwrap();
    db.create_task(TaskInput {
        feature: "test".into(),
        discipline: "backend".into(),
        title: "Task".into(),
        ..Default::default()
    })
    .unwrap();

    let enriched = db.get_tasks();
    assert_eq!(
        enriched[0].inferred_status,
        sqlite_db::InferredTaskStatus::Ready
    );
}

#[test]
fn test_inferred_status_waiting_on_deps() {
    let db = create_test_db();
    db.create_feature(feature("test", "Test", "TEST")).unwrap();

    let a = db
        .create_task(TaskInput {
            feature: "test".into(),
            discipline: "backend".into(),
            title: "A".into(),
            ..Default::default()
        })
        .unwrap();
    db.create_task(TaskInput {
        feature: "test".into(),
        discipline: "backend".into(),
        title: "B".into(),
        depends_on: vec![a],
        ..Default::default()
    })
    .unwrap();

    let enriched = db.get_tasks();
    let b_enriched = enriched.iter().find(|t| t.title == "B").unwrap();
    assert_eq!(
        b_enriched.inferred_status,
        sqlite_db::InferredTaskStatus::WaitingOnDeps
    );
}

// === STATS tests ===

// === METADATA tests ===

#[test]
fn test_project_info() {
    let db = create_test_db();
    db.initialize_metadata("My Project".into(), Some("Description".into()))
        .unwrap();

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
    db.create_feature(feature("auth", "Auth", "AUTH")).unwrap();
    db.create_task(TaskInput {
        feature: "auth".into(),
        discipline: "backend".into(),
        title: "Login".into(),
        ..Default::default()
    })
    .unwrap();

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
    db.create_feature(feature("auth", "Auth", "AUTH")).unwrap();

    let task_id = db
        .create_task(TaskInput {
            feature: "auth".into(),
            discipline: "backend".into(),
            title: "Initial".into(),
            priority: Some(Priority::Low),
            ..Default::default()
        })
        .unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.title, "Initial");
    assert_eq!(task.status, TaskStatus::Pending);

    db.update_task(
        task_id,
        TaskInput {
            feature: "auth".into(),
            discipline: "frontend".into(),
            title: "Updated".into(),
            priority: Some(Priority::High),
            tags: vec!["updated".into()],
            ..Default::default()
        },
    )
    .unwrap();

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
    db.create_feature(feature("auth", "Auth", "AUTH")).unwrap();
    let task_id = db
        .create_task(TaskInput {
            feature: "auth".into(),
            discipline: "backend".into(),
            title: "Login flow".into(),
            ..Default::default()
        })
        .unwrap();

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
    db.create_feature(FeatureInput {
        name: "test".into(),
        display_name: "Test \"Feature\"".into(),
        acronym: "TSTF".into(),
        description: Some("A description with \"quotes\" and\nnewlines".into()),
        ..Default::default()
    })
    .unwrap();
    let task_id = db
        .create_task(TaskInput {
            feature: "test".into(),
            discipline: "backend".into(),
            title: "Fix the \"bug\" in code".into(),
            description: Some("Line 1\nLine 2\tTabbed".into()),
            status: None,
            priority: None,
            tags: vec!["tag with \"quotes\"".into()],
            depends_on: vec![],
            acceptance_criteria: Some(vec!["Check \"output\" is correct".into()]),
            context_files: vec![],
            output_artifacts: vec![],
            hints: None,
            estimated_turns: None,
            provenance: None,
        })
        .unwrap();
    db.add_comment(
        task_id,
        "human".to_owned(),
        None,
        None,
        None,
        "Comment with \"quotes\" and\nnewlines".into(),
    )
    .unwrap();

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
