use crate::comfy::{randomize_seed, run_workflow, GenerationProgress, WorkflowNode};
use std::collections::HashMap;

pub struct DisciplinePrompts {
    pub positive_global: String,
    pub negative_global: String,
    pub positive_stack: String,
    pub negative_stack: String,
    pub positive_discipline: String,
    pub negative_discipline: String,
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
    inject_discipline_prompts(workflow, prompts)?;
    randomize_seed(workflow);

    run_workflow(config, std::mem::take(workflow), "images", on_progress).await
}

fn inject_discipline_prompts(
    workflow: &mut HashMap<String, WorkflowNode>,
    prompts: DisciplinePrompts,
) -> Result<(), String> {
    let replacements = vec![
        ("__POSITIVE_GLOBAL__", prompts.positive_global),
        ("__NEGATIVE_GLOBAL__", prompts.negative_global),
        ("__POSITIVE_STACK__", prompts.positive_stack),
        ("__NEGATIVE_STACK__", prompts.negative_stack),
        ("__POSITIVE_DISCIPLINE__", prompts.positive_discipline),
        ("__NEGATIVE_DISCIPLINE__", prompts.negative_discipline),
    ];

    let mut found_count = 0;

    for node in workflow.values_mut() {
        if node.class_type == "CLIPTextEncode" {
            if let Some(text) = node.inputs.get("text") {
                if let Some(text_str) = text.as_str() {
                    for (marker, replacement) in &replacements {
                        if text_str.contains(marker) {
                            node.inputs.insert(
                                "text".into(),
                                serde_json::Value::String(replacement.clone()),
                            );
                            found_count += 1;
                            break;
                        }
                    }
                }
            }
        }
    }

    if found_count < 5 {
        return Err(format!(
            "Workflow must have CLIPTextEncode nodes with all prompt markers. Found {found_count}/6"
        ));
    }

    Ok(())
}
