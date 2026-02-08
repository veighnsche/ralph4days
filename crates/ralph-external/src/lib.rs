//! External service integrations for Ralph.
//!
//! This crate provides clients for optional external AI services:
//! - **Ollama**: Local LLM (embeddings + text generation)
//! - **ComfyUI**: Image generation via workflows
//!
//! All services are **gated** - they only activate when:
//! 1. User has configured them in settings
//! 2. Services are reachable and healthy
//!
//! If unavailable, Ralph continues normally without these features.

pub mod comfy;
pub mod config;
pub mod ollama;

pub use comfy::{
    check_available as check_comfy_available, generate_audio, generate_image, ComfyStatus,
};
pub use config::{ComfyConfig, ExternalServicesConfig, OllamaConfig};
pub use ollama::{
    check_available as check_ollama_available, embed_texts, generate_text, OllamaStatus,
};
