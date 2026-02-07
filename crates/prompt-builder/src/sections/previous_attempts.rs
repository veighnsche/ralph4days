use crate::context::PromptContext;
use crate::recipe::Section;

fn build(ctx: &PromptContext) -> Option<String> {
    let task = ctx.target_task()?;

    if task.comments.is_empty() {
        return None;
    }

    let mut out = String::from("## Previous Attempts\n\n");
    for comment in &task.comments {
        let author = comment.author.as_str();
        out.push_str(&format!(
            "### Attempt (by {author})\n\n{}\n\n",
            comment.body
        ));
    }

    Some(out.trim_end().to_string())
}

pub fn previous_attempts() -> Section {
    Section {
        name: "previous_attempts",
        build,
    }
}
