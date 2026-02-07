use crate::context::PromptContext;
use crate::recipe::Section;

fn build(ctx: &PromptContext) -> Option<String> {
    let title = &ctx.metadata.title;
    let desc = ctx
        .metadata
        .description
        .as_deref()
        .unwrap_or("No description provided.");
    Some(format!("## Project: {title}\n\n{desc}"))
}

pub fn project_metadata() -> Section {
    Section {
        name: "project_metadata",
        build,
    }
}
