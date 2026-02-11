use ralph_external::comment_embeddings::{
    build_embedding_text, embed_query, embed_text, should_embed, CommentEmbeddingConfig,
};
use ralph_external::OllamaConfig;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn test_ollama_config(server: &MockServer) -> OllamaConfig {
    OllamaConfig {
        api_url: server.uri(),
        embedding_model: "nomic-embed-text".to_owned(),
        embedding_dims: 3,
        llm_model: "test".to_owned(),
        llm_temperature: 0.7,
    }
}

fn test_embedding_config(ollama: &OllamaConfig) -> CommentEmbeddingConfig<'_> {
    CommentEmbeddingConfig {
        ollama,
        document_prefix: "search_document: ",
        query_prefix: "search_query: ",
        min_search_score: 0.3,
        max_search_results: 5,
    }
}

async fn mock_ollama_embed(embedding: Vec<f32>) -> MockServer {
    let server = MockServer::start().await;
    let body = serde_json::json!({ "embeddings": [embedding] });
    Mock::given(method("POST"))
        .and(path("/api/embed"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;
    server
}

#[tokio::test]
async fn should_embed_new_comment() {
    let db = sqlite_db::SqliteDb::open_in_memory(None).unwrap();
    db.create_feature(sqlite_db::FeatureInput {
        name: "auth".to_owned(),
        display_name: "Auth".to_owned(),
        acronym: "AUTH".to_owned(),
        ..Default::default()
    })
    .unwrap();
    db.add_feature_comment(sqlite_db::AddFeatureCommentInput {
        feature_name: "auth".to_owned(),
        category: "gotcha".to_owned(),
        discipline: None,
        agent_task_id: None,
        body: "Use JWT".to_owned(),
        summary: None,
        reason: None,
        source_iteration: None,
    })
    .unwrap();
    let comment_id = db.get_features()[0].comments[0].id;

    let result = should_embed(&db, comment_id, "gotcha", "Use JWT", None);
    assert!(result.is_some());
}

#[tokio::test]
async fn should_embed_unchanged_comment() {
    let db = sqlite_db::SqliteDb::open_in_memory(None).unwrap();
    db.create_feature(sqlite_db::FeatureInput {
        name: "auth".to_owned(),
        display_name: "Auth".to_owned(),
        acronym: "AUTH".to_owned(),
        ..Default::default()
    })
    .unwrap();
    db.add_feature_comment(sqlite_db::AddFeatureCommentInput {
        feature_name: "auth".to_owned(),
        category: "gotcha".to_owned(),
        discipline: None,
        agent_task_id: None,
        body: "Use JWT".to_owned(),
        summary: None,
        reason: None,
        source_iteration: None,
    })
    .unwrap();
    let comment_id = db.get_features()[0].comments[0].id;

    let text = build_embedding_text("gotcha", "Use JWT", None);
    let hash = ralph_rag::embedding::hash_text(&text);
    db.upsert_comment_embedding(comment_id, &[0.1; 768], "test", &hash)
        .unwrap();

    let result = should_embed(&db, comment_id, "gotcha", "Use JWT", None);
    assert!(result.is_none());
}

#[tokio::test]
async fn should_embed_changed_comment() {
    let db = sqlite_db::SqliteDb::open_in_memory(None).unwrap();
    db.create_feature(sqlite_db::FeatureInput {
        name: "auth".to_owned(),
        display_name: "Auth".to_owned(),
        acronym: "AUTH".to_owned(),
        ..Default::default()
    })
    .unwrap();
    db.add_feature_comment(sqlite_db::AddFeatureCommentInput {
        feature_name: "auth".to_owned(),
        category: "gotcha".to_owned(),
        discipline: None,
        agent_task_id: None,
        body: "Use JWT".to_owned(),
        summary: None,
        reason: None,
        source_iteration: None,
    })
    .unwrap();
    let comment_id = db.get_features()[0].comments[0].id;

    db.upsert_comment_embedding(comment_id, &[0.1; 768], "test", "old_stale_hash")
        .unwrap();

    let result = should_embed(&db, comment_id, "gotcha", "Use JWT", None);
    assert!(result.is_some());
}

#[tokio::test]
async fn embed_text_calls_ollama() {
    let server = mock_ollama_embed(vec![0.1, 0.2, 0.3]).await;
    let ollama = test_ollama_config(&server);
    let config = test_embedding_config(&ollama);

    let result = embed_text(&config, "gotcha: Use JWT").await.unwrap();
    assert_eq!(result.vector, vec![0.1, 0.2, 0.3]);
    assert_eq!(result.model, "nomic-embed-text");
    assert!(!result.hash.is_empty());
}

#[tokio::test]
async fn embed_text_ollama_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/embed"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let ollama = test_ollama_config(&server);
    let config = test_embedding_config(&ollama);

    let result = embed_text(&config, "test text").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn embed_query_calls_ollama() {
    let server = mock_ollama_embed(vec![0.4, 0.5, 0.6]).await;
    let ollama = test_ollama_config(&server);
    let config = test_embedding_config(&ollama);

    let result = embed_query(&config, "what is JWT?").await.unwrap();
    assert_eq!(result, vec![0.4, 0.5, 0.6]);
}

#[tokio::test]
async fn embed_text_document_prefix() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/embed"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({ "embeddings": [[0.1, 0.2, 0.3]] })),
        )
        .expect(1)
        .mount(&server)
        .await;

    let ollama = test_ollama_config(&server);
    let config = test_embedding_config(&ollama);

    embed_text(&config, "gotcha: test").await.unwrap();
    // wiremock expect(1) validates exactly one call was made
}
