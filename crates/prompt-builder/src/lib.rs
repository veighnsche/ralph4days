pub mod context;
pub mod error;
pub mod mcp;
pub mod output;
pub mod recipe;
pub mod recipes;
pub mod sections;
pub mod snapshot;
pub mod stagnation;

// Re-exports for convenience
pub use context::PromptContext;
pub use error::PromptError;
pub use output::{McpScript, PromptOutput};
pub use recipe::PromptSection;
pub use sections::metadata::SectionInfo;
pub use snapshot::CodebaseSnapshot;
pub use stagnation::{check_completion, hash_content};

/// The six prompt surfaces Ralph supports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptType {
    Braindump,
    Yap,
    Ramble,
    Discuss,
    TaskExecution,
    OpusReview,
}

impl PromptType {
    /// Parse from a lowercase string (e.g. "braindump", "task_execution").
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "braindump" => Some(Self::Braindump),
            "yap" => Some(Self::Yap),
            "ramble" => Some(Self::Ramble),
            "discuss" => Some(Self::Discuss),
            "task_execution" => Some(Self::TaskExecution),
            "opus_review" => Some(Self::OpusReview),
            _ => None,
        }
    }
}

/// Build a prompt from a recipe. Pure function — no I/O.
pub fn build(prompt_type: PromptType, ctx: &PromptContext) -> Result<PromptOutput, PromptError> {
    let recipe = recipes::get(prompt_type);
    recipe::execute_recipe(&recipe, ctx)
}

/// Build each section individually for preview purposes. Pure function — no I/O.
pub fn build_sections(prompt_type: PromptType, ctx: &PromptContext) -> Vec<PromptSection> {
    let r = recipes::get(prompt_type);
    recipe::build_sections(&r, ctx)
}

/// Build sections from an arbitrary list of section names. Pure function — no I/O.
pub fn build_custom_sections(section_names: &[&str], ctx: &PromptContext) -> Vec<PromptSection> {
    recipe::build_sections_from_names(section_names, ctx)
}

/// Get the section names for a built-in recipe.
pub fn get_recipe_section_names(prompt_type: PromptType) -> Vec<&'static str> {
    let r = recipes::get(prompt_type);
    r.sections.iter().map(|s| s.name).collect()
}

/// Get the default instruction text for a prompt type (before any overrides).
pub fn default_instructions(prompt_type: PromptType) -> String {
    match prompt_type {
        PromptType::Braindump => sections::instructions::braindump::default_text(),
        PromptType::Yap => sections::instructions::yap::default_text(),
        PromptType::Ramble => sections::instructions::ramble::default_text(),
        PromptType::Discuss => sections::instructions::discuss::default_text(),
        PromptType::TaskExecution => sections::instructions::task_exec::default_text(),
        PromptType::OpusReview => sections::instructions::opus_review::default_text(),
    }
}
