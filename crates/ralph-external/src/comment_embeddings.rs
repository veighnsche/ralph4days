use crate::config::OllamaConfig;
use crate::ollama;
use ralph_rag::embedding;

pub struct CommentEmbeddingConfig<'a> {
    pub ollama: &'a OllamaConfig,
    pub document_prefix: &'a str,
    pub query_prefix: &'a str,
    pub min_search_score: f32,
    pub max_search_results: u32,
}

pub struct EmbeddingResult {
    pub vector: Vec<f32>,
    pub model: String,
    pub hash: String,
}

pub fn build_embedding_text(category: &str, body: &str, reason: Option<&str>) -> String {
    embedding::build_embedding_text(category, body, reason)
}

pub fn should_embed(
    db: &sqlite_db::SqliteDb,
    comment_id: u32,
    category: &str,
    body: &str,
    reason: Option<&str>,
) -> Option<String> {
    let embedding_text = embedding::build_embedding_text(category, body, reason);
    let hash = embedding::hash_text(&embedding_text);

    if let Some(existing_hash) = db.get_embedding_hash(comment_id) {
        if existing_hash == hash {
            return None;
        }
    }

    Some(embedding_text)
}

pub async fn embed_text(
    config: &CommentEmbeddingConfig<'_>,
    embedding_text: &str,
) -> Result<EmbeddingResult, String> {
    let hash = embedding::hash_text(embedding_text);
    let text_to_embed = format!("{}{embedding_text}", config.document_prefix);
    let embeddings = ollama::embed_texts(config.ollama, vec![text_to_embed]).await?;
    let vector = embeddings
        .into_iter()
        .next()
        .ok_or("No embedding returned from Ollama")?;

    Ok(EmbeddingResult {
        vector,
        model: config.ollama.embedding_model.clone(),
        hash,
    })
}

pub async fn embed_query(
    config: &CommentEmbeddingConfig<'_>,
    query: &str,
) -> Result<Vec<f32>, String> {
    let text_to_embed = format!("{}{query}", config.query_prefix);
    let embeddings = ollama::embed_texts(config.ollama, vec![text_to_embed]).await?;
    embeddings
        .into_iter()
        .next()
        .ok_or("No embedding returned from Ollama".to_owned())
}
