use crate::context::PromptContext;
use crate::recipe::Section;
use sqlite_db::TaskStatus;

fn build(ctx: &PromptContext) -> Option<String> {
    let task = ctx.target_task()?;

    if task.depends_on.is_empty() {
        return None;
    }

    let completed_deps: Vec<_> = task
        .depends_on
        .iter()
        .filter_map(|dep_id| ctx.tasks.iter().find(|t| t.id == *dep_id))
        .filter(|t| t.status == TaskStatus::Done)
        .collect();

    if completed_deps.is_empty() {
        return None;
    }

    let mut out = String::from("## Completed Prerequisites\n\n");
    for dep in &completed_deps {
        let desc = dep
            .description
            .as_deref()
            .unwrap_or("No description");
        // Truncate description to first sentence or 120 chars for a summary
        let summary = desc
            .split_once('.')
            .map(|(first, _)| first)
            .unwrap_or(desc);
        let summary = if summary.len() > 120 {
            &summary[..120]
        } else {
            summary
        };
        out.push_str(&format!("- **{}** (#{}): {summary}\n", dep.title, dep.id));
    }

    Some(out.trim_end().to_string())
}

pub fn dependency_context() -> Section {
    Section {
        name: "dependency_context",
        build,
    }
}
