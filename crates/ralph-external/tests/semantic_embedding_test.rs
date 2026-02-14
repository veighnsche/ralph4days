//! End-to-end semantic embedding test.
//!
//! Requires Ollama running locally with nomic-embed-text.
//! Run: cargo test -p ralph-external --test semantic_embedding_test -- --ignored --nocapture

use ralph_external::comment_embeddings::{embed_query, embed_text, CommentEmbeddingConfig};
use ralph_external::OllamaConfig;
use sqlite_db::{AddSubsystemCommentInput, SqliteDb, SubsystemInput};

fn default_ollama() -> OllamaConfig {
    OllamaConfig {
        api_url: "http://localhost:11434".to_owned(),
        embedding_model: "nomic-embed-text".to_owned(),
        embedding_dims: 768,
        llm_model: "unused".to_owned(),
        llm_temperature: 0.0,
    }
}

fn embed_config(ollama: &OllamaConfig) -> CommentEmbeddingConfig<'_> {
    CommentEmbeddingConfig {
        ollama,
        document_prefix: "search_document: ",
        query_prefix: "search_query: ",
        min_search_score: 0.3,
        max_search_results: 10,
    }
}

fn seed_feature_with_comments(db: &SqliteDb) {
    db.create_subsystem(SubsystemInput {
        name: "auth".to_owned(),
        display_name: "Authentication".to_owned(),
        acronym: "AUTH".to_owned(),
        description: Some("User authentication and session management".to_owned()),
    })
    .unwrap();

    let comments = [
        (
            "design-decision",
            "Use JWT tokens with short expiry and refresh token rotation",
            Some("Stateless auth scales better than server-side sessions"),
        ),
        (
            "gotcha",
            "bcrypt has a 72-byte password limit — truncates silently",
            None,
        ),
        (
            "convention",
            "All auth endpoints return 401 for invalid credentials, never 403",
            None,
        ),
        (
            "architecture",
            "Auth middleware validates JWT on every request, no caching of decoded tokens",
            Some("Prevents stale permission checks after role changes"),
        ),
        (
            "gotcha",
            "OAuth2 state parameter must be cryptographically random to prevent CSRF",
            None,
        ),
        (
            "design-decision",
            "Rate limit login attempts to 5 per minute per IP using sliding window",
            Some("Brute force protection without locking out legitimate users"),
        ),
    ];

    for (category, body, reason) in comments {
        db.add_subsystem_comment(AddSubsystemCommentInput {
            subsystem_name: "auth".to_owned(),
            category: category.to_owned(),
            discipline: None,
            agent_task_id: None,
            body: body.to_owned(),
            summary: None,
            reason: reason.map(str::to_owned),
            source_iteration: None,
        })
        .unwrap();
    }

    db.create_subsystem(SubsystemInput {
        name: "billing".to_owned(),
        display_name: "Billing".to_owned(),
        acronym: "BILL".to_owned(),
        description: Some("Payment processing and subscription management".to_owned()),
    })
    .unwrap();

    let billing_comments = [
        (
            "design-decision",
            "Use Stripe webhooks for payment confirmation, never trust client-side success",
        ),
        (
            "gotcha",
            "Stripe webhook signatures expire after 5 minutes — clock skew breaks verification",
        ),
        (
            "convention",
            "All monetary amounts stored as integers in cents, never floats",
        ),
    ];

    for (category, body) in billing_comments {
        db.add_subsystem_comment(AddSubsystemCommentInput {
            subsystem_name: "billing".to_owned(),
            category: category.to_owned(),
            discipline: None,
            agent_task_id: None,
            body: body.to_owned(),
            summary: None,
            reason: None,
            source_iteration: None,
        })
        .unwrap();
    }
}

async fn embed_all_comments(db: &SqliteDb, config: &CommentEmbeddingConfig<'_>) {
    for feature in db.get_subsystems() {
        for comment in &feature.comments {
            let text = ralph_external::comment_embeddings::build_embedding_text(
                &comment.category,
                &comment.body,
                comment.reason.as_deref(),
            );
            let result = embed_text(config, &text).await.unwrap();
            db.upsert_comment_embedding(comment.id, &result.vector, &result.model, &result.hash)
                .unwrap();
        }
    }
}

#[tokio::test]
#[ignore = "Requires local Ollama running with nomic-embed-text"]
async fn semantic_search_surfaces_relevant_auth_comments() {
    let db = SqliteDb::open_in_memory(None).unwrap();
    let ollama = default_ollama();
    let config = embed_config(&ollama);

    seed_feature_with_comments(&db);
    embed_all_comments(&db, &config).await;

    // Query about password security — should surface bcrypt gotcha and JWT decision
    let query_vec = embed_query(
        &config,
        "How should we handle password hashing and storage?",
    )
    .await
    .unwrap();

    let results = db.search_subsystem_comments("auth", &query_vec, 3, 0.3);

    println!("\n=== Query: password hashing ===");
    for r in &results {
        println!("  [{:.3}] [{}] {}", r.score, r.category, r.body);
    }

    assert!(!results.is_empty(), "Search returned no results");
    // bcrypt gotcha should be highly relevant to password hashing
    assert!(
        results.iter().any(|r| r.body.contains("bcrypt")),
        "Expected bcrypt gotcha to surface for password query. Got: {:?}",
        results.iter().map(|r| &r.body).collect::<Vec<_>>()
    );
}

#[tokio::test]
#[ignore = "Requires local Ollama running with nomic-embed-text"]
async fn semantic_search_isolates_features() {
    let db = SqliteDb::open_in_memory(None).unwrap();
    let ollama = default_ollama();
    let config = embed_config(&ollama);

    seed_feature_with_comments(&db);
    embed_all_comments(&db, &config).await;

    // Query about payments — should only search billing feature
    let query_vec = embed_query(&config, "How do we process credit card payments?")
        .await
        .unwrap();

    let auth_results = db.search_subsystem_comments("auth", &query_vec, 10, 0.3);
    let billing_results = db.search_subsystem_comments("billing", &query_vec, 10, 0.3);

    println!("\n=== Query: credit card payments ===");
    println!("  Auth results: {}", auth_results.len());
    for r in &auth_results {
        println!("    [{:.3}] {}", r.score, r.body);
    }
    println!("  Billing results: {}", billing_results.len());
    for r in &billing_results {
        println!("    [{:.3}] {}", r.score, r.body);
    }

    assert!(
        !billing_results.is_empty(),
        "Expected billing results for payment query"
    );
    assert!(
        billing_results.iter().any(|r| r.body.contains("Stripe")),
        "Expected Stripe comment for payment query"
    );
}

#[tokio::test]
#[ignore = "Requires local Ollama running with nomic-embed-text"]
async fn semantic_search_feeds_into_prompt_builder() {
    let db = SqliteDb::open_in_memory(None).unwrap();
    let ollama = default_ollama();
    let config = embed_config(&ollama);

    seed_feature_with_comments(&db);
    embed_all_comments(&db, &config).await;

    let query_vec = embed_query(&config, "Implement the login endpoint with rate limiting")
        .await
        .unwrap();

    let results = db.search_subsystem_comments("auth", &query_vec, 5, 0.3);

    println!("\n=== Query: login endpoint with rate limiting ===");
    for r in &results {
        println!("  [{:.3}] [{}] {}", r.score, r.category, r.body);
    }

    assert!(
        results.len() >= 2,
        "Expected at least 2 relevant results, got {}",
        results.len()
    );

    // Convert to prompt-builder types and render
    let scored: Vec<prompt_builder::context::ScoredFeatureComment> = results
        .iter()
        .map(|r| prompt_builder::context::ScoredFeatureComment {
            category: r.category.clone(),
            body: r.body.clone(),
            summary: r.summary.clone(),
            reason: r.reason.clone(),
            score: r.score,
        })
        .collect();

    let feature = db
        .get_subsystems()
        .into_iter()
        .find(|f| f.name == "auth")
        .unwrap();

    let mut ctx = prompt_builder::context::PromptContext {
        features: vec![feature],
        tasks: vec![],
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
        script_dir: "/tmp/ralph-mcp".to_owned(),
        api_server_port: None,
        user_input: None,
        target_task_id: None,
        target_feature: None,
        codebase_snapshot: None,
        instruction_overrides: std::collections::HashMap::new(),
        relevant_comments: None,
    };
    ctx.tasks = vec![sqlite_db::Task {
        id: 1,
        subsystem: "auth".to_owned(),
        discipline: "backend".to_owned(),
        title: "Implement login endpoint".to_owned(),
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
        subsystem_display_name: "Authentication".to_owned(),
        subsystem_acronym: "AUTH".to_owned(),
        discipline_display_name: "Backend".to_owned(),
        discipline_acronym: "BACK".to_owned(),
        discipline_icon: "Server".to_owned(),
        discipline_color: "#8b5cf6".to_owned(),
    }];
    ctx.target_task_id = Some(1);
    ctx.relevant_comments = Some(scored);

    let section = prompt_builder::sections::feature_context::feature_context();
    let output = (section.build)(&ctx).expect("Section should produce output");

    println!("\n=== Rendered prompt section ===\n{output}");

    assert!(output.contains("Authentication"), "Missing feature name");
    assert!(output.contains("most relevant"), "Missing RAG header");
    // Rate limiting comment should appear since we queried about it
    assert!(
        output.contains("rate limit") || output.contains("Rate limit"),
        "Expected rate limiting knowledge in prompt. Full output:\n{output}"
    );
}
