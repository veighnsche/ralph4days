use crate::context::PromptContext;
use crate::recipe::Section;

fn build(ctx: &PromptContext) -> Option<String> {
    if ctx.disciplines.is_empty() {
        return None;
    }

    let mut out = String::from(
        "## Available Disciplines\n\n\
         | Discipline | Icon |\n\
         |---|---|",
    );

    for d in &ctx.disciplines {
        out.push_str(&format!("\n| {} | {} |", d.display_name, d.icon));
    }

    Some(out)
}

pub fn discipline_listing() -> Section {
    Section {
        name: "discipline_listing",
        build,
    }
}
