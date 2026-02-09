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
    #[serde(default)]
    pub generation: Option<GenerationSettings>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GenerationQuality {
    pub steps: u32,
    pub megapixels: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GenerationSettings {
    pub dev: GenerationQuality,
    pub prod: GenerationQuality,
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
    (
        3,
        &[
            include_str!("defaults/disciplines/03_saas/00_nextjs_app_router.yaml"),
            include_str!("defaults/disciplines/03_saas/01_react_components.yaml"),
            include_str!("defaults/disciplines/03_saas/02_api_routes.yaml"),
            include_str!("defaults/disciplines/03_saas/03_database_prisma.yaml"),
            include_str!("defaults/disciplines/03_saas/04_authentication.yaml"),
            include_str!("defaults/disciplines/03_saas/05_monorepo_deployment.yaml"),
            include_str!("defaults/disciplines/03_saas/06_testing.yaml"),
            include_str!("defaults/disciplines/03_saas/07_documentation.yaml"),
        ],
    ),
    (
        4,
        &[
            include_str!("defaults/disciplines/04_mobile/00_flutter_ui.yaml"),
            include_str!("defaults/disciplines/04_mobile/01_dart_logic.yaml"),
            include_str!("defaults/disciplines/04_mobile/02_firebase_backend.yaml"),
            include_str!("defaults/disciplines/04_mobile/03_state_management.yaml"),
            include_str!("defaults/disciplines/04_mobile/04_platform_integration.yaml"),
            include_str!("defaults/disciplines/04_mobile/05_testing.yaml"),
            include_str!("defaults/disciplines/04_mobile/06_app_distribution.yaml"),
            include_str!("defaults/disciplines/04_mobile/07_documentation.yaml"),
        ],
    ),
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
