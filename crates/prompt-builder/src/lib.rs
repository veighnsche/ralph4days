pub mod context;
pub mod error;
pub mod mcp;
pub mod output;
pub mod recipe;
pub mod recipes;
pub mod sections;
pub mod stagnation;

// Re-exports for convenience
pub use context::PromptContext;
pub use error::PromptError;
pub use output::{McpScript, PromptOutput};
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

/// Build a prompt from a recipe. Pure function — no I/O.
pub fn build(prompt_type: PromptType, ctx: &PromptContext) -> Result<PromptOutput, PromptError> {
    let recipe = recipes::get(prompt_type);
    recipe::execute_recipe(&recipe, ctx)
}
