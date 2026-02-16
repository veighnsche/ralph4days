use crate::context::PromptContext;
use crate::recipe::Section;

fn build(ctx: &PromptContext) -> Option<String> {
    let task = ctx.target_task()?;

    let mut out = format!("## Your Task\n\n**{}**", task.title);

    if let Some(desc) = &task.description {
        out.push_str(&format!("\n\n{desc}"));
    }

    if !task.acceptance_criteria.is_empty() {
        out.push_str("\n\n### Acceptance Criteria\n\n");
        for ac in &task.acceptance_criteria {
            out.push_str(&format!("- [ ] {ac}\n"));
        }
    }

    if let Some(pseudocode) = &task.pseudocode {
        out.push_str(&format!("\n### Pseudocode\n\n{pseudocode}"));
    }

    if let Some(hints) = &task.hints {
        out.push_str(&format!("\n### Hints\n\n{hints}"));
    }

    Some(out)
}

pub fn task_details() -> Section {
    Section {
        name: "task_details",
        build,
    }
}
