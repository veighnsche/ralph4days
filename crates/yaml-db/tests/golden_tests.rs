//! Golden tests for yaml-db crate
//!
//! These tests verify that YAML serialization output matches expected snapshots.
//! Run with: cargo test
//! Update snapshots with: cargo insta review

use std::fs;
use tempfile::TempDir;
use yaml_db::{
    Discipline, DisciplinesFile, FeaturesFile, MetadataFile, Priority, ProjectMetadata, Task,
    TaskInput, TaskStatus, TasksFile, YamlDatabase,
};

/// Helper to create a temporary database directory
fn create_temp_db() -> (TempDir, std::path::PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("db");
    fs::create_dir(&db_path).unwrap();
    (temp_dir, db_path)
}

#[test]
fn test_tasks_yaml_output() {
    let (_temp, db_path) = create_temp_db();
    let mut tasks_file = TasksFile::new(db_path.join("tasks.yaml"));

    // Add sample tasks
    tasks_file.add_task(Task {
        id: 1,
        feature: "authentication".to_string(),
        discipline: "backend".to_string(),
        title: "Implement OAuth2".to_string(),
        description: Some("Add OAuth2 support for Google and GitHub".to_string()),
        status: TaskStatus::InProgress,
        priority: Some(Priority::High),
        tags: vec!["security".to_string(), "auth".to_string()],
        depends_on: vec![],
        blocked_by: None,
        created: Some("2024-01-15".to_string()),
        updated: Some("2024-01-16".to_string()),
        completed: None,
        acceptance_criteria: vec![
            "Users can sign in with Google".to_string(),
            "Users can sign in with GitHub".to_string(),
        ],
    });

    tasks_file.add_task(Task {
        id: 2,
        feature: "authentication".to_string(),
        discipline: "frontend".to_string(),
        title: "Create login UI".to_string(),
        description: None,
        status: TaskStatus::Pending,
        priority: Some(Priority::Medium),
        tags: vec![],
        depends_on: vec![1],
        blocked_by: None,
        created: Some("2024-01-15".to_string()),
        updated: None,
        completed: None,
        acceptance_criteria: vec![],
    });

    tasks_file.save().unwrap();
    let yaml_content = fs::read_to_string(db_path.join("tasks.yaml")).unwrap();
    insta::assert_snapshot!("tasks_yaml", yaml_content);
}

#[test]
fn test_features_yaml_output() {
    let (_temp, db_path) = create_temp_db();
    let mut features_file = FeaturesFile::new(db_path.join("features.yaml"));

    features_file
        .ensure_feature_exists("authentication")
        .unwrap();
    features_file.ensure_feature_exists("user-profile").unwrap();

    features_file.save().unwrap();
    let yaml_content = fs::read_to_string(db_path.join("features.yaml")).unwrap();
    insta::assert_snapshot!("features_yaml", yaml_content);
}

#[test]
fn test_disciplines_yaml_output() {
    let (_temp, db_path) = create_temp_db();
    let mut disciplines_file = DisciplinesFile::new(db_path.join("disciplines.yaml"));

    // Load defaults
    disciplines_file.load_with_defaults().unwrap();

    disciplines_file.save().unwrap();
    let yaml_content = fs::read_to_string(db_path.join("disciplines.yaml")).unwrap();
    insta::assert_snapshot!("disciplines_yaml_defaults", yaml_content);
}

#[test]
fn test_disciplines_yaml_custom() {
    let (_temp, db_path) = create_temp_db();
    let mut disciplines_file = DisciplinesFile::new(db_path.join("disciplines.yaml"));

    // Add custom disciplines
    disciplines_file.add(Discipline {
        name: "ml-ops".to_string(),
        display_name: "ML/Ops".to_string(),
        icon: "Brain".to_string(),
        color: "violet".to_string(),
    });

    disciplines_file.add(Discipline {
        name: "data-eng".to_string(),
        display_name: "Data Engineering".to_string(),
        icon: "Database".to_string(),
        color: "emerald".to_string(),
    });

    disciplines_file.save().unwrap();
    let yaml_content = fs::read_to_string(db_path.join("disciplines.yaml")).unwrap();
    insta::assert_snapshot!("disciplines_yaml_custom", yaml_content);
}

#[test]
fn test_metadata_yaml_output() {
    let (_temp, db_path) = create_temp_db();
    let mut metadata_file = MetadataFile::new(db_path.join("metadata.yaml"));

    metadata_file.project = ProjectMetadata {
        title: "My Awesome Project".to_string(),
        description: Some("A revolutionary new application".to_string()),
        created: Some("2024-01-15".to_string()),
    };

    // Simulate task counters
    let tasks = vec![
        Task {
            id: 1,
            feature: "auth".to_string(),
            discipline: "backend".to_string(),
            title: "Task 1".to_string(),
            description: None,
            status: TaskStatus::Done,
            priority: None,
            tags: vec![],
            depends_on: vec![],
            blocked_by: None,
            created: None,
            updated: None,
            completed: None,
            acceptance_criteria: vec![],
        },
        Task {
            id: 2,
            feature: "auth".to_string(),
            discipline: "frontend".to_string(),
            title: "Task 2".to_string(),
            description: None,
            status: TaskStatus::Pending,
            priority: None,
            tags: vec![],
            depends_on: vec![],
            blocked_by: None,
            created: None,
            updated: None,
            completed: None,
            acceptance_criteria: vec![],
        },
    ];

    metadata_file.rebuild_counters(&tasks);
    metadata_file.save().unwrap();

    let yaml_content = fs::read_to_string(db_path.join("metadata.yaml")).unwrap();
    insta::assert_snapshot!("metadata_yaml", yaml_content);
}

#[test]
fn test_full_database_creation() {
    let (_temp, db_path) = create_temp_db();
    let mut db = YamlDatabase::from_path(db_path.clone()).unwrap();

    // Create tasks with automatic feature/discipline creation
    let task_id_1 = db
        .create_task(TaskInput {
            feature: "user-management".to_string(),
            discipline: "backend".to_string(),
            title: "Create user API endpoints".to_string(),
            description: Some("Implement CRUD operations for users".to_string()),
            priority: Some(Priority::High),
            tags: vec!["api".to_string()],
            depends_on: vec![],
            acceptance_criteria: Some(vec![
                "POST /users creates a user".to_string(),
                "GET /users/:id returns user".to_string(),
            ]),
        })
        .unwrap();

    let _task_id_2 = db
        .create_task(TaskInput {
            feature: "user-management".to_string(),
            discipline: "frontend".to_string(),
            title: "Build user profile page".to_string(),
            description: None,
            priority: Some(Priority::Medium),
            tags: vec![],
            depends_on: vec![task_id_1],
            acceptance_criteria: None,
        })
        .unwrap();

    // Verify all files exist and have correct content
    let tasks_yaml = fs::read_to_string(db_path.join("tasks.yaml")).unwrap();
    let features_yaml = fs::read_to_string(db_path.join("features.yaml")).unwrap();
    let disciplines_yaml = fs::read_to_string(db_path.join("disciplines.yaml")).unwrap();
    let metadata_yaml = fs::read_to_string(db_path.join("metadata.yaml")).unwrap();

    insta::assert_snapshot!("full_db_tasks", tasks_yaml);
    insta::assert_snapshot!("full_db_features", features_yaml);
    insta::assert_snapshot!("full_db_disciplines", disciplines_yaml);
    insta::assert_snapshot!("full_db_metadata", metadata_yaml);
}

#[test]
fn test_task_with_all_fields() {
    let (_temp, db_path) = create_temp_db();
    let mut tasks_file = TasksFile::new(db_path.join("tasks.yaml"));

    tasks_file.add_task(Task {
        id: 42,
        feature: "payments".to_string(),
        discipline: "backend".to_string(),
        title: "Integrate Stripe payment gateway".to_string(),
        description: Some("Add Stripe integration for subscription payments".to_string()),
        status: TaskStatus::InProgress,
        priority: Some(Priority::Critical),
        tags: vec![
            "payments".to_string(),
            "stripe".to_string(),
            "billing".to_string(),
        ],
        depends_on: vec![10, 15, 20],
        blocked_by: Some("Waiting for legal approval".to_string()),
        created: Some("2024-01-10".to_string()),
        updated: Some("2024-01-20".to_string()),
        completed: None,
        acceptance_criteria: vec![
            "Customers can subscribe to monthly plan".to_string(),
            "Customers can subscribe to annual plan".to_string(),
            "Webhook handles payment success".to_string(),
            "Webhook handles payment failure".to_string(),
        ],
    });

    tasks_file.save().unwrap();
    let yaml_content = fs::read_to_string(db_path.join("tasks.yaml")).unwrap();
    insta::assert_snapshot!("task_all_fields", yaml_content);
}

#[test]
fn test_task_minimal_fields() {
    let (_temp, db_path) = create_temp_db();
    let mut tasks_file = TasksFile::new(db_path.join("tasks.yaml"));

    tasks_file.add_task(Task {
        id: 1,
        feature: "simple".to_string(),
        discipline: "backend".to_string(),
        title: "Simple task".to_string(),
        description: None,
        status: TaskStatus::Pending,
        priority: None,
        tags: vec![],
        depends_on: vec![],
        blocked_by: None,
        created: None,
        updated: None,
        completed: None,
        acceptance_criteria: vec![],
    });

    tasks_file.save().unwrap();
    let yaml_content = fs::read_to_string(db_path.join("tasks.yaml")).unwrap();
    insta::assert_snapshot!("task_minimal_fields", yaml_content);
}

#[test]
fn test_task_status_serialization() {
    let (_temp, db_path) = create_temp_db();
    let mut tasks_file = TasksFile::new(db_path.join("tasks.yaml"));

    // Test all task statuses
    for (id, status) in [
        (1, TaskStatus::Pending),
        (2, TaskStatus::InProgress),
        (3, TaskStatus::Done),
        (4, TaskStatus::Blocked),
        (5, TaskStatus::Skipped),
    ] {
        tasks_file.add_task(Task {
            id,
            feature: "test".to_string(),
            discipline: "backend".to_string(),
            title: format!("Task with status {:?}", status),
            description: None,
            status,
            priority: None,
            tags: vec![],
            depends_on: vec![],
            blocked_by: None,
            created: None,
            updated: None,
            completed: None,
            acceptance_criteria: vec![],
        });
    }

    tasks_file.save().unwrap();
    let yaml_content = fs::read_to_string(db_path.join("tasks.yaml")).unwrap();
    insta::assert_snapshot!("task_statuses", yaml_content);
}

#[test]
fn test_task_priority_serialization() {
    let (_temp, db_path) = create_temp_db();
    let mut tasks_file = TasksFile::new(db_path.join("tasks.yaml"));

    // Test all priorities
    for (id, priority) in [
        (1, Some(Priority::Low)),
        (2, Some(Priority::Medium)),
        (3, Some(Priority::High)),
        (4, Some(Priority::Critical)),
        (5, None),
    ] {
        tasks_file.add_task(Task {
            id,
            feature: "test".to_string(),
            discipline: "backend".to_string(),
            title: format!("Task with priority {:?}", priority),
            description: None,
            status: TaskStatus::Pending,
            priority,
            tags: vec![],
            depends_on: vec![],
            blocked_by: None,
            created: None,
            updated: None,
            completed: None,
            acceptance_criteria: vec![],
        });
    }

    tasks_file.save().unwrap();
    let yaml_content = fs::read_to_string(db_path.join("tasks.yaml")).unwrap();
    insta::assert_snapshot!("task_priorities", yaml_content);
}

#[test]
fn test_metadata_without_counters() {
    let (_temp, db_path) = create_temp_db();
    let metadata_file = MetadataFile::new(db_path.join("metadata.yaml"));

    metadata_file.save().unwrap();
    let yaml_content = fs::read_to_string(db_path.join("metadata.yaml")).unwrap();

    // Counters should not appear in YAML when empty (skip_serializing_if)
    assert!(!yaml_content.contains("_counters"));
    insta::assert_snapshot!("metadata_no_counters", yaml_content);
}

#[test]
fn test_counter_rebuild() {
    let (_temp, db_path) = create_temp_db();
    let mut metadata_file = MetadataFile::new(db_path.join("metadata.yaml"));

    let tasks = vec![
        Task {
            id: 5,
            feature: "auth".to_string(),
            discipline: "frontend".to_string(),
            title: "Task".to_string(),
            description: None,
            status: TaskStatus::Done,
            priority: None,
            tags: vec![],
            depends_on: vec![],
            blocked_by: None,
            created: None,
            updated: None,
            completed: None,
            acceptance_criteria: vec![],
        },
        Task {
            id: 10,
            feature: "auth".to_string(),
            discipline: "backend".to_string(),
            title: "Task".to_string(),
            description: None,
            status: TaskStatus::Done,
            priority: None,
            tags: vec![],
            depends_on: vec![],
            blocked_by: None,
            created: None,
            updated: None,
            completed: None,
            acceptance_criteria: vec![],
        },
        Task {
            id: 3,
            feature: "payments".to_string(),
            discipline: "backend".to_string(),
            title: "Task".to_string(),
            description: None,
            status: TaskStatus::Done,
            priority: None,
            tags: vec![],
            depends_on: vec![],
            blocked_by: None,
            created: None,
            updated: None,
            completed: None,
            acceptance_criteria: vec![],
        },
    ];

    metadata_file.rebuild_counters(&tasks);
    metadata_file.save().unwrap();

    let yaml_content = fs::read_to_string(db_path.join("metadata.yaml")).unwrap();
    insta::assert_snapshot!("metadata_with_counters", yaml_content);
}
