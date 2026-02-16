use core_macros::ipc_type;
use std::collections::HashMap;

#[ipc_type]
pub struct SectionSettingsData {
    pub enabled: bool,
    pub instruction_override: Option<String>,
}

#[ipc_type]
pub struct PromptBuilderConfigInput {
    pub name: String,
    pub base_prompt: String,
    pub section_order: Vec<String>,
    pub sections: HashMap<String, SectionSettingsData>,
}

#[ipc_type]
pub struct PromptBuilderConfigData {
    pub name: String,
    pub base_prompt: String,
    pub section_order: Vec<String>,
    pub sections: HashMap<String, SectionSettingsData>,
    pub created: Option<String>,
    pub updated: Option<String>,
}

#[ipc_type]
pub struct PromptBuilderConfigGetArgs {
    pub name: String,
}

#[ipc_type]
pub struct PromptBuilderConfigSaveArgs {
    pub config: PromptBuilderConfigInput,
}

#[ipc_type]
pub struct PromptBuilderConfigDeleteArgs {
    pub name: String,
}

#[ipc_type]
pub struct PromptPreviewSection {
    pub name: String,
    pub content: String,
}

#[ipc_type]
pub struct PromptPreview {
    pub sections: Vec<PromptPreviewSection>,
    pub full_prompt: String,
}

#[ipc_type]
pub struct SectionConfig {
    pub name: String,
    pub enabled: bool,
    pub instruction_override: Option<String>,
}

#[ipc_type]
pub struct PromptBuilderPreviewArgs {
    pub sections: Vec<SectionConfig>,
    pub user_input: Option<String>,
}
