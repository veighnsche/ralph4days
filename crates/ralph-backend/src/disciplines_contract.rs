use ralph_macros::ipc_type;

#[ipc_type]
pub struct McpServerConfigData {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: std::collections::HashMap<String, String>,
}

#[ipc_type]
pub struct CropBoxData {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[ipc_type]
pub struct DisciplineCropsData {
    pub face: CropBoxData,
    pub card: CropBoxData,
    pub upperbody: Option<CropBoxData>,
    pub portrait: Option<CropBoxData>,
    pub landscape: Option<CropBoxData>,
    pub strip: Option<CropBoxData>,
}

#[ipc_type]
pub struct DisciplineImagePromptData {
    pub positive: String,
    pub negative: String,
}

#[ipc_type]
pub struct DisciplineTaskTemplateData {
    pub id: u32,
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub hints: Option<String>,
    pub estimated_turns: Option<u32>,
    pub agent: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub thinking: Option<bool>,
    pub pseudocode: Option<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub pulled_count: u32,
}

#[ipc_type]
pub struct DisciplineConfig {
    pub id: u32,
    pub name: String,
    pub display_name: String,
    pub icon: String,
    pub color: String,
    pub acronym: String,
    pub description: Option<String>,
    pub system_prompt: Option<String>,
    pub agent: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub thinking: Option<bool>,
    pub skills: Vec<String>,
    pub conventions: Option<String>,
    pub mcp_servers: Vec<McpServerConfigData>,
    pub stack_id: Option<u8>,
    pub image_path: Option<String>,
    pub crops: Option<DisciplineCropsData>,
    pub image_prompt: Option<DisciplineImagePromptData>,
    pub task_templates: Vec<DisciplineTaskTemplateData>,
}

#[ipc_type]
pub struct DisciplinesCreateArgs {
    pub name: String,
    pub display_name: String,
    pub acronym: String,
    pub icon: String,
    pub color: String,
    pub system_prompt: Option<String>,
    pub agent: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub thinking: Option<bool>,
    pub skills: Vec<String>,
    pub conventions: Option<String>,
    pub mcp_servers: Vec<McpServerConfigData>,
}

#[ipc_type]
pub struct DisciplinesUpdateArgs {
    pub name: String,
    pub display_name: String,
    pub acronym: String,
    pub icon: String,
    pub color: String,
    pub system_prompt: Option<String>,
    pub agent: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub thinking: Option<bool>,
    pub skills: Vec<String>,
    pub conventions: Option<String>,
    pub mcp_servers: Vec<McpServerConfigData>,
}

#[ipc_type]
pub struct DisciplinesDeleteArgs {
    pub name: String,
}

#[ipc_type]
pub struct DisciplinesImageDataGetArgs {
    pub discipline_name: String,
}

#[ipc_type]
pub struct DisciplinesCroppedImageGetArgs {
    pub discipline_name: String,
    pub crop: CropBoxData,
    pub label: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discipline_config_serializes_required_arrays_even_when_empty() {
        let cfg = DisciplineConfig {
            id: 1,
            name: "frontend".to_owned(),
            display_name: "Frontend".to_owned(),
            icon: "code".to_owned(),
            color: "#000000".to_owned(),
            acronym: "FE".to_owned(),
            description: None,
            system_prompt: None,
            agent: None,
            model: None,
            effort: None,
            thinking: None,
            skills: vec![],
            conventions: None,
            mcp_servers: vec![],
            stack_id: None,
            image_path: None,
            crops: None,
            image_prompt: None,
            task_templates: vec![],
        };

        let value = serde_json::to_value(cfg).expect("DisciplineConfig should serialize");
        let obj = value
            .as_object()
            .expect("DisciplineConfig should serialize to an object");

        let skills = obj
            .get("skills")
            .expect("skills should be present")
            .as_array()
            .expect("skills should be an array");
        assert!(
            skills.is_empty(),
            "skills should serialize as an empty array"
        );

        let mcp_servers = obj
            .get("mcpServers")
            .expect("mcpServers should be present")
            .as_array()
            .expect("mcpServers should be an array");
        assert!(
            mcp_servers.is_empty(),
            "mcpServers should serialize as an empty array"
        );

        let task_templates = obj
            .get("taskTemplates")
            .expect("taskTemplates should be present")
            .as_array()
            .expect("taskTemplates should be an array");
        assert!(
            task_templates.is_empty(),
            "taskTemplates should serialize as an empty array"
        );
    }
}
