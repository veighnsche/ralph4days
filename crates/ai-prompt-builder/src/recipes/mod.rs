mod braindump;
mod discuss;
mod enrichment;
mod opus_review;
mod ramble;
mod task_exec;
mod yap;

use crate::recipe::Recipe;
use crate::PromptType;

/// Get the recipe for a prompt type.
pub fn get(prompt_type: PromptType) -> Recipe {
    match prompt_type {
        PromptType::Braindump => braindump::recipe(),
        PromptType::Yap => yap::recipe(),
        PromptType::Ramble => ramble::recipe(),
        PromptType::Discuss => discuss::recipe(),
        PromptType::TaskExecution => task_exec::recipe(),
        PromptType::OpusReview => opus_review::recipe(),
        PromptType::Enrichment => enrichment::recipe(),
    }
}
