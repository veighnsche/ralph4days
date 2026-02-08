use crate::config::ComfyConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComfyStatus {
    pub available: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
struct PromptRequest {
    prompt: HashMap<String, WorkflowNode>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkflowNode {
    inputs: HashMap<String, serde_json::Value>,
    class_type: String,
}

#[derive(Debug, Deserialize)]
struct QueueResponse {
    prompt_id: String,
}

pub async fn check_available(config: &ComfyConfig) -> ComfyStatus {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .expect("Failed to build HTTP client");

    match client
        .get(format!("{}/system_stats", config.api_url))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => ComfyStatus {
            available: true,
            error: None,
        },
        Ok(resp) => ComfyStatus {
            available: false,
            error: Some(format!("HTTP error: {}", resp.status())),
        },
        Err(e) => ComfyStatus {
            available: false,
            error: Some(format!("Cannot reach ComfyUI: {e}")),
        },
    }
}

pub async fn generate_image(
    config: &ComfyConfig,
    positive_prompt: String,
    negative_prompt: String,
) -> Result<Vec<u8>, String> {
    let workflow_path = get_workflow_path(config)?;

    let workflow_json = std::fs::read_to_string(&workflow_path)
        .map_err(|e| format!("Failed to read workflow file: {e}"))?;

    let mut workflow: HashMap<String, WorkflowNode> = serde_json::from_str(&workflow_json)
        .map_err(|e| format!("Failed to parse workflow JSON: {e}"))?;

    inject_prompts(&mut workflow, positive_prompt, negative_prompt)?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(config.timeout_secs))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

    let prompt_request = PromptRequest { prompt: workflow };

    let response = client
        .post(format!("{}/prompt", config.api_url))
        .json(&prompt_request)
        .send()
        .await
        .map_err(|e| format!("Failed to queue prompt: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("ComfyUI error: HTTP {}", response.status()));
    }

    let queue_response: QueueResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse queue response: {e}"))?;

    poll_until_complete(&client, config, &queue_response.prompt_id).await?;

    fetch_generated_image(&client, config, &queue_response.prompt_id).await
}

fn get_workflow_path(config: &ComfyConfig) -> Result<std::path::PathBuf, String> {
    let config_dir = dirs::config_dir()
        .ok_or("No config directory on this platform")?
        .join("ralph")
        .join("workflows");

    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create workflows dir: {e}"))?;

    let workflow_path = config_dir.join(&config.default_workflow);

    if !workflow_path.starts_with(&config_dir) {
        return Err("Workflow path escapes workflows directory".into());
    }

    if !workflow_path.exists() {
        return Err(format!(
            "Workflow file not found: {}. Place your ComfyUI workflow JSON at this location.",
            workflow_path.display()
        ));
    }

    Ok(workflow_path)
}

fn inject_prompts(
    workflow: &mut HashMap<String, WorkflowNode>,
    positive: String,
    negative: String,
) -> Result<(), String> {
    let mut found_positive = false;
    let mut found_negative = false;

    for node in workflow.values_mut() {
        if node.class_type == "CLIPTextEncode" {
            if let Some(text) = node.inputs.get("text") {
                if text.as_str().unwrap_or("").contains("positive_prompt") {
                    node.inputs
                        .insert("text".into(), serde_json::Value::String(positive.clone()));
                    found_positive = true;
                } else if text.as_str().unwrap_or("").contains("negative_prompt") {
                    node.inputs
                        .insert("text".into(), serde_json::Value::String(negative.clone()));
                    found_negative = true;
                }
            }
        }
    }

    if !found_positive || !found_negative {
        return Err(
            "Workflow must have CLIPTextEncode nodes with 'positive_prompt' and 'negative_prompt' markers".into()
        );
    }

    Ok(())
}

async fn poll_until_complete(
    client: &reqwest::Client,
    config: &ComfyConfig,
    prompt_id: &str,
) -> Result<(), String> {
    let max_polls = config.timeout_secs / 5;

    for _ in 0..max_polls {
        tokio::time::sleep(Duration::from_secs(5)).await;

        let history_response = client
            .get(format!("{}/history/{}", config.api_url, prompt_id))
            .send()
            .await
            .map_err(|e| format!("Failed to check history: {e}"))?;

        if history_response.status().is_success() {
            let history: serde_json::Value = history_response
                .json()
                .await
                .map_err(|e| format!("Failed to parse history: {e}"))?;

            if history.get(prompt_id).is_some() {
                return Ok(());
            }
        }
    }

    Err("Image generation timed out".into())
}

async fn fetch_generated_image(
    client: &reqwest::Client,
    config: &ComfyConfig,
    prompt_id: &str,
) -> Result<Vec<u8>, String> {
    let history_response = client
        .get(format!("{}/history/{}", config.api_url, prompt_id))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch history: {e}"))?;

    let history: serde_json::Value = history_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse history: {e}"))?;

    let outputs = history
        .get(prompt_id)
        .and_then(|h| h.get("outputs"))
        .ok_or("No outputs in history")?;

    let images = find_first_image_output(outputs).ok_or("No images in output")?;

    let filename = images
        .get(0)
        .and_then(|img| img.get("filename"))
        .and_then(|f| f.as_str())
        .ok_or("No filename in image output")?;

    let image_response = client
        .get(format!("{}/view?filename={}", config.api_url, filename))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch image: {e}"))?;

    let image_bytes = image_response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read image bytes: {e}"))?;

    Ok(image_bytes.to_vec())
}

fn find_first_image_output(outputs: &serde_json::Value) -> Option<&serde_json::Value> {
    if let serde_json::Value::Object(map) = outputs {
        for value in map.values() {
            if let Some(images) = value.get("images") {
                if images.is_array() && !images.as_array().unwrap().is_empty() {
                    return Some(images);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn check_available_handles_unreachable() {
        let config = ComfyConfig {
            api_url: "http://localhost:99999".into(),
            default_workflow: "test.json".into(),
            timeout_secs: 10,
        };

        let status = check_available(&config).await;
        assert!(!status.available);
        assert!(status.error.is_some());
    }

    #[test]
    fn inject_prompts_validates_workflow() {
        let mut workflow = HashMap::new();

        let result = inject_prompts(&mut workflow, "test".into(), "test".into());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("CLIPTextEncode"));
    }
}
