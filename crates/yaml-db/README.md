# yaml-db

A multi-file YAML database for managing PRD (Product Requirements Document) data including tasks, features, disciplines, and metadata.

## Overview

This crate provides a thread-safe, file-based database that replaces a single monolithic `prd.yaml` file with four specialized YAML files:

- **tasks.yaml**: Task records with status, priority, dependencies, and acceptance criteria
- **features.yaml**: Feature definitions
- **disciplines.yaml**: Discipline definitions with display names, icons, and colors
- **metadata.yaml**: Project metadata and ID counters

## Features

- **Thread-safe task creation**: Exclusive file locking prevents race conditions during concurrent operations
- **Atomic writes**: Temp file + rename pattern ensures data consistency
- **Auto-migration**: Seamlessly migrates from legacy single-file format
- **Dependency validation**: Ensures task dependencies reference existing tasks
- **Auto-populate**: Automatically creates features and disciplines when creating tasks
- **Configurable disciplines**: Softcoded discipline system with 10 sensible defaults

## Usage

```rust
use yaml_db::{YamlDatabase, TaskInput, Priority};

// Initialize database from .ralph/db/ directory
let db_path = project_path.join(".ralph").join("db");
let mut db = YamlDatabase::from_path(db_path)?;

// Create a new task (thread-safe with automatic ID assignment)
let task_id = db.create_task(TaskInput {
    feature: "authentication".to_string(),
    discipline: "backend".to_string(),
    title: "Implement OAuth2".to_string(),
    description: Some("Add OAuth2 support for Google and GitHub".to_string()),
    priority: Some(Priority::High),
    tags: vec!["security".to_string()],
    depends_on: vec![],
    acceptance_criteria: Some(vec![
        "Users can sign in with Google".to_string(),
        "Users can sign in with GitHub".to_string(),
    ]),
})?;

// Access data
let tasks = db.get_tasks();
let features = db.get_features();
let disciplines = db.get_disciplines();
let project_info = db.get_project_info();
```

## Default Disciplines

The crate provides 10 default disciplines:

- Frontend (Monitor icon, Blue)
- Backend (Server icon, Green)
- Database (Database icon, Purple)
- DevOps (Cloud icon, Orange)
- Design (Palette icon, Pink)
- Testing (TestTube icon, Yellow)
- Documentation (FileText icon, Gray)
- Security (Shield icon, Red)
- Mobile (Smartphone icon, Cyan)
- API (Webhook icon, Indigo)

## License

MIT OR Apache-2.0
