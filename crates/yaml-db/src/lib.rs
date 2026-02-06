//! Multi-file YAML database for managing PRD tasks, features, disciplines, and metadata
//!
//! This crate provides a thread-safe, file-based database using YAML files.
//! It replaces a single prd.yaml with 4 separate files:
//! - tasks.yaml: Task records
//! - features.yaml: Feature definitions
//! - disciplines.yaml: Discipline definitions
//! - metadata.yaml: Project metadata and counters

use serde::{Deserialize, Serialize};

pub mod acronym;
mod database;
mod disciplines;
mod entity;
mod features;
mod metadata;
mod migration;
mod tasks;

// Re-export public types
pub use database::{TaskInput, YamlDatabase};
pub use disciplines::{Discipline, DisciplinesFile};
pub use features::{Feature, FeaturesFile};
pub use metadata::{MetadataFile, ProjectMetadata};
pub use tasks::TasksFile;
// Re-export enriched types (defined in this file)

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

/// Task with pre-joined feature/discipline display data for IPC.
/// Uses camelCase for JSON serialization (frontend-ready).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrichedTask {
    pub id: u32,
    pub feature: String,
    pub discipline: String,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: Option<Priority>,
    pub tags: Vec<String>,
    pub depends_on: Vec<u32>,
    pub blocked_by: Option<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub completed: Option<String>,
    pub acceptance_criteria: Vec<String>,
    // Pre-joined display fields
    pub feature_display_name: String,
    pub feature_acronym: String,
    pub discipline_display_name: String,
    pub discipline_acronym: String,
    pub discipline_icon: String,
    pub discipline_color: String,
}

/// Stats for a group of tasks (feature or discipline)
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupStats {
    pub name: String,
    pub display_name: String,
    pub total: u32,
    pub done: u32,
    pub pending: u32,
    pub in_progress: u32,
    pub blocked: u32,
    pub skipped: u32,
}

/// Overall project progress
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectProgress {
    pub total_tasks: u32,
    pub done_tasks: u32,
    pub progress_percent: u32,
}
