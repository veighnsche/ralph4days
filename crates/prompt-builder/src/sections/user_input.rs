use crate::context::PromptContext;
use crate::recipe::Section;

fn build(ctx: &PromptContext) -> Option<String> {
    let text = ctx.user_input.as_ref()?;
    Some(format!("## User's Input\n\n{text}"))
}

pub fn user_input() -> Section {
    Section {
        name: "user_input",
        build,
    }
}
