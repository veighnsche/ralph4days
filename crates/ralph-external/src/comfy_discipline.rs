use crate::comfy::{randomize_seed, run_workflow, GenerationProgress, WorkflowNode};
use std::collections::HashMap;

pub struct DisciplinePrompts {
    pub positive: String,
    pub negative: String,
}

pub async fn generate_discipline_portrait(
    config: &crate::config::ComfyConfig,
    prompts: DisciplinePrompts,
    workflow: &mut HashMap<String, WorkflowNode>,
) -> Result<Vec<u8>, String> {
    generate_discipline_portrait_with_progress(config, prompts, workflow, |_| {}).await
}

pub async fn generate_discipline_portrait_with_progress(
    config: &crate::config::ComfyConfig,
    prompts: DisciplinePrompts,
    workflow: &mut HashMap<String, WorkflowNode>,
    on_progress: impl Fn(GenerationProgress),
) -> Result<Vec<u8>, String> {
    inject_prompts(workflow, prompts)?;
    randomize_seed(workflow);

    run_workflow(config, std::mem::take(workflow), "images", on_progress).await
}

fn inject_prompts(
    workflow: &mut HashMap<String, WorkflowNode>,
    prompts: DisciplinePrompts,
) -> Result<(), String> {
    let mut found_positive = false;
    let mut found_negative = false;

    for node in workflow.values_mut() {
        if node.class_type == "CLIPTextEncode" {
            if let Some(text) = node.inputs.get("text") {
                if let Some(text_str) = text.as_str() {
                    if text_str.contains("__POSITIVE__") {
                        node.inputs.insert(
                            "text".into(),
                            serde_json::Value::String(prompts.positive.clone()),
                        );
                        found_positive = true;
                    } else if text_str.contains("__NEGATIVE__") {
                        node.inputs.insert(
                            "text".into(),
                            serde_json::Value::String(prompts.negative.clone()),
                        );
                        found_negative = true;
                    }
                }
            }
        }
    }

    if !found_positive || !found_negative {
        return Err(
            "Workflow must have CLIPTextEncode nodes with __POSITIVE__ and __NEGATIVE__ markers"
                .into(),
        );
    }

    Ok(())
}
