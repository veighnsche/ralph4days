use thiserror::Error;

#[derive(Debug, Error)]
pub enum PromptError {
    #[error("No recipe found for prompt type")]
    UnknownRecipe,

    #[error("Target task {0} not found in context")]
    TaskNotFound(u32),

    #[error("Target feature '{0}' not found in context")]
    FeatureNotFound(String),

    #[error("Section '{0}' not found in registry")]
    SectionNotFound(String),
}
