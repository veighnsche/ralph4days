use crate::context::PromptContext;
use crate::recipe::Section;

fn build(ctx: &PromptContext) -> Option<String> {
    if ctx.tasks.is_empty() {
        return None;
    }

    let mut out = String::from(
        "## Existing Tasks\n\n\
         | ID | Feature | Title | Status | Priority |\n\
         |---|---|---|---|---|",
    );

    for task in &ctx.tasks {
        let priority = task
            .priority
            .as_ref()
            .map_or("-", sqlite_db::Priority::as_str);
        out.push_str(&format!(
            "\n| {} | {} | {} | {} | {} |",
            task.id,
            task.feature,
            task.title,
            task.status.as_str(),
            priority,
        ));
    }

    Some(out)
}

pub fn task_listing() -> Section {
    Section {
        name: "task_listing",
        build,
    }
}
