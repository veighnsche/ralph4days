//! Helper to generate transformation test snapshots
//! Run with: cargo test --test generate_snapshots -- --nocapture

use std::fs;
use tempfile::TempDir;
use yaml_db::{Priority, TaskInput, YamlDatabase};

#[test]
fn generate_empty_to_first_task_after() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("db");
    fs::create_dir_all(&db_path).unwrap();

    // Copy before snapshots
    let before_dir = "tests/snapshots/transformations/empty_to_first_task/before";
    for file in &["tasks.yaml", "features.yaml", "disciplines.yaml", "metadata.yaml"] {
        fs::copy(
            format!("{}/{}", before_dir, file),
            db_path.join(file),
        )
        .unwrap();
    }

    // Perform transformation
    let mut db = YamlDatabase::from_path(db_path.clone()).unwrap();
    db.create_feature("authentication".to_string(), "Authentication".to_string(), "AUTH".to_string(), None).unwrap();
    db.create_task(TaskInput {
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

    // Print actual output
    println!("\n=== tasks.yaml ===");
    let tasks = fs::read_to_string(db_path.join("tasks.yaml")).unwrap();
    println!("{}", tasks);

    println!("\n=== features.yaml ===");
    let features = fs::read_to_string(db_path.join("features.yaml")).unwrap();
    println!("{}", features);

    println!("\n=== metadata.yaml ===");
    let metadata = fs::read_to_string(db_path.join("metadata.yaml")).unwrap();
    println!("{}", metadata);

    // Write to after directory
    let after_dir = "tests/snapshots/transformations/empty_to_first_task/after";
    fs::write(format!("{}/tasks.yaml", after_dir), tasks).unwrap();
    fs::write(format!("{}/features.yaml", after_dir), features).unwrap();
    fs::write(format!("{}/metadata.yaml", after_dir), metadata).unwrap();

    println!("\n✓ Generated snapshots in {}", after_dir);
}

#[test]
fn generate_all_transformation_snapshots() {
    // This will generate all snapshots by running the actual transformations
    println!("Generating all transformation snapshots...");

    // Test 1: Empty to first task - already done above

    // Test 2: Add dependent task
    {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("db");
        fs::create_dir_all(&db_path).unwrap();

        // Create before state with one task
        let mut db = YamlDatabase::from_path(db_path.clone()).unwrap();
        db.create_feature("authentication".to_string(), "Authentication".to_string(), "AUTH".to_string(), None).unwrap();
        db.create_task(TaskInput {
            feature: "authentication".to_string(),
            discipline: "backend".to_string(),
            title: "Implement login API".to_string(),
            description: None,
            priority: Some(Priority::High),
            tags: vec![],
            depends_on: vec![],
            acceptance_criteria: None,
            ..Default::default()
        })
        .unwrap();

        // Save before state
        let before_dir = "tests/snapshots/transformations/add_dependent_task/before";
        for file in &["tasks.yaml", "features.yaml", "disciplines.yaml", "metadata.yaml"] {
            fs::copy(db_path.join(file), format!("{}/{}", before_dir, file)).unwrap();
        }

        // Perform transformation - add dependent task
        db.create_task(TaskInput {
            feature: "authentication".to_string(),
            discipline: "frontend".to_string(),
            title: "Build login form".to_string(),
            description: Some("Create UI for user login".to_string()),
            priority: Some(Priority::Medium),
            tags: vec!["ui".to_string()],
            depends_on: vec![1],
            acceptance_criteria: Some(vec!["Form validates input".to_string()]),
            ..Default::default()
        })
        .unwrap();

        // Save after state
        let after_dir = "tests/snapshots/transformations/add_dependent_task/after";
        for file in &["tasks.yaml", "features.yaml", "disciplines.yaml", "metadata.yaml"] {
            fs::copy(db_path.join(file), format!("{}/{}", after_dir, file)).unwrap();
        }

        println!("✓ Generated add_dependent_task snapshots");
    }

    // Test 3: Multi-feature expansion
    {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("db");
        fs::create_dir_all(&db_path).unwrap();

        let mut db = YamlDatabase::from_path(db_path.clone()).unwrap();

        db.create_feature("authentication".to_string(), "Authentication".to_string(), "AUTH".to_string(), None).unwrap();

        // Create initial task
        db.create_task(TaskInput {
            feature: "authentication".to_string(),
            discipline: "backend".to_string(),
            title: "Implement login API".to_string(),
            description: None,
            priority: None,
            tags: vec![],
            depends_on: vec![],
            acceptance_criteria: None,
            ..Default::default()
        }).unwrap();

        // Save before state
        let before_dir = "tests/snapshots/transformations/multi_feature_expansion/before";
        for file in &["tasks.yaml", "features.yaml", "disciplines.yaml", "metadata.yaml"] {
            fs::copy(db_path.join(file), format!("{}/{}", before_dir, file)).unwrap();
        }

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
        }).unwrap();

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
        }).unwrap();

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
        }).unwrap();

        // Save after state
        let after_dir = "tests/snapshots/transformations/multi_feature_expansion/after";
        for file in &["tasks.yaml", "features.yaml", "disciplines.yaml", "metadata.yaml"] {
            fs::copy(db_path.join(file), format!("{}/{}", after_dir, file)).unwrap();
        }

        println!("✓ Generated multi_feature_expansion snapshots");
    }

    // Test 4: Counter rebuild
    {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("db");
        fs::create_dir_all(&db_path).unwrap();

        // Create tasks with specific IDs by creating them in order
        let mut db = YamlDatabase::from_path(db_path.clone()).unwrap();

        db.create_feature("auth".to_string(), "Auth".to_string(), "AUTH".to_string(), None).unwrap();

        // Create 10 tasks to get to ID 10
        for i in 1..=10 {
            let feature = if i <= 5 { "auth" } else if i <= 10 { "auth" } else { "payments" };
            let discipline = if i == 5 { "frontend" } else if i == 10 { "backend" } else { "backend" };
            db.create_task(TaskInput {
                feature: feature.to_string(),
                discipline: discipline.to_string(),
                title: format!("Task {}", i),
                description: None,
                priority: None,
                tags: vec![],
                depends_on: vec![],
                acceptance_criteria: None,
                ..Default::default()
            }).unwrap();
        }

        // Create one for payments at ID 3 position - actually we need to be smarter
        // Let me just create the exact scenario we want

        // Actually, let's manually create the before state with the exact IDs we want
        let before_tasks = r#"tasks:
- id: 5
  feature: auth
  discipline: frontend
  title: Task 1
  status: done
- id: 10
  feature: auth
  discipline: backend
  title: Task 2
  status: done
- id: 3
  feature: payments
  discipline: backend
  title: Task 3
  status: done
"#;
        let before_features = r#"features:
- name: auth
  display_name: Auth
- name: payments
  display_name: Payments
"#;
        let before_metadata = r#"schema_version: '1.0'
project:
  title: Test Project
_counters:
  auth:
    backend: 1
    frontend: 2
  payments:
    backend: 1
"#;

        let before_dir = "tests/snapshots/transformations/counter_rebuild/before";
        fs::write(format!("{}/tasks.yaml", before_dir), before_tasks).unwrap();
        fs::write(format!("{}/features.yaml", before_dir), before_features).unwrap();
        fs::write(format!("{}/metadata.yaml", before_dir), before_metadata).unwrap();

        // For after, counters should be rebuilt correctly
        let after_tasks = before_tasks; // Same tasks
        let after_features = before_features; // Same features
        let after_metadata = r#"schema_version: '1.0'
project:
  title: Test Project
_counters:
  auth:
    backend: 10
    frontend: 5
  payments:
    backend: 3
"#;

        let after_dir = "tests/snapshots/transformations/counter_rebuild/after";
        fs::write(format!("{}/tasks.yaml", after_dir), after_tasks).unwrap();
        fs::write(format!("{}/features.yaml", after_dir), after_features).unwrap();
        fs::write(format!("{}/metadata.yaml", after_dir), after_metadata).unwrap();

        println!("✓ Generated counter_rebuild snapshots");
    }

    // Test 5: Custom discipline
    {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("db");
        fs::create_dir_all(&db_path).unwrap();

        // Set up before state with Test Project metadata
        let before_dir = "tests/snapshots/transformations/custom_discipline/before";
        fs::copy("tests/snapshots/transformations/empty_to_first_task/before/tasks.yaml",
                 format!("{}/tasks.yaml", before_dir)).unwrap();
        fs::copy("tests/snapshots/transformations/empty_to_first_task/before/features.yaml",
                 format!("{}/features.yaml", before_dir)).unwrap();
        fs::copy("tests/snapshots/transformations/empty_to_first_task/before/disciplines.yaml",
                 format!("{}/disciplines.yaml", before_dir)).unwrap();

        let before_metadata = r#"schema_version: '1.0'
project:
  title: Test Project
"#;
        fs::write(format!("{}/metadata.yaml", before_dir), before_metadata).unwrap();

        // Copy before state to temp db
        for file in &["tasks.yaml", "features.yaml", "disciplines.yaml", "metadata.yaml"] {
            fs::copy(format!("{}/{}", before_dir, file), db_path.join(file)).unwrap();
        }

        // Create task with custom discipline
        let mut db = YamlDatabase::from_path(db_path.clone()).unwrap();
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
        }).unwrap();

        // Save after state (including new discipline)
        let after_dir = "tests/snapshots/transformations/custom_discipline/after";
        for file in &["tasks.yaml", "features.yaml", "disciplines.yaml", "metadata.yaml"] {
            fs::copy(db_path.join(file), format!("{}/{}", after_dir, file)).unwrap();
        }

        println!("✓ Generated custom_discipline snapshots");
    }

    // Test 6: Reload and modify
    {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("db");
        fs::create_dir_all(&db_path).unwrap();

        // Set up before state
        let before_dir = "tests/snapshots/transformations/reload_modify/before";
        fs::copy("tests/snapshots/transformations/empty_to_first_task/before/tasks.yaml",
                 format!("{}/tasks.yaml", before_dir)).unwrap();
        fs::copy("tests/snapshots/transformations/empty_to_first_task/before/features.yaml",
                 format!("{}/features.yaml", before_dir)).unwrap();
        fs::copy("tests/snapshots/transformations/empty_to_first_task/before/disciplines.yaml",
                 format!("{}/disciplines.yaml", before_dir)).unwrap();
        fs::copy("tests/snapshots/transformations/empty_to_first_task/before/metadata.yaml",
                 format!("{}/metadata.yaml", before_dir)).unwrap();

        // Copy before state to temp db
        for file in &["tasks.yaml", "features.yaml", "disciplines.yaml", "metadata.yaml"] {
            fs::copy(format!("{}/{}", before_dir, file), db_path.join(file)).unwrap();
        }

        // First operation
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
            }).unwrap();
        }

        // Second operation (reload and add)
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
            }).unwrap();
        }

        // Save after state
        let after_dir = "tests/snapshots/transformations/reload_modify/after";
        for file in &["tasks.yaml", "features.yaml", "disciplines.yaml", "metadata.yaml"] {
            fs::copy(db_path.join(file), format!("{}/{}", after_dir, file)).unwrap();
        }

        println!("✓ Generated reload_modify snapshots");
    }

    println!("\n✅ ALL 6 TRANSFORMATION SNAPSHOTS GENERATED!");
}
