//! Multi-file YAML database for managing PRD tasks, features, disciplines, and metadata
//!
//! This crate provides a thread-safe, file-based database using YAML files.
//! It replaces a single prd.yaml with 4 separate files:
//! - tasks.yaml: Task records
//! - features.yaml: Feature definitions
//! - disciplines.yaml: Discipline definitions
//! - metadata.yaml: Project metadata and counters

use serde::{Deserialize, Serialize};

mod database;
mod disciplines;
mod entity;
mod features;
mod metadata;
mod tasks;

// Re-export public types
pub use database::{TaskInput, YamlDatabase};
pub use disciplines::{Discipline, DisciplinesFile};
pub use features::{Feature, FeaturesFile};
pub use metadata::{MetadataFile, ProjectMetadata};
pub use tasks::TasksFile;

// Core data types

/// Task status enum
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Done,
    Blocked,
    Skipped,
}

/// Task priority enum
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Task record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub feature: String,
    pub discipline: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: TaskStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<Priority>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub acceptance_criteria: Vec<String>,
}
