use crate::context::PromptContext;
use crate::recipe::Section;

fn build(ctx: &PromptContext) -> Option<String> {
    let discipline = ctx.target_task_discipline()?;
    let system_prompt = discipline.system_prompt.as_ref()?;

    let mut out = format!("## You Are\n\n{system_prompt}");

    if !discipline.skills.is_empty() {
        out.push_str("\n\n### Skills\n\n");
        for skill in &discipline.skills {
            out.push_str(&format!("- {skill}\n"));
        }
    }

    if let Some(conventions) = &discipline.conventions {
        out.push_str(&format!("\n### Conventions\n\n{conventions}"));
    }

    Some(out)
}

pub fn discipline_persona() -> Section {
    Section {
        name: "discipline_persona",
        build,
    }
}
