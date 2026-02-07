use crate::context::PromptContext;
use crate::recipe::Section;

fn build(ctx: &PromptContext) -> Option<String> {
    if ctx.features.is_empty() {
        return None;
    }

    let mut out = String::from(
        "## Existing Features\n\n\
         | Feature | Description | Tasks |\n\
         |---|---|---|",
    );

    for feature in &ctx.features {
        let desc = feature
            .description
            .as_deref()
            .unwrap_or("-");
        let task_count = ctx
            .tasks
            .iter()
            .filter(|t| t.feature == feature.name)
            .count();
        out.push_str(&format!("\n| {} | {} | {} |", feature.display_name, desc, task_count));
    }

    Some(out)
}

pub fn feature_listing() -> Section {
    Section {
        name: "feature_listing",
        build,
    }
}
