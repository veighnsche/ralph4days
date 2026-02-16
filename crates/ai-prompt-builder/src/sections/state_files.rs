use crate::context::PromptContext;
use crate::recipe::Section;

fn build(ctx: &PromptContext) -> Option<String> {
    if ctx.progress_txt.is_none() && ctx.learnings_txt.is_none() {
        return None;
    }

    let mut out = String::new();

    if let Some(progress) = &ctx.progress_txt {
        out.push_str(&format!("## Progress Log\n\n{progress}"));
    }

    if let Some(learnings) = &ctx.learnings_txt {
        if !out.is_empty() {
            out.push_str("\n\n");
        }
        out.push_str(&format!("## Learnings\n\n{learnings}"));
    }

    Some(out)
}

pub fn state_files() -> Section {
    Section {
        name: "state_files",
        build,
    }
}
