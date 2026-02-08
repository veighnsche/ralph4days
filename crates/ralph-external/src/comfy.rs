use crate::config::ComfyConfig;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComfyStatus {
    pub available: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct PromptRequest {
    pub prompt: HashMap<String, WorkflowNode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowNode {
    pub inputs: HashMap<String, serde_json::Value>,
    pub class_type: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct QueueResponse {
    pub prompt_id: String,
}

#[derive(Debug, Clone)]
pub struct GenerationProgress {
    pub step: u32,
    pub total: u32,
    pub node: String,
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

pub async fn generate_audio(
    config: &ComfyConfig,
    text: String,
    workflow_name: &str,
) -> Result<Vec<u8>, String> {
    let workflow_path = get_workflow_path_by_name(workflow_name)?;

    let workflow_json = std::fs::read_to_string(&workflow_path)
        .map_err(|e| format!("Failed to read workflow file: {e}"))?;

    let mut workflow: HashMap<String, WorkflowNode> = serde_json::from_str(&workflow_json)
        .map_err(|e| format!("Failed to parse workflow JSON: {e}"))?;

    inject_text(&mut workflow, text)?;

    run_workflow(config, workflow, "audio", |_| {}).await
}

pub async fn generate_image(
    config: &ComfyConfig,
    positive_prompt: String,
    negative_prompt: String,
) -> Result<Vec<u8>, String> {
    let workflow_path = get_workflow_path_by_name(&config.default_workflow)?;

    let workflow_json = std::fs::read_to_string(&workflow_path)
        .map_err(|e| format!("Failed to read workflow file: {e}"))?;

    let mut workflow: HashMap<String, WorkflowNode> = serde_json::from_str(&workflow_json)
        .map_err(|e| format!("Failed to parse workflow JSON: {e}"))?;

    inject_prompts(&mut workflow, positive_prompt, negative_prompt)?;

    run_workflow(config, workflow, "images", |_| {}).await
}

/// Queue a workflow, wait for completion via WebSocket, fetch output bytes.
pub(crate) async fn run_workflow(
    config: &ComfyConfig,
    workflow: HashMap<String, WorkflowNode>,
    output_type: &str,
    on_progress: impl Fn(GenerationProgress),
) -> Result<Vec<u8>, String> {
    let client_id = uuid::Uuid::new_v4().to_string();

    let ws_url = config
        .api_url
        .replace("http://", "ws://")
        .replace("https://", "wss://");
    let ws_url = format!("{ws_url}/ws?clientId={client_id}");

    let (ws_stream, _) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .map_err(|e| format!("Failed to connect WebSocket: {e}"))?;

    let (_, mut read) = ws_stream.split();

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

    let prompt_id = &queue_response.prompt_id;

    let timeout = tokio::time::sleep(Duration::from_secs(config.timeout_secs));
    tokio::pin!(timeout);

    let mut execution_started = false;

    loop {
        tokio::select! {
            msg = read.next() => {
                let Some(Ok(msg)) = msg else { continue };
                let tokio_tungstenite::tungstenite::Message::Text(text) = msg else { continue };

                let Ok(data) = serde_json::from_str::<serde_json::Value>(&text) else { continue };

                let msg_type = data.get("type").and_then(|t| t.as_str()).unwrap_or("");
                let msg_data = data.get("data");

                match msg_type {
                    "progress" => {
                        execution_started = true;
                        if let Some(d) = msg_data {
                            let pid = d.get("prompt_id").and_then(|p| p.as_str()).unwrap_or("");
                            if pid == prompt_id {
                                let step = d.get("value").and_then(serde_json::Value::as_u64).unwrap_or(0) as u32;
                                let total = d.get("max").and_then(serde_json::Value::as_u64).unwrap_or(0) as u32;
                                let node = d.get("node").and_then(|v| v.as_str()).unwrap_or("").to_owned();
                                on_progress(GenerationProgress { step, total, node });
                            }
                        }
                    }
                    "progress_state" => {
                        execution_started = true;
                    }
                    "status" => {
                        if execution_started {
                            let queue_remaining = msg_data
                                .and_then(|d| d.get("status"))
                                .and_then(|s| s.get("exec_info"))
                                .and_then(|e| e.get("queue_remaining"))
                                .and_then(serde_json::Value::as_u64);
                            if queue_remaining == Some(0) {
                                break;
                            }
                        }
                    }
                    "executing" => {
                        if let Some(d) = msg_data {
                            let node = d.get("node");
                            if node.is_none() || node.map_or(false, |n| n.is_null()) {
                                break;
                            }
                        }
                    }
                    "executed" => {
                        if let Some(d) = msg_data {
                            let pid = d.get("prompt_id").and_then(|p| p.as_str()).unwrap_or("");
                            if pid == prompt_id && d.get("output").is_some() {
                                break;
                            }
                        }
                    }
                    _ => {}
                }
            }
            () = &mut timeout => {
                return Err("Generation timed out".into());
            }
        }
    }

    fetch_generated_output(&client, config, prompt_id, output_type).await
}

pub fn randomize_seed(workflow: &mut HashMap<String, WorkflowNode>) {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;

    for node in workflow.values_mut() {
        if node.class_type == "KSampler" {
            node.inputs.insert("seed".into(), serde_json::json!(seed));
        }
    }
}

pub fn set_steps(workflow: &mut HashMap<String, WorkflowNode>, steps: u32) {
    for node in workflow.values_mut() {
        if node.class_type == "KSampler" {
            node.inputs.insert("steps".into(), serde_json::json!(steps));
        }
    }
}

pub fn set_dimensions(workflow: &mut HashMap<String, WorkflowNode>, width: u32, height: u32) {
    for node in workflow.values_mut() {
        if node.class_type.contains("Latent") {
            if node.inputs.contains_key("width") {
                node.inputs.insert("width".into(), serde_json::json!(width));
            }
            if node.inputs.contains_key("height") {
                node.inputs.insert("height".into(), serde_json::json!(height));
            }
        }
    }
}

pub fn compute_dimensions(ratio_w: f64, ratio_h: f64, megapixels: f64) -> (u32, u32) {
    let total_pixels = megapixels * 1_048_576.0;
    let w = (total_pixels * ratio_w / ratio_h).sqrt();
    let h = (total_pixels * ratio_h / ratio_w).sqrt();
    let w = ((w / 8.0).round() * 8.0) as u32;
    let h = ((h / 8.0).round() * 8.0) as u32;
    (w, h)
}

fn get_workflow_path_by_name(workflow_name: &str) -> Result<std::path::PathBuf, String> {
    let config_dir = dirs::config_dir()
        .ok_or("No config directory on this platform")?
        .join("ralph")
        .join("workflows");

    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create workflows dir: {e}"))?;

    let workflow_path = config_dir.join(workflow_name);

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

fn inject_text(workflow: &mut HashMap<String, WorkflowNode>, text: String) -> Result<(), String> {
    let mut found_text_input = false;

    for node in workflow.values_mut() {
        if let Some(text_field) = node.inputs.get("text") {
            if text_field.as_str().unwrap_or("").contains("tts_input_text") {
                node.inputs
                    .insert("text".into(), serde_json::Value::String(text));
                found_text_input = true;
                break;
            }
        }
    }

    if !found_text_input {
        return Err(
            "Workflow must have a node with 'text' input containing 'tts_input_text' marker".into(),
        );
    }

    Ok(())
}

async fn fetch_generated_output(
    client: &reqwest::Client,
    config: &ComfyConfig,
    prompt_id: &str,
    output_type: &str,
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

    let output_files =
        find_first_output(outputs, output_type).ok_or("No outputs in generated result")?;

    let filename = output_files
        .get(0)
        .and_then(|file: &serde_json::Value| file.get("filename"))
        .and_then(|f: &serde_json::Value| f.as_str())
        .ok_or("No filename in output")?;

    let file_response = client
        .get(format!("{}/view?filename={}", config.api_url, filename))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch output file: {e}"))?;

    let file_bytes = file_response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read output bytes: {e}"))?;

    Ok(file_bytes.to_vec())
}

fn find_first_output<'a>(
    outputs: &'a serde_json::Value,
    output_type: &str,
) -> Option<&'a serde_json::Value> {
    if let serde_json::Value::Object(map) = outputs {
        for value in map.values() {
            if let Some(output_array) = value.get(output_type) {
                if output_array.is_array() && !output_array.as_array().unwrap().is_empty() {
                    return Some(output_array);
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
