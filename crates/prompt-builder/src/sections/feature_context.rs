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
                    let display = c.summary.as_deref().unwrap_or(&c.body);
                    out.push_str(&format!("- {display}"));
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
                let display = c.summary.as_deref().unwrap_or(&c.body);
                out.push_str(&format!("- {display}"));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{test_context, ScoredFeatureComment};
    use sqlite_db::{Feature, FeatureComment, FeatureStatus, Task, TaskStatus};

    fn test_feature(name: &str, display: &str, description: Option<&str>) -> Feature {
        Feature {
            name: name.to_owned(),
            display_name: display.to_owned(),
            acronym: "TEST".to_owned(),
            description: description.map(ToOwned::to_owned),
            created: None,
            status: FeatureStatus::Active,
            comments: vec![],
        }
    }

    fn test_task(id: u32, feature: &str) -> Task {
        Task {
            id,
            feature: feature.to_owned(),
            discipline: "backend".to_owned(),
            title: "Test task".to_owned(),
            description: None,
            status: TaskStatus::Pending,
            priority: None,
            tags: vec![],
            depends_on: vec![],
            created: None,
            updated: None,
            completed: None,
            acceptance_criteria: vec![],
            context_files: vec![],
            output_artifacts: vec![],
            hints: None,
            estimated_turns: None,
            provenance: None,
            pseudocode: None,
            enriched_at: None,
            signals: vec![],
            feature_display_name: "Test".to_owned(),
            feature_acronym: "TEST".to_owned(),
            discipline_display_name: "Backend".to_owned(),
            discipline_acronym: "BACK".to_owned(),
            discipline_icon: "Server".to_owned(),
            discipline_color: "#8b5cf6".to_owned(),
        }
    }

    fn test_comment(id: u32, category: &str, body: &str, reason: Option<&str>) -> FeatureComment {
        FeatureComment {
            id,
            category: category.to_owned(),
            discipline: None,
            agent_task_id: None,
            body: body.to_owned(),
            summary: None,
            reason: reason.map(ToOwned::to_owned),
            source_iteration: None,
            created: None,
            updated: None,
        }
    }

    #[test]
    fn no_feature_returns_none() {
        let ctx = test_context();
        assert!(build(&ctx).is_none());
    }

    #[test]
    fn feature_description_only() {
        let mut ctx = test_context();
        ctx.features = vec![test_feature(
            "auth",
            "Authentication",
            Some("Handles login"),
        )];
        ctx.tasks = vec![test_task(1, "auth")];
        ctx.target_task_id = Some(1);

        let output = build(&ctx).unwrap();
        assert!(output.contains("Authentication"));
        assert!(output.contains("Handles login"));
        assert!(!output.contains("Knowledge"));
    }

    #[test]
    fn fallback_injects_all_comments() {
        let mut ctx = test_context();
        let mut feat = test_feature("auth", "Auth", Some("Login system"));
        feat.comments = vec![
            test_comment(1, "architecture", "Use JWT", None),
            test_comment(2, "convention", "snake_case everywhere", None),
            test_comment(3, "architecture", "Layered design", None),
        ];
        ctx.features = vec![feat];
        ctx.tasks = vec![test_task(1, "auth")];
        ctx.target_task_id = Some(1);
        ctx.relevant_comments = None;

        let output = build(&ctx).unwrap();
        assert!(output.contains("### Feature Knowledge\n"));
        assert!(output.contains("**architecture:**"));
        assert!(output.contains("**convention:**"));
        assert!(output.contains("Use JWT"));
        assert!(output.contains("Layered design"));
        assert!(output.contains("snake_case everywhere"));
        // BTreeMap orders alphabetically: architecture before convention
        let arch_pos = output.find("**architecture:**").unwrap();
        let conv_pos = output.find("**convention:**").unwrap();
        assert!(arch_pos < conv_pos);
    }

    #[test]
    fn rag_injects_relevant_comments() {
        let mut ctx = test_context();
        ctx.features = vec![test_feature("auth", "Auth", Some("Login"))];
        ctx.tasks = vec![test_task(1, "auth")];
        ctx.target_task_id = Some(1);
        ctx.relevant_comments = Some(vec![
            ScoredFeatureComment {
                category: "gotcha".to_owned(),
                body: "Watch for XSS".to_owned(),
                summary: None,
                reason: None,
                score: 0.95,
            },
            ScoredFeatureComment {
                category: "gotcha".to_owned(),
                body: "Sanitize inputs".to_owned(),
                summary: None,
                reason: None,
                score: 0.8,
            },
        ]);

        let output = build(&ctx).unwrap();
        assert!(output.contains("most relevant"));
        assert!(output.contains("Watch for XSS"));
        assert!(output.contains("Sanitize inputs"));
    }

    #[test]
    fn rag_empty_vec_no_knowledge_section() {
        let mut ctx = test_context();
        ctx.features = vec![test_feature("auth", "Auth", Some("Login"))];
        ctx.tasks = vec![test_task(1, "auth")];
        ctx.target_task_id = Some(1);
        ctx.relevant_comments = Some(vec![]);

        let output = build(&ctx).unwrap();
        assert!(!output.contains("Knowledge"));
    }

    #[test]
    fn comment_with_reason_annotated() {
        let mut ctx = test_context();
        let mut feat = test_feature("auth", "Auth", Some("Login"));
        feat.comments = vec![test_comment(
            1,
            "gotcha",
            "Use bcrypt",
            Some("Industry standard"),
        )];
        ctx.features = vec![feat];
        ctx.tasks = vec![test_task(1, "auth")];
        ctx.target_task_id = Some(1);

        let output = build(&ctx).unwrap();
        assert!(output.contains("(why: Industry standard)"));
    }

    #[test]
    fn summary_preferred_over_body_in_fallback() {
        let mut ctx = test_context();
        let mut feat = test_feature("auth", "Auth", Some("Login"));
        let mut c = test_comment(1, "gotcha", "Full detailed reasoning about bcrypt", None);
        c.summary = Some("Use bcrypt".to_owned());
        feat.comments = vec![c];
        ctx.features = vec![feat];
        ctx.tasks = vec![test_task(1, "auth")];
        ctx.target_task_id = Some(1);

        let output = build(&ctx).unwrap();
        assert!(output.contains("- Use bcrypt\n"));
        assert!(!output.contains("Full detailed reasoning"));
    }

    #[test]
    fn summary_preferred_over_body_in_rag() {
        let mut ctx = test_context();
        ctx.features = vec![test_feature("auth", "Auth", Some("Login"))];
        ctx.tasks = vec![test_task(1, "auth")];
        ctx.target_task_id = Some(1);
        ctx.relevant_comments = Some(vec![ScoredFeatureComment {
            category: "gotcha".to_owned(),
            body: "Full detailed reasoning about bcrypt".to_owned(),
            summary: Some("Use bcrypt".to_owned()),
            reason: None,
            score: 0.9,
        }]);

        let output = build(&ctx).unwrap();
        assert!(output.contains("- Use bcrypt\n"));
        assert!(!output.contains("Full detailed reasoning"));
    }

    #[test]
    fn comment_without_reason_no_annotation() {
        let mut ctx = test_context();
        let mut feat = test_feature("auth", "Auth", Some("Login"));
        feat.comments = vec![test_comment(1, "gotcha", "Use bcrypt", None)];
        ctx.features = vec![feat];
        ctx.tasks = vec![test_task(1, "auth")];
        ctx.target_task_id = Some(1);

        let output = build(&ctx).unwrap();
        assert!(output.contains("Use bcrypt"));
        assert!(!output.contains("(why:"));
    }
}
