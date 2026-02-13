use sqlite_db::{
    AddSubsystemCommentInput, FixedClock, Priority, SqliteDb, SubsystemInput, TaskInput, TaskStatus,
};

fn comment(subsystem: &str, category: &str, body: &str) -> AddSubsystemCommentInput {
    AddSubsystemCommentInput {
        subsystem_name: subsystem.to_owned(),
        category: category.to_owned(),
        discipline: None,
        agent_task_id: None,
        body: body.to_owned(),
        summary: None,
        reason: None,
        source_iteration: None,
    }
}

fn create_test_db() -> SqliteDb {
    let clock = Box::new(FixedClock(
        chrono::NaiveDate::from_ymd_opt(2026, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc(),
    ));
    let db = SqliteDb::open_in_memory(Some(clock)).unwrap();
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
            agent: None,
            model: None,
            effort: None,
            thinking: None,
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

fn subsystem(name: &str, display_name: &str, acronym: &str) -> SubsystemInput {
    SubsystemInput {
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
    db.create_subsystem(subsystem("auth", "Auth", "AUTH"))
        .unwrap();

    let task_id = db
        .create_task(TaskInput {
            subsystem: "auth".into(),
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
fn test_create_task_empty_subsystem_rejected() {
    let db = create_test_db();
    let result = db.create_task(TaskInput {
        subsystem: "   ".into(),
        discipline: "backend".into(),
        title: "Task".into(),
        ..Default::default()
    });
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("Subsystem name cannot be empty"));
}

#[test]
fn test_create_task_empty_discipline_rejected() {
    let db = create_test_db();
    let result = db.create_task(TaskInput {
        subsystem: "test".into(),
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
        subsystem: "test".into(),
        discipline: "backend".into(),
        title: "     ".into(),
        ..Default::default()
    });
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Task title cannot be empty"));
}

#[test]
fn test_create_task_nonexistent_subsystem_rejected() {
    let db = create_test_db();
    let result = db.create_task(TaskInput {
        subsystem: "nope".into(),
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
    db.create_subsystem(subsystem("test", "Test", "TEST"))
        .unwrap();
    let result = db.create_task(TaskInput {
        subsystem: "test".into(),
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
    db.create_subsystem(subsystem("test", "Test", "TEST"))
        .unwrap();

    let task_id = db
        .create_task(TaskInput {
            subsystem: "test".into(),
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
    db.create_subsystem(subsystem("auth", "Auth", "AUTH"))
        .unwrap();

    let task_id = db
        .create_task(TaskInput {
            subsystem: "auth".into(),
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
            subsystem: "auth".into(),
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
    db.create_subsystem(subsystem("test", "Test", "TEST"))
        .unwrap();
    let result = db.update_task(
        999,
        TaskInput {
            subsystem: "test".into(),
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
    db.create_subsystem(subsystem("test", "Test", "TEST"))
        .unwrap();

    let task_id = db
        .create_task(TaskInput {
            subsystem: "test".into(),
            discipline: "backend".into(),
            title: "Task".into(),
            ..Default::default()
        })
        .unwrap();

    let result = db.update_task(
        task_id,
        TaskInput {
            subsystem: "test".into(),
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
    db.create_subsystem(subsystem("test", "Test", "TEST"))
        .unwrap();

    let a = db
        .create_task(TaskInput {
            subsystem: "test".into(),
            discipline: "backend".into(),
            title: "A".into(),
            ..Default::default()
        })
        .unwrap();

    let b = db
        .create_task(TaskInput {
            subsystem: "test".into(),
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
            subsystem: "test".into(),
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
    db.create_subsystem(subsystem("test", "Test", "TEST"))
        .unwrap();

    let a = db
        .create_task(TaskInput {
            subsystem: "test".into(),
            discipline: "backend".into(),
            title: "A".into(),
            ..Default::default()
        })
        .unwrap();
    let b = db
        .create_task(TaskInput {
            subsystem: "test".into(),
            discipline: "backend".into(),
            title: "B".into(),
            depends_on: vec![a],
            ..Default::default()
        })
        .unwrap();
    let c = db
        .create_task(TaskInput {
            subsystem: "test".into(),
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
            subsystem: "test".into(),
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
    db.create_subsystem(subsystem("test", "Test", "TEST"))
        .unwrap();

    let task_id = db
        .create_task(TaskInput {
            subsystem: "test".into(),
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
            subsystem: "test".into(),
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
    db.create_subsystem(subsystem("test", "Test", "TEST"))
        .unwrap();

    let task_id = db
        .create_task(TaskInput {
            subsystem: "test".into(),
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
    db.create_subsystem(subsystem("test", "Test", "TEST"))
        .unwrap();

    let a = db
        .create_task(TaskInput {
            subsystem: "test".into(),
            discipline: "backend".into(),
            title: "A".into(),
            ..Default::default()
        })
        .unwrap();
    db.create_task(TaskInput {
        subsystem: "test".into(),
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
    db.create_subsystem(subsystem("test", "Test", "TEST"))
        .unwrap();

    let a = db
        .create_task(TaskInput {
            subsystem: "test".into(),
            discipline: "backend".into(),
            title: "A".into(),
            ..Default::default()
        })
        .unwrap();
    let b = db
        .create_task(TaskInput {
            subsystem: "test".into(),
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
fn test_create_subsystem() {
    let db = create_test_db();
    db.create_subsystem(SubsystemInput {
        name: "auth".into(),
        display_name: "Auth".into(),
        acronym: "AUTH".into(),
        description: Some("Auth subsystem".into()),
    })
    .unwrap();
    let subsystems = db.get_subsystems();
    let f = subsystems.iter().find(|f| f.name == "auth").unwrap();
    assert_eq!(f.created, Some("2026-01-01".into()));
}

#[test]
fn test_create_duplicate_subsystem_rejected() {
    let db = create_test_db();
    db.create_subsystem(subsystem("auth", "Auth", "AUTH"))
        .unwrap();
    let result = db.create_subsystem(subsystem("auth", "Auth2", "AUT2"));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already exists"));
}

#[test]
fn test_create_subsystem_duplicate_acronym_rejected() {
    let db = create_test_db();
    db.create_subsystem(subsystem("auth", "Auth", "AUTH"))
        .unwrap();
    let result = db.create_subsystem(subsystem("other", "Other", "AUTH"));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already used"));
}

#[test]
fn test_update_subsystem() {
    let db = create_test_db();
    db.create_subsystem(subsystem("auth", "Auth", "AUTH"))
        .unwrap();
    db.update_subsystem(SubsystemInput {
        name: "auth".into(),
        display_name: "Authentication".into(),
        acronym: "AUTH".into(),
        description: Some("Updated".into()),
    })
    .unwrap();

    let subsystems = db.get_subsystems();
    let f = subsystems.iter().find(|f| f.name == "auth").unwrap();
    assert_eq!(f.display_name, "Authentication");
    assert_eq!(f.description, Some("Updated".into()));
    assert!(f.created.is_some()); // Preserved
}

#[test]
fn test_delete_subsystem() {
    let db = create_test_db();
    db.create_subsystem(subsystem("auth", "Auth", "AUTH"))
        .unwrap();
    let initial = db.get_subsystems().len();
    db.delete_subsystem("auth".into()).unwrap();
    assert_eq!(db.get_subsystems().len(), initial - 1);
}

#[test]
fn test_delete_subsystem_nonexistent() {
    let db = create_test_db();
    let result = db.delete_subsystem("nope".into());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn test_delete_subsystem_with_tasks_rejected() {
    let db = create_test_db();
    db.create_subsystem(subsystem("auth", "Auth", "AUTH"))
        .unwrap();
    db.create_task(TaskInput {
        subsystem: "auth".into(),
        discipline: "backend".into(),
        title: "Task".into(),
        ..Default::default()
    })
    .unwrap();

    let result = db.delete_subsystem("auth".into());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cannot delete subsystem"));
}

// === FEATURE fields tests ===

#[test]
fn test_update_subsystem_preserves_comments() {
    let db = create_test_db();
    db.create_subsystem(subsystem("auth", "Auth", "AUTH"))
        .unwrap();

    db.add_subsystem_comment(comment("auth", "architecture", "Use bcrypt not SHA256"))
        .unwrap();

    db.update_subsystem(SubsystemInput {
        name: "auth".into(),
        display_name: "Authentication".into(),
        acronym: "AUTH".into(),
        ..Default::default()
    })
    .unwrap();

    let subsystems = db.get_subsystems();
    let f = subsystems.iter().find(|f| f.name == "auth").unwrap();
    assert_eq!(f.display_name, "Authentication");
    assert_eq!(f.comments.len(), 1);
    assert_eq!(f.comments[0].body, "Use bcrypt not SHA256");
}

// === FEATURE COMMENT tests ===

#[test]
fn test_add_subsystem_comment_basic() {
    let db = create_test_db();
    db.create_subsystem(subsystem("auth", "Auth", "AUTH"))
        .unwrap();

    db.add_subsystem_comment(AddSubsystemCommentInput {
        reason: Some("Industry standard".into()),
        ..comment("auth", "architecture", "OAuth2 + JWT flow")
    })
    .unwrap();

    let subsystems = db.get_subsystems();
    let f = subsystems.iter().find(|f| f.name == "auth").unwrap();
    assert_eq!(f.comments.len(), 1);
    assert_eq!(f.comments[0].category, "architecture");
    assert_eq!(f.comments[0].body, "OAuth2 + JWT flow");
    assert_eq!(f.comments[0].reason, Some("Industry standard".into()));
    assert!(f.comments[0].created.is_some());
}

#[test]
fn test_add_subsystem_comment_empty_body_rejected() {
    let db = create_test_db();
    db.create_subsystem(subsystem("auth", "Auth", "AUTH"))
        .unwrap();

    let result = db.add_subsystem_comment(comment("auth", "architecture", "   "));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot be empty"));
}

#[test]
fn test_add_subsystem_comment_empty_category_rejected() {
    let db = create_test_db();
    db.create_subsystem(subsystem("auth", "Auth", "AUTH"))
        .unwrap();

    let result = db.add_subsystem_comment(comment("auth", "  ", "body"));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("category cannot be empty"));
}

#[test]
fn test_add_subsystem_comment_nonexistent_subsystem() {
    let db = create_test_db();
    let result = db.add_subsystem_comment(comment("nope", "architecture", "body"));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn test_update_subsystem_comment() {
    let db = create_test_db();
    db.create_subsystem(subsystem("auth", "Auth", "AUTH"))
        .unwrap();

    db.add_subsystem_comment(comment("auth", "gotcha", "Original"))
        .unwrap();

    let subsystems = db.get_subsystems();
    let comment_id = subsystems
        .iter()
        .find(|f| f.name == "auth")
        .unwrap()
        .comments[0]
        .id;

    db.update_subsystem_comment(
        "auth",
        comment_id,
        "Edited",
        None,
        Some("new reason".into()),
    )
    .unwrap();

    let subsystems = db.get_subsystems();
    let f = subsystems.iter().find(|f| f.name == "auth").unwrap();
    assert_eq!(f.comments[0].body, "Edited");
    assert_eq!(f.comments[0].reason, Some("new reason".into()));
    assert!(f.comments[0].updated.is_some());
}

#[test]
fn test_delete_subsystem_comment() {
    let db = create_test_db();
    db.create_subsystem(subsystem("auth", "Auth", "AUTH"))
        .unwrap();

    db.add_subsystem_comment(comment("auth", "gotcha", "First"))
        .unwrap();
    db.add_subsystem_comment(comment("auth", "convention", "Second"))
        .unwrap();

    let subsystems = db.get_subsystems();
    let f = subsystems.iter().find(|f| f.name == "auth").unwrap();
    let first_id = f.comments.iter().find(|c| c.body == "First").unwrap().id;

    db.delete_subsystem_comment("auth", first_id).unwrap();

    let subsystems = db.get_subsystems();
    let f = subsystems.iter().find(|f| f.name == "auth").unwrap();
    assert_eq!(f.comments.len(), 1);
    assert_eq!(f.comments[0].body, "Second");
}

#[test]
fn test_delete_subsystem_cascades_comments() {
    let db = create_test_db();
    db.create_subsystem(subsystem("auth", "Auth", "AUTH"))
        .unwrap();
    db.add_subsystem_comment(comment("auth", "gotcha", "Note"))
        .unwrap();

    db.delete_subsystem("auth".into()).unwrap();

    // Subsystem and its comments are gone
    assert!(db.get_subsystems().iter().all(|f| f.name != "auth"));
}

// === FEATURE COMMENT EXTENDED tests ===

#[test]
fn test_subsystem_comment_with_all_fields() {
    let db = create_test_db();
    db.create_subsystem(subsystem("auth", "Auth", "AUTH"))
        .unwrap();

    db.add_subsystem_comment(AddSubsystemCommentInput {
        discipline: Some("backend".to_owned()),
        agent_task_id: Some(42),
        reason: Some("Prevents replay attacks".to_owned()),
        source_iteration: Some(3),
        ..comment("auth", "design-decision", "Use nonce-based CSRF tokens")
    })
    .unwrap();

    let f = db
        .get_subsystems()
        .into_iter()
        .find(|f| f.name == "auth")
        .unwrap();
    let c = &f.comments[0];
    assert_eq!(c.discipline, Some("backend".to_owned()));
    assert_eq!(c.agent_task_id, Some(42));
    assert_eq!(c.reason, Some("Prevents replay attacks".to_owned()));
    assert_eq!(c.source_iteration, Some(3));
}

#[test]
fn test_subsystem_comment_update_clears_reason() {
    let db = create_test_db();
    db.create_subsystem(subsystem("auth", "Auth", "AUTH"))
        .unwrap();

    db.add_subsystem_comment(AddSubsystemCommentInput {
        reason: Some("Old reason".to_owned()),
        ..comment("auth", "gotcha", "Watch out for XSS")
    })
    .unwrap();

    let comment_id = db
        .get_subsystems()
        .into_iter()
        .find(|f| f.name == "auth")
        .unwrap()
        .comments[0]
        .id;

    db.update_subsystem_comment("auth", comment_id, "Watch out for XSS", None, None)
        .unwrap();

    let f = db
        .get_subsystems()
        .into_iter()
        .find(|f| f.name == "auth")
        .unwrap();
    assert_eq!(f.comments[0].reason, None);
}

#[test]
fn test_subsystem_comment_ordering() {
    let db = create_test_db();
    db.create_subsystem(subsystem("auth", "Auth", "AUTH"))
        .unwrap();

    db.add_subsystem_comment(comment("auth", "gotcha", "First"))
        .unwrap();
    db.add_subsystem_comment(comment("auth", "convention", "Second"))
        .unwrap();
    db.add_subsystem_comment(comment("auth", "architecture", "Third"))
        .unwrap();

    let f = db
        .get_subsystems()
        .into_iter()
        .find(|f| f.name == "auth")
        .unwrap();
    assert_eq!(f.comments.len(), 3);
    assert_eq!(f.comments[0].body, "Third");
    assert_eq!(f.comments[1].body, "Second");
    assert_eq!(f.comments[2].body, "First");
    assert!(f.comments[0].id > f.comments[1].id);
    assert!(f.comments[1].id > f.comments[2].id);
}

#[test]
fn test_update_nonexistent_subsystem_comment() {
    let db = create_test_db();
    db.create_subsystem(subsystem("auth", "Auth", "AUTH"))
        .unwrap();

    let result = db.update_subsystem_comment("auth", 9999, "new body", None, None);
    assert!(result.is_err());
}

#[test]
fn test_delete_nonexistent_subsystem_comment() {
    let db = create_test_db();
    db.create_subsystem(subsystem("auth", "Auth", "AUTH"))
        .unwrap();

    let result = db.delete_subsystem_comment("auth", 9999);
    assert!(result.is_err());
}

#[test]
fn test_multiple_subsystems_comments_isolated() {
    let db = create_test_db();
    db.create_subsystem(subsystem("auth", "Auth", "AUTH"))
        .unwrap();
    db.create_subsystem(subsystem("billing", "Billing", "BILL"))
        .unwrap();

    db.add_subsystem_comment(comment("auth", "gotcha", "Auth comment"))
        .unwrap();
    db.add_subsystem_comment(comment("billing", "convention", "Billing comment"))
        .unwrap();

    let subsystems = db.get_subsystems();
    let auth = subsystems.iter().find(|f| f.name == "auth").unwrap();
    let billing = subsystems.iter().find(|f| f.name == "billing").unwrap();

    assert_eq!(auth.comments.len(), 1);
    assert_eq!(auth.comments[0].body, "Auth comment");
    assert_eq!(billing.comments.len(), 1);
    assert_eq!(billing.comments[0].body, "Billing comment");
}

// === CONTEXT FILE tests ===

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
        agent: None,
        model: None,
        effort: None,
        thinking: None,
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
        agent: None,
        model: None,
        effort: None,
        thinking: None,
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
        agent: None,
        model: None,
        effort: None,
        thinking: None,
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
        agent: None,
        model: None,
        effort: None,
        thinking: None,
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
        agent: None,
        model: None,
        effort: None,
        thinking: None,
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
    db.create_subsystem(subsystem("test", "Test", "TEST"))
        .unwrap();
    db.create_task(TaskInput {
        subsystem: "test".into(),
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
    db.create_subsystem(subsystem("test", "Test", "TEST"))
        .unwrap();
    let task_id = db
        .create_task(TaskInput {
            subsystem: "test".into(),
            discipline: "backend".into(),
            title: "Task".into(),
            ..Default::default()
        })
        .unwrap();

    db.add_signal(task_id, None, None, None, "Use bcrypt".into())
        .unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.signals.len(), 1);
    assert_eq!(task.signals[0].body, "Use bcrypt");
    assert_eq!(task.signals[0].author, "human");
    assert_eq!(task.signals[0].created, Some("2026-01-01T00:00:00Z".into()));
}

#[test]
fn test_add_agent_comment() {
    let db = create_test_db();
    db.create_subsystem(subsystem("test", "Test", "TEST"))
        .unwrap();
    let task_id = db
        .create_task(TaskInput {
            subsystem: "test".into(),
            discipline: "backend".into(),
            title: "Task".into(),
            ..Default::default()
        })
        .unwrap();

    db.add_signal(
        task_id,
        Some("backend".into()),
        Some(5),
        None,
        "Failed: missing .env".into(),
    )
    .unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.signals[0].author, "human");
}

#[test]
fn test_add_comment_empty_body_rejected() {
    let db = create_test_db();
    db.create_subsystem(subsystem("test", "Test", "TEST"))
        .unwrap();
    let task_id = db
        .create_task(TaskInput {
            subsystem: "test".into(),
            discipline: "backend".into(),
            title: "Task".into(),
            ..Default::default()
        })
        .unwrap();

    let result = db.add_signal(task_id, None, None, None, "   ".into());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot be empty"));
}

#[test]
fn test_add_comment_nonexistent_task() {
    let db = create_test_db();
    let result = db.add_signal(999, None, None, None, "Hello".into());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn test_update_comment_by_id() {
    let db = create_test_db();
    db.create_subsystem(subsystem("test", "Test", "TEST"))
        .unwrap();
    let task_id = db
        .create_task(TaskInput {
            subsystem: "test".into(),
            discipline: "backend".into(),
            title: "Task".into(),
            ..Default::default()
        })
        .unwrap();

    db.add_signal(task_id, None, None, None, "Original".into())
        .unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    let comment_id = task.signals[0].id;

    db.update_signal(task_id, comment_id, "Edited".into())
        .unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.signals[0].body, "Edited");
}

#[test]
fn test_delete_comment_by_id() {
    let db = create_test_db();
    db.create_subsystem(subsystem("test", "Test", "TEST"))
        .unwrap();
    let task_id = db
        .create_task(TaskInput {
            subsystem: "test".into(),
            discipline: "backend".into(),
            title: "Task".into(),
            ..Default::default()
        })
        .unwrap();

    db.add_signal(task_id, None, None, None, "First".into())
        .unwrap();
    db.add_signal(task_id, None, None, None, "Second".into())
        .unwrap();
    db.add_signal(task_id, None, None, None, "Third".into())
        .unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    let second_id = task.signals.iter().find(|c| c.body == "Second").unwrap().id;

    // Delete middle comment by stable ID
    db.delete_signal(task_id, second_id).unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.signals.len(), 2);
    assert_eq!(task.signals[0].body, "Third");
    assert_eq!(task.signals[1].body, "First");
}

#[test]
fn test_update_task_preserves_comments() {
    let db = create_test_db();
    db.create_subsystem(subsystem("test", "Test", "TEST"))
        .unwrap();
    let task_id = db
        .create_task(TaskInput {
            subsystem: "test".into(),
            discipline: "backend".into(),
            title: "Original".into(),
            ..Default::default()
        })
        .unwrap();

    db.add_signal(task_id, None, None, None, "Keep me".into())
        .unwrap();

    db.update_task(
        task_id,
        TaskInput {
            subsystem: "test".into(),
            discipline: "backend".into(),
            title: "Updated".into(),
            ..Default::default()
        },
    )
    .unwrap();

    let task = db.get_task_by_id(task_id).unwrap();
    assert_eq!(task.title, "Updated");
    assert_eq!(task.signals.len(), 1);
    assert_eq!(task.signals[0].body, "Keep me");
}

// === ENRICHED TASKS tests ===

#[test]
fn test_enriched_tasks() {
    let db = create_test_db();
    db.create_subsystem(SubsystemInput {
        name: "auth".into(),
        display_name: "Authentication".into(),
        acronym: "AUTH".into(),
        ..Default::default()
    })
    .unwrap();
    db.create_task(TaskInput {
        subsystem: "auth".into(),
        discipline: "backend".into(),
        title: "Login".into(),
        ..Default::default()
    })
    .unwrap();

    let enriched = db.get_tasks();
    assert_eq!(enriched.len(), 1);
    assert_eq!(enriched[0].subsystem_display_name, "Authentication");
    assert_eq!(enriched[0].subsystem_acronym, "AUTH");
    assert_eq!(enriched[0].discipline_display_name, "Backend");
    assert_eq!(enriched[0].discipline_acronym, "BACK");
    assert_eq!(enriched[0].discipline_icon, "Server");
}

#[test]
fn test_enriched_tasks_comments_visible() {
    let db = create_test_db();
    db.create_subsystem(subsystem("test", "Test", "TEST"))
        .unwrap();
    db.create_task(TaskInput {
        subsystem: "test".into(),
        discipline: "backend".into(),
        title: "Task".into(),
        ..Default::default()
    })
    .unwrap();
    db.add_signal(1, None, None, None, "Visible in enriched".into())
        .unwrap();

    let enriched = db.get_tasks();
    assert_eq!(enriched[0].signals.len(), 1);
    assert_eq!(enriched[0].signals[0].body, "Visible in enriched");
}

// === INFERRED STATUS tests ===

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
    db.create_subsystem(subsystem("auth", "Auth", "AUTH"))
        .unwrap();
    db.create_task(TaskInput {
        subsystem: "auth".into(),
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
    db.create_subsystem(subsystem("auth", "Auth", "AUTH"))
        .unwrap();

    let task_id = db
        .create_task(TaskInput {
            subsystem: "auth".into(),
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
            subsystem: "auth".into(),
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
    db.create_subsystem(subsystem("auth", "Auth", "AUTH"))
        .unwrap();
    let task_id = db
        .create_task(TaskInput {
            subsystem: "auth".into(),
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
    db.create_subsystem(SubsystemInput {
        name: "test".into(),
        display_name: "Test \"Subsystem\"".into(),
        acronym: "TSTF".into(),
        description: Some("A description with \"quotes\" and\nnewlines".into()),
    })
    .unwrap();
    let task_id = db
        .create_task(TaskInput {
            subsystem: "test".into(),
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
            agent: None,
            model: None,
            effort: None,
            thinking: None,
        })
        .unwrap();
    db.add_signal(
        task_id,
        None,
        None,
        None,
        "Comment with \"quotes\" and\nnewlines".into(),
    )
    .unwrap();

    let yaml = db.export_prd_yaml().unwrap();

    // Verify escaped quotes don't break the YAML structure
    assert!(yaml.contains(r#"display_name: "Test \"Subsystem\"""#));
    assert!(yaml.contains(r#"title: "Fix the \"bug\" in code""#));
    assert!(yaml.contains(r#"description: "Line 1\nLine 2\tTabbed""#));
    assert!(yaml.contains(r#"- "tag with \"quotes\"""#));
    assert!(yaml.contains(r#"body: "Comment with \"quotes\" and\nnewlines""#));
    // Verify no unescaped quotes mid-string
    assert!(!yaml.contains("\"bug\""));
}
