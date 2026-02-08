pub mod comfy;
pub mod comfy_discipline;
pub mod config;
pub mod ollama;

pub use comfy::{
    check_available as check_comfy_available, generate_audio, generate_image, ComfyStatus,
    GenerationProgress,
};
pub use comfy_discipline::{
    generate_discipline_portrait, generate_discipline_portrait_with_progress, DisciplinePrompts,
};
pub use config::{ComfyConfig, ExternalServicesConfig, OllamaConfig};
pub use ollama::{
    check_available as check_ollama_available, embed_texts, generate_text, OllamaStatus,
};
