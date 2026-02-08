use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct VisualIdentity {
    pub style: String,
    pub theme: String,
    pub tone: String,
    pub references: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StackImagePrompt {
    pub positive: String,
    pub negative: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StackMetadata {
    pub stack_id: u8,
    pub name: String,
    pub description: String,
    pub philosophy: String,
    pub visual_identity: VisualIdentity,
    pub when_to_use: Vec<String>,
    pub discipline_count: u8,
    pub characteristics: Vec<String>,
    #[serde(default)]
    pub image_prompt: Option<StackImagePrompt>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DisciplineImagePrompt {
    pub positive: String,
    pub negative: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DisciplineDef {
    pub name: String,
    pub display_name: String,
    #[serde(default)]
    pub acronym: String,
    pub icon: String,
    pub color: String,
    #[serde(default)]
    pub description: Option<String>,
    pub conventions: String,
    pub skills: Vec<String>,
    pub system_prompt: String,
    #[serde(default)]
    pub image_prompt: Option<DisciplineImagePrompt>,
}

const STACK_ABOUTS: &[(u8, &str)] = &[
    (
        1,
        include_str!("defaults/disciplines/01_generic/ABOUT.yaml"),
    ),
    (
        2,
        include_str!("defaults/disciplines/02_desktop/ABOUT.yaml"),
    ),
    (3, include_str!("defaults/disciplines/03_saas/ABOUT.yaml")),
    (4, include_str!("defaults/disciplines/04_mobile/ABOUT.yaml")),
];

const STACK_DISCIPLINES: &[(u8, &[&str])] = &[
    (
        1,
        &[
            include_str!("defaults/disciplines/01_generic/00_implementation.yaml"),
            include_str!("defaults/disciplines/01_generic/01_refactoring.yaml"),
            include_str!("defaults/disciplines/01_generic/02_investigation.yaml"),
            include_str!("defaults/disciplines/01_generic/03_testing.yaml"),
            include_str!("defaults/disciplines/01_generic/04_architecture.yaml"),
            include_str!("defaults/disciplines/01_generic/05_devops.yaml"),
            include_str!("defaults/disciplines/01_generic/06_security.yaml"),
            include_str!("defaults/disciplines/01_generic/07_documentation.yaml"),
        ],
    ),
    (
        2,
        &[
            include_str!("defaults/disciplines/02_desktop/00_frontend.yaml"),
            include_str!("defaults/disciplines/02_desktop/01_backend.yaml"),
            include_str!("defaults/disciplines/02_desktop/02_data.yaml"),
            include_str!("defaults/disciplines/02_desktop/03_integration.yaml"),
            include_str!("defaults/disciplines/02_desktop/04_platform.yaml"),
            include_str!("defaults/disciplines/02_desktop/05_quality.yaml"),
            include_str!("defaults/disciplines/02_desktop/06_security.yaml"),
            include_str!("defaults/disciplines/02_desktop/07_documentation.yaml"),
        ],
    ),
    // TODO: Add 03_saas and 04_mobile disciplines when created
];

pub fn get_all_stack_metadata() -> Vec<StackMetadata> {
    STACK_ABOUTS
        .iter()
        .filter_map(|(id, content)| {
            serde_yaml::from_str::<StackMetadata>(content)
                .ok()
                .filter(|m| m.stack_id == *id)
        })
        .collect()
}

pub fn get_stack_metadata(stack_id: u8) -> Option<StackMetadata> {
    STACK_ABOUTS
        .iter()
        .find(|(id, _)| *id == stack_id)
        .and_then(|(_, content)| serde_yaml::from_str::<StackMetadata>(content).ok())
}

#[derive(Debug, Clone, Deserialize)]
pub struct GlobalImagePrompts {
    pub global: GlobalPromptPair,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GlobalPromptPair {
    pub positive: String,
    pub negative: String,
}

const GLOBAL_IMAGE_PROMPTS: &str = include_str!("image_prompts.yaml");

pub const DISCIPLINE_WORKFLOW: &str =
    include_str!("comfyui_workflows/generate_discipline.json");

pub fn get_global_image_prompts() -> GlobalImagePrompts {
    serde_yaml::from_str(GLOBAL_IMAGE_PROMPTS).expect("embedded image_prompts.yaml is valid")
}

pub fn get_disciplines_for_stack(stack_id: u8) -> Vec<DisciplineDef> {
    STACK_DISCIPLINES
        .iter()
        .find(|(id, _)| *id == stack_id)
        .map(|(_, yamls)| {
            yamls
                .iter()
                .filter_map(|content| serde_yaml::from_str::<DisciplineDef>(content).ok())
                .collect()
        })
        .unwrap_or_default()
}
