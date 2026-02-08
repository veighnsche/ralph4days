use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualIdentity {
    pub style: String,
    pub theme: String,
    pub tone: String,
    pub references: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackMetadata {
    pub stack_id: u8,
    pub name: String,
    pub description: String,
    pub philosophy: String,
    pub visual_identity: VisualIdentity,
    pub when_to_use: Vec<String>,
    pub discipline_count: u8,
    pub characteristics: Vec<String>,
}

const STACKS: &[(u8, &str)] = &[
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

pub fn get_all_stack_metadata() -> Vec<StackMetadata> {
    STACKS
        .iter()
        .filter_map(|(stack_id, content)| {
            serde_yaml::from_str::<StackMetadata>(content)
                .ok()
                .filter(|m| m.stack_id == *stack_id)
        })
        .collect()
}

pub fn get_stack_metadata(stack_id: u8) -> Option<StackMetadata> {
    STACKS
        .iter()
        .find(|(id, _)| *id == stack_id)
        .and_then(|(_, content)| serde_yaml::from_str::<StackMetadata>(content).ok())
}
