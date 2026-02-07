use crate::context::PromptContext;
use crate::recipe::Section;

fn build(ctx: &PromptContext) -> Option<String> {
    let feature = ctx.target_task_feature()?;
    let desc = feature
        .description
        .as_deref()
        .unwrap_or("No description provided.");
    Some(format!("## Feature: {}\n\n{desc}", feature.display_name))
}

pub fn feature_context() -> Section {
    Section {
        name: "feature_context",
        build,
    }
}
