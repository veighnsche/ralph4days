use crate::context::PromptContext;
use crate::recipe::Section;

fn build(ctx: &PromptContext) -> Option<String> {
    let content = ctx.claude_ralph_md.as_ref()?;
    Some(format!("## Project Context\n\n{content}"))
}

pub fn project_context() -> Section {
    Section {
        name: "project_context",
        build,
    }
}
