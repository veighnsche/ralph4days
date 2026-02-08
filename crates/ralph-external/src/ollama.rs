use crate::config::OllamaConfig;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaStatus {
    pub available: bool,
    pub models: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
struct EmbedRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct EmbedResponse {
    embeddings: Vec<Vec<f32>>,
}

#[derive(Debug, Serialize)]
struct GenerateRequest {
    model: String,
    prompt: String,
    temperature: f32,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct GenerateResponse {
    response: String,
}

#[derive(Debug, Deserialize)]
struct TagsResponse {
    models: Vec<ModelInfo>,
}

#[derive(Debug, Deserialize)]
struct ModelInfo {
    name: String,
}

pub async fn check_available(config: &OllamaConfig) -> OllamaStatus {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .expect("Failed to build HTTP client");

    match client
        .get(format!("{}/api/tags", config.api_url))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => match resp.json::<TagsResponse>().await {
            Ok(tags) => OllamaStatus {
                available: true,
                models: tags.models.into_iter().map(|m| m.name).collect(),
                error: None,
            },
            Err(e) => OllamaStatus {
                available: false,
                models: vec![],
                error: Some(format!("Failed to parse response: {e}")),
            },
        },
        Ok(resp) => OllamaStatus {
            available: false,
            models: vec![],
            error: Some(format!("HTTP error: {}", resp.status())),
        },
        Err(e) => OllamaStatus {
            available: false,
            models: vec![],
            error: Some(format!("Cannot reach Ollama: {e}")),
        },
    }
}

pub async fn embed_texts(
    config: &OllamaConfig,
    texts: Vec<String>,
) -> Result<Vec<Vec<f32>>, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

    let request = EmbedRequest {
        model: config.embedding_model.clone(),
        input: texts,
    };

    let response = client
        .post(format!("{}/api/embed", config.api_url))
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Embedding request failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("Embedding failed: HTTP {}", response.status()));
    }

    let embed_response: EmbedResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse embedding response: {e}"))?;

    for embedding in &embed_response.embeddings {
        if embedding.len() != config.embedding_dims as usize {
            return Err(format!(
                "Expected {} dimensions, got {}",
                config.embedding_dims,
                embedding.len()
            ));
        }
    }

    Ok(embed_response.embeddings)
}

pub async fn generate_text(config: &OllamaConfig, prompt: String) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

    let request = GenerateRequest {
        model: config.llm_model.clone(),
        prompt,
        temperature: config.llm_temperature,
        stream: false,
    };

    let response = client
        .post(format!("{}/api/generate", config.api_url))
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Generation request failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("Generation failed: HTTP {}", response.status()));
    }

    let gen_response: GenerateResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse generation response: {e}"))?;

    Ok(gen_response.response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn check_available_handles_unreachable() {
        let config = OllamaConfig {
            api_url: "http://localhost:99999".into(),
            embedding_model: "test".into(),
            embedding_dims: 768,
            llm_model: "test".into(),
            llm_temperature: 0.7,
        };

        let status = check_available(&config).await;
        assert!(!status.available);
        assert!(status.error.is_some());
    }
}
