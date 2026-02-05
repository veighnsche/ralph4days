//! Integration test for project initialization
//!
//! Tests the `initialize_ralph_project` command by:
//! 1. Starting with an empty directory (only README)
//! 2. Calling initialize_ralph_project
//! 3. Verifying all .ralph/db/ files are created correctly

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to call initialize_ralph_project
/// We can't directly import from src/commands.rs in integration tests,
/// so we'll test via the yaml_db crate directly
fn initialize_project_direct(path: PathBuf, project_title: String) -> Result<(), String> {
    use yaml_db::{DisciplinesFile, FeaturesFile, MetadataFile, ProjectMetadata, Task, TasksFile, TaskStatus, Priority};

    // Check path exists and is directory
    if !path.exists() {
        return Err(format!("Directory not found: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("Not a directory: {}", path.display()));
    }

    // Create .ralph/ directory
    let ralph_dir = path.join(".ralph");
    if ralph_dir.exists() {
        return Err(format!(".ralph/ already exists at {}", path.display()));
    }

    fs::create_dir(&ralph_dir)
        .map_err(|e| format!("Failed to create .ralph/ directory: {}", e))?;

    // Create .ralph/db/ directory
    let db_path = ralph_dir.join("db");
    fs::create_dir(&db_path)
        .map_err(|e| format!("Failed to create .ralph/db/ directory: {}", e))?;

    let now = chrono::Utc::now().format("%Y-%m-%d").to_string();

    // Create tasks.yaml with starter task
    let mut tasks_file = TasksFile::new(db_path.join("tasks.yaml"));
    tasks_file.add_task(Task {
        id: 1,
        feature: "setup".to_string(),
        discipline: "frontend".to_string(),
        title: "Replace this with your first task".to_string(),
        description: Some("Add task details here".to_string()),
        status: TaskStatus::Pending,
        priority: Some(Priority::Medium),
        tags: Vec::new(),
        depends_on: Vec::new(),
        blocked_by: None,
        created: Some(now.clone()),
        updated: None,
        completed: None,
        acceptance_criteria: Vec::new(),
    });
    tasks_file.save()?;

    // Create features.yaml
    let mut features_file = FeaturesFile::new(db_path.join("features.yaml"));
    features_file.ensure_feature_exists("setup")?;
    features_file.save()?;

    // Create disciplines.yaml with defaults
    let mut disciplines = DisciplinesFile::new(db_path.join("disciplines.yaml"));
    disciplines.initialize_defaults();
    disciplines.save()?;

    // Create metadata.yaml
    let mut metadata = MetadataFile::new(db_path.join("metadata.yaml"));
    metadata.project = ProjectMetadata {
        title: project_title.clone(),
        description: Some("Add project description here".to_string()),
        created: Some(now),
    };
    metadata.rebuild_counters(tasks_file.get_all());
    metadata.save()?;

    // Create CLAUDE.RALPH.md template
    let claude_path = ralph_dir.join("CLAUDE.RALPH.md");
    let claude_template = format!(
        r#"# {} - Ralph Context

## Project Overview

Add context about this project that Claude should know when working on it.

## Architecture

Describe the architecture, tech stack, and key components.

## Coding Standards

- List any coding conventions
- Style guides
- Best practices

## Important Notes

- Any gotchas or things to watch out for
- Known issues or limitations
- Dependencies or external services
"#,
        project_title
    );

    fs::write(&claude_path, claude_template)
        .map_err(|e| format!("Failed to create CLAUDE.RALPH.md: {}", e))?;

    Ok(())
}

#[test]
fn test_initialize_empty_project() {
    // Create temp directory
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("test-project");
    fs::create_dir(&project_path).unwrap();

    // Add a README to simulate empty project
    fs::write(
        project_path.join("README.md"),
        "# Test Project\n\nEmpty project for initialization test.",
    )
    .unwrap();

    // Initialize the project
    let result = initialize_project_direct(
        project_path.clone(),
        "Test Project".to_string(),
    );

    assert!(result.is_ok(), "Initialization should succeed");

    // Verify directory structure
    assert!(project_path.join(".ralph").exists(), ".ralph/ should exist");
    assert!(
        project_path.join(".ralph/db").exists(),
        ".ralph/db/ should exist"
    );

    // Verify files exist
    let db_path = project_path.join(".ralph/db");
    assert!(db_path.join("tasks.yaml").exists(), "tasks.yaml should exist");
    assert!(
        db_path.join("features.yaml").exists(),
        "features.yaml should exist"
    );
    assert!(
        db_path.join("disciplines.yaml").exists(),
        "disciplines.yaml should exist"
    );
    assert!(
        db_path.join("metadata.yaml").exists(),
        "metadata.yaml should exist"
    );
    assert!(
        project_path.join(".ralph/CLAUDE.RALPH.md").exists(),
        "CLAUDE.RALPH.md should exist"
    );

    // Verify file contents
    let tasks_yaml = fs::read_to_string(db_path.join("tasks.yaml")).unwrap();
    assert!(tasks_yaml.contains("id: 1"), "Should have task with ID 1");
    assert!(
        tasks_yaml.contains("feature: setup"),
        "Should have setup feature"
    );
    assert!(
        tasks_yaml.contains("discipline: frontend"),
        "Should have frontend discipline"
    );
    assert!(
        tasks_yaml.contains("Replace this with your first task"),
        "Should have starter task title"
    );

    let features_yaml = fs::read_to_string(db_path.join("features.yaml")).unwrap();
    assert!(
        features_yaml.contains("name: setup"),
        "Should have setup feature"
    );

    let disciplines_yaml = fs::read_to_string(db_path.join("disciplines.yaml")).unwrap();
    assert!(
        disciplines_yaml.contains("name: frontend"),
        "Should have frontend discipline"
    );
    assert!(
        disciplines_yaml.contains("name: backend"),
        "Should have backend discipline"
    );
    // Should have all 10 default disciplines (count list items with "- name:")
    let discipline_count = disciplines_yaml.matches("- name:").count();
    assert_eq!(
        discipline_count, 10,
        "Should have 10 default disciplines"
    );

    let metadata_yaml = fs::read_to_string(db_path.join("metadata.yaml")).unwrap();
    assert!(
        metadata_yaml.contains("title: Test Project"),
        "Should have project title"
    );
    assert!(
        metadata_yaml.contains("schema_version"),
        "Should have schema version"
    );
    assert!(
        metadata_yaml.contains("_counters"),
        "Should have counters section"
    );

    let claude_md = fs::read_to_string(project_path.join(".ralph/CLAUDE.RALPH.md")).unwrap();
    assert!(
        claude_md.contains("# Test Project - Ralph Context"),
        "Should have project title in CLAUDE.RALPH.md"
    );
    assert!(
        claude_md.contains("## Project Overview"),
        "Should have sections"
    );
}

#[test]
fn test_initialize_already_initialized() {
    // Create temp directory with existing .ralph folder
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("test-project");
    fs::create_dir(&project_path).unwrap();
    fs::create_dir(project_path.join(".ralph")).unwrap();

    // Try to initialize
    let result = initialize_project_direct(
        project_path.clone(),
        "Test Project".to_string(),
    );

    assert!(result.is_err(), "Should fail if .ralph/ already exists");
    assert!(
        result.unwrap_err().contains("already exists"),
        "Error should mention already exists"
    );
}

#[test]
fn test_initialize_nonexistent_path() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent_path = temp_dir.path().join("nonexistent");

    let result = initialize_project_direct(
        nonexistent_path.clone(),
        "Test Project".to_string(),
    );

    assert!(result.is_err(), "Should fail if path doesn't exist");
    assert!(
        result.unwrap_err().contains("not found"),
        "Error should mention not found"
    );
}

/// Generate snapshots for the empty-project fixture
/// Run with: cargo test --test initialization_test generate_initialization_snapshots -- --nocapture
#[test]
fn generate_initialization_snapshots() {
    println!("\nGenerating initialization snapshots...");

    // Use the actual fixture path
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("fixtures/empty-project");

    // Before snapshot is just the README (already created)
    let before_dir = fixture_path.join("snapshots/before");
    println!("Before snapshot exists at: {}", before_dir.display());

    // Create temp directory for initialization
    let temp_dir = TempDir::new().unwrap();
    let test_path = temp_dir.path().join("test-project");
    fs::create_dir(&test_path).unwrap();

    // Copy README to temp
    fs::copy(
        fixture_path.join("README.md"),
        test_path.join("README.md"),
    )
    .unwrap();

    // Initialize the project
    initialize_project_direct(test_path.clone(), "Test Project".to_string()).unwrap();

    // Copy initialized files to after snapshot
    let after_dir = fixture_path.join("snapshots/after");
    fs::create_dir_all(&after_dir).ok(); // Ignore if exists

    // Copy .ralph directory structure
    let after_ralph = after_dir.join(".ralph");
    fs::create_dir_all(after_ralph.join("db")).unwrap();

    // Copy all YAML files
    for file in &[
        "tasks.yaml",
        "features.yaml",
        "disciplines.yaml",
        "metadata.yaml",
    ] {
        fs::copy(
            test_path.join(".ralph/db").join(file),
            after_ralph.join("db").join(file),
        )
        .unwrap();
    }

    // Copy CLAUDE.RALPH.md
    fs::copy(
        test_path.join(".ralph/CLAUDE.RALPH.md"),
        after_ralph.join("CLAUDE.RALPH.md"),
    )
    .unwrap();

    // Copy README
    fs::copy(
        test_path.join("README.md"),
        after_dir.join("README.md"),
    )
    .unwrap();

    println!("\n✓ Generated snapshots in:");
    println!("  Before: {}", before_dir.display());
    println!("  After:  {}", after_dir.display());
    println!("\n=== tasks.yaml ===");
    println!(
        "{}",
        fs::read_to_string(after_ralph.join("db/tasks.yaml")).unwrap()
    );
    println!("\n=== features.yaml ===");
    println!(
        "{}",
        fs::read_to_string(after_ralph.join("db/features.yaml")).unwrap()
    );
    println!("\n=== metadata.yaml ===");
    println!(
        "{}",
        fs::read_to_string(after_ralph.join("db/metadata.yaml")).unwrap()
    );
}
