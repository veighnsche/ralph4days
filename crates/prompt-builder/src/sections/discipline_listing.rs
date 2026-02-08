use crate::context::PromptContext;
use crate::recipe::Section;

fn build(ctx: &PromptContext) -> Option<String> {
    if ctx.disciplines.is_empty() {
        return None;
    }

    let mut out = String::from(
        "## Available Disciplines\n\n\
         | Discipline | Acronym | Skills |\n\
         |---|---|---|",
    );

    for d in &ctx.disciplines {
        let skills = if d.skills.is_empty() {
            "—".to_owned()
        } else if d.skills.len() <= 3 {
            d.skills.join(", ")
        } else {
            format!(
                "{} (+{} more)",
                d.skills[..2].join(", "),
                d.skills.len() - 2
            )
        };

        out.push_str(&format!(
            "\n| {} {} | {} | {} |",
            d.icon, d.display_name, d.acronym, skills
        ));
    }

    Some(out)
}

pub fn discipline_listing() -> Section {
    Section {
        name: "discipline_listing",
        build,
    }
}
