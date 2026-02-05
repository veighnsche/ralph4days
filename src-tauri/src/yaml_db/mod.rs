// yaml_db module - Multi-file YAML database for PRD management
//
// Replaces single prd.yaml with 4 separate files:
// - tasks.yaml: Task records
// - features.yaml: Feature definitions
// - disciplines.yaml: Discipline definitions
// - metadata.yaml: Project metadata and counters

mod database;
mod disciplines;
mod features;
mod metadata;
mod tasks;

// Re-export public types
pub use database::{YamlDatabase, TaskInput};
pub use disciplines::{Discipline, DisciplinesFile};
pub use features::{Feature, FeaturesFile};
pub use metadata::{MetadataFile, ProjectMetadata};
pub use tasks::TasksFile;
