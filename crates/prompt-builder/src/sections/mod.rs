pub mod codebase_state;
pub mod dependency_context;
pub mod discipline_listing;
pub mod discipline_persona;
pub mod feature_context;
pub mod feature_files;
pub mod feature_listing;
pub mod feature_state;
pub mod instructions;
pub mod metadata;
pub mod previous_attempts;
pub mod project_context;
pub mod project_metadata;
pub mod state_files;
pub mod task_details;
pub mod task_files;
pub mod task_listing;
pub mod user_input;

// Re-export constructors so recipes can call `sections::project_context()` etc.
pub use codebase_state::codebase_state;
pub use dependency_context::dependency_context;
pub use discipline_listing::discipline_listing;
pub use discipline_persona::discipline_persona;
pub use feature_context::feature_context;
pub use feature_files::feature_files;
pub use feature_listing::feature_listing;
pub use feature_state::feature_state;
pub use instructions::braindump::braindump_instructions;
pub use instructions::discuss::discuss_instructions;
pub use instructions::opus_review::opus_review_instructions;
pub use instructions::ramble::ramble_instructions;
pub use instructions::task_exec::task_exec_instructions;
pub use instructions::yap::yap_instructions;
pub use previous_attempts::previous_attempts;
pub use project_context::project_context;
pub use project_metadata::project_metadata;
pub use state_files::state_files;
pub use task_details::task_details;
pub use task_files::task_files;
pub use task_listing::task_listing;
pub use user_input::user_input;

use crate::recipe::Section;

/// Look up a section constructor by name.
pub fn get_section(name: &str) -> Option<Section> {
    match name {
        "codebase_state" => Some(codebase_state::codebase_state()),
        "project_context" => Some(project_context::project_context()),
        "project_metadata" => Some(project_metadata::project_metadata()),
        "discipline_persona" => Some(discipline_persona::discipline_persona()),
        "feature_context" => Some(feature_context::feature_context()),
        "feature_files" => Some(feature_files::feature_files()),
        "feature_state" => Some(feature_state::feature_state()),
        "feature_listing" => Some(feature_listing::feature_listing()),
        "task_details" => Some(task_details::task_details()),
        "task_files" => Some(task_files::task_files()),
        "task_listing" => Some(task_listing::task_listing()),
        "dependency_context" => Some(dependency_context::dependency_context()),
        "previous_attempts" => Some(previous_attempts::previous_attempts()),
        "discipline_listing" => Some(discipline_listing::discipline_listing()),
        "state_files" => Some(state_files::state_files()),
        "user_input" => Some(user_input::user_input()),
        "braindump_instructions" => Some(instructions::braindump::braindump_instructions()),
        "yap_instructions" => Some(instructions::yap::yap_instructions()),
        "ramble_instructions" => Some(instructions::ramble::ramble_instructions()),
        "discuss_instructions" => Some(instructions::discuss::discuss_instructions()),
        "task_exec_instructions" => Some(instructions::task_exec::task_exec_instructions()),
        "opus_review_instructions" => Some(instructions::opus_review::opus_review_instructions()),
        _ => None,
    }
}
