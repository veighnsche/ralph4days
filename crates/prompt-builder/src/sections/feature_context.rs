use crate::context::PromptContext;
use crate::recipe::Section;

fn build(ctx: &PromptContext) -> Option<String> {
    let feature = ctx.target_task_feature()?;
    let desc = feature
        .description
        .as_deref()
        .unwrap_or("No description provided.");

    let mut out = format!("## Feature: {}\n\n{desc}", feature.display_name);

    // If RAG provided relevant comments, use those (sorted by relevance)
    if let Some(relevant) = &ctx.relevant_comments {
        if !relevant.is_empty() {
            out.push_str("\n\n### Feature Knowledge (most relevant)\n");

            let mut by_category: std::collections::BTreeMap<
                &str,
                Vec<&crate::context::ScoredFeatureComment>,
            > = std::collections::BTreeMap::new();
            for c in relevant {
                by_category.entry(&c.category).or_default().push(c);
            }

            for (category, comments) in &by_category {
                out.push_str(&format!("\n**{category}:**\n"));
                for c in comments {
                    out.push_str(&format!("- {}", c.body));
                    if let Some(reason) = &c.reason {
                        out.push_str(&format!(" (why: {reason})"));
                    }
                    out.push('\n');
                }
            }
        }
    } else if !feature.comments.is_empty() {
        // Fallback: inject all comments when RAG is unavailable
        out.push_str("\n\n### Feature Knowledge\n");

        let mut by_category: std::collections::BTreeMap<&str, Vec<&sqlite_db::FeatureComment>> =
            std::collections::BTreeMap::new();
        for c in &feature.comments {
            by_category.entry(&c.category).or_default().push(c);
        }

        for (category, comments) in &by_category {
            out.push_str(&format!("\n**{category}:**\n"));
            for c in comments {
                out.push_str(&format!("- {}", c.body));
                if let Some(reason) = &c.reason {
                    out.push_str(&format!(" (why: {reason})"));
                }
                out.push('\n');
            }
        }
    }

    Some(out)
}

pub fn feature_context() -> Section {
    Section {
        name: "feature_context",
        build,
    }
}
