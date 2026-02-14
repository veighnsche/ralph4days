//! Smoke test: proves the full embedding pipeline works end-to-end.
//!
//! Requires: ollama serve + qwen3-embedding:0.6b
//! Run: cargo test -p ralph-external --test pipeline_smoke_test -- --ignored --nocapture

use ralph_external::comment_embeddings::{
    build_embedding_text, embed_query, embed_text, CommentEmbeddingConfig,
};
use ralph_external::OllamaConfig;
use sqlite_db::{AddSubsystemCommentInput, SqliteDb, SubsystemInput};

fn ollama() -> OllamaConfig {
    OllamaConfig {
        api_url: "http://localhost:11434".to_owned(),
        embedding_model: "qwen3-embedding:0.6b".to_owned(),
        embedding_dims: 1024,
        llm_model: "unused".to_owned(),
        llm_temperature: 0.0,
    }
}

fn config(ollama: &OllamaConfig) -> CommentEmbeddingConfig<'_> {
    CommentEmbeddingConfig {
        ollama,
        document_prefix: "search_document: ",
        query_prefix: "search_query: ",
        min_search_score: 0.3,
        max_search_results: 10,
    }
}

#[tokio::test]
#[ignore = "Requires local Ollama (ollama serve) + qwen3-embedding:0.6b"]
async fn smoke_embed_store_search_render() {
    let db = SqliteDb::open_in_memory(None).unwrap();
    let ollama = ollama();
    let cfg = config(&ollama);

    // 1. Create two features
    db.create_subsystem(SubsystemInput {
        name: "auth".to_owned(),
        display_name: "Auth".to_owned(),
        acronym: "AUTH".to_owned(),
        description: Some("User authentication".to_owned()),
    })
    .unwrap();
    db.create_subsystem(SubsystemInput {
        name: "billing".to_owned(),
        display_name: "Billing".to_owned(),
        acronym: "BILL".to_owned(),
        description: Some("Payment processing".to_owned()),
    })
    .unwrap();

    // 2. Agents dump knowledge
    let comments = [
        (
            "auth",
            "gotcha",
            "bcrypt silently truncates passwords longer than 72 bytes",
        ),
        (
            "auth",
            "design-decision",
            "Use short-lived JWT access tokens with refresh token rotation",
        ),
        (
            "auth",
            "convention",
            "Return 401 for bad credentials, never 403",
        ),
        (
            "billing",
            "gotcha",
            "Stripe webhook signatures expire after 5 minutes",
        ),
        (
            "billing",
            "convention",
            "Store all monetary amounts as integer cents, never floats",
        ),
    ];
    for (feat, cat, body) in comments {
        db.add_subsystem_comment(AddSubsystemCommentInput {
            subsystem_name: feat.to_owned(),
            category: cat.to_owned(),
            discipline: None,
            agent_task_id: None,
            body: body.to_owned(),
            summary: None,
            reason: None,
            source_iteration: None,
        })
        .unwrap();
    }

    // 3. Embed every comment (this is the real Ollama call)
    let all_features = db.get_subsystems();
    for f in &all_features {
        for c in &f.comments {
            let text = build_embedding_text(&c.category, &c.body, c.reason.as_deref());
            let result = embed_text(&cfg, &text).await.unwrap();
            assert_eq!(result.vector.len(), 1024, "Wrong embedding dimensions");
            db.upsert_comment_embedding(c.id, &result.vector, &result.model, &result.hash)
                .unwrap();
        }
    }
    println!(
        "Embedded {} comments",
        all_features.iter().map(|f| f.comments.len()).sum::<usize>()
    );

    // 4. Search: "password hashing" should find bcrypt, not Stripe
    let query_vec = embed_query(&cfg, "How should we hash passwords?")
        .await
        .unwrap();
    let auth_hits = db.search_subsystem_comments("auth", &query_vec, 3, 0.3);
    let billing_hits = db.search_subsystem_comments("billing", &query_vec, 3, 0.3);

    println!("\nQuery: 'How should we hash passwords?'");
    println!("  Auth hits:");
    for r in &auth_hits {
        println!("    [{:.3}] [{}] {}", r.score, r.category, r.body);
    }
    println!("  Billing hits:");
    for r in &billing_hits {
        println!("    [{:.3}] [{}] {}", r.score, r.category, r.body);
    }

    assert!(!auth_hits.is_empty(), "Expected auth results");
    assert!(
        auth_hits[0].body.contains("bcrypt"),
        "Top auth hit should be bcrypt, got: {}",
        auth_hits[0].body
    );

    // 5. Feed into prompt-builder — does the rendered section contain the knowledge?
    let scored: Vec<prompt_builder::context::ScoredFeatureComment> = auth_hits
        .iter()
        .map(|r| prompt_builder::context::ScoredFeatureComment {
            category: r.category.clone(),
            body: r.body.clone(),
            summary: r.summary.clone(),
            reason: r.reason.clone(),
            score: r.score,
        })
        .collect();

    let section = prompt_builder::sections::feature_context::feature_context();
    let ctx = prompt_builder::context::PromptContext {
        features: vec![all_features.into_iter().find(|f| f.name == "auth").unwrap()],
        tasks: vec![sqlite_db::Task {
            id: 1,
            subsystem: "auth".to_owned(),
            discipline: "backend".to_owned(),
            title: "Implement password hashing".to_owned(),
            description: None,
            status: sqlite_db::TaskStatus::Pending,
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
            agent: None,
            model: None,
            effort: None,
            thinking: None,
            pseudocode: None,
            enriched_at: None,
            signals: vec![],
            subsystem_display_name: "Auth".to_owned(),
            subsystem_acronym: "AUTH".to_owned(),
            discipline_display_name: "Backend".to_owned(),
            discipline_acronym: "BACK".to_owned(),
            discipline_icon: "Server".to_owned(),
            discipline_color: "#8b5cf6".to_owned(),
        }],
        disciplines: vec![],
        metadata: sqlite_db::ProjectMetadata {
            title: "Test".to_owned(),
            description: None,
            created: None,
        },
        file_contents: std::collections::HashMap::new(),
        progress_txt: None,
        learnings_txt: None,
        claude_ralph_md: None,
        project_path: "/tmp/test".to_owned(),
        db_path: "/tmp/test/.ralph/db/ralph.db".to_owned(),
        script_dir: "/tmp/mcp".to_owned(),
        api_server_port: None,
        user_input: None,
        target_task_id: Some(1),
        target_feature: None,
        codebase_snapshot: None,
        instruction_overrides: std::collections::HashMap::new(),
        relevant_comments: Some(scored),
    };

    let output = (section.build)(&ctx).expect("Section should render");
    println!("\n=== Rendered prompt section ===\n{output}");

    assert!(
        output.contains("bcrypt"),
        "Prompt should contain bcrypt knowledge"
    );
    println!("\n✓ Pipeline works: comment → embed → search → prompt");
}
