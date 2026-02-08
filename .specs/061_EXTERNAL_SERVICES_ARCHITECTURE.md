# SPEC-061: External Services Architecture

**Status:** Draft
**Created:** 2026-02-08
**Dependencies:** ralph-rag crate (already exists)

## Overview

Ralph gains **optional, gated integration** with external AI services. These features are:
- **Prompt-on-demand**: Only checked when user attempts to use them
- **Globally configured**: `~/.config/ralph/external_services.json`
- **Gracefully degraded**: App works fully without them

This architecture supports:
1. **Ollama** (embedding + LLM) for RAG and future AI features
2. **ComfyUI** for discipline character imagery generation
3. **Qdrant** (embedded binary) for vector search

## Philosophy

**Ralph remains deterministic by default.** AI features are:
- Explicitly triggered by user (no autonomous AI decision-making)
- Clearly labeled in UI (e.g., "✨ Generate Image" button)
- Optional and user-configured
- Never required for core functionality (task execution, loop control, project management)

Think of these as "power user plugins" — Ralph orchestrates them, but doesn't depend on them.

## Config Schema

### File Location
**Global only**: `~/.config/ralph/external_services.json`

Projects inherit global config (no per-project overrides for now — keeps complexity low).

### Schema v1

```json
{
  "version": 1,
  "qdrant": {
    "rest_url": "http://localhost:6333",
    "grpc_url": "http://localhost:6334"
  },
  "ollama": {
    "api_url": "http://localhost:11434",
    "embedding": {
      "model": "nomic-embed-text",
      "dimensions": 768
    },
    "llm": {
      "model": "qwen2.5-coder:7b",
      "temperature": 0.7,
      "note": "Model must support tool calling for future features"
    }
  },
  "comfy": {
    "api_url": "http://localhost:8188",
    "default_workflow": "discipline_character.json",
    "timeout_secs": 300
  }
}
```

### Rust Types

```rust
// crates/ralph-external-services/src/config.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalServicesConfig {
    pub version: u32,
    pub qdrant: QdrantConfig,
    pub ollama: OllamaConfig,
    pub comfy: ComfyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantConfig {
    pub rest_url: String,
    pub grpc_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub api_url: String,
    pub embedding: EmbeddingConfig,
    pub llm: LlmConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub model: String,
    pub dimensions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub model: String,
    pub temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComfyConfig {
    pub api_url: String,
    pub default_workflow: String,
    pub timeout_secs: u64,
}

impl Default for ExternalServicesConfig {
    fn default() -> Self {
        Self {
            version: 1,
            qdrant: QdrantConfig {
                rest_url: "http://localhost:6333".into(),
                grpc_url: "http://localhost:6334".into(),
            },
            ollama: OllamaConfig {
                api_url: "http://localhost:11434".into(),
                embedding: EmbeddingConfig {
                    model: "nomic-embed-text".into(),
                    dimensions: 768,
                },
                llm: LlmConfig {
                    model: "qwen2.5-coder:7b".into(),
                    temperature: 0.7,
                    note: Some("Model must support tool calling".into()),
                },
            },
            comfy: ComfyConfig {
                api_url: "http://localhost:8188".into(),
                default_workflow: "discipline_character.json".into(),
                timeout_secs: 300,
            },
        }
    }
}
```

## Gating Strategy: Prompt-on-Demand

Services are **NOT** health-checked at app startup. Instead:

1. User clicks an AI-gated feature (e.g., "✨ Generate Image" on discipline detail page)
2. Frontend shows loading state
3. Backend checks if service is configured AND reachable (1-2 second timeout)
4. **If unavailable**: Show modal with setup instructions
5. **If available**: Execute feature

### Health Check Flow

```rust
// Tauri command
#[tauri::command]
async fn check_comfy_available() -> Result<ComfyStatus, String> {
    let config = load_global_config()?;

    // Quick health ping (2s timeout)
    match reqwest::Client::new()
        .get(format!("{}/system_stats", config.comfy.api_url))
        .timeout(Duration::from_secs(2))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            Ok(ComfyStatus { available: true, error: None })
        }
        Ok(_) => {
            Ok(ComfyStatus {
                available: false,
                error: Some("ComfyUI responded but with an error".into()),
            })
        }
        Err(e) => {
            Ok(ComfyStatus {
                available: false,
                error: Some(format!("Cannot reach ComfyUI: {}", e)),
            })
        }
    }
}
```

### Setup Modal (Frontend)

When service is unavailable, show dialog:

```tsx
<Dialog open={showComfySetup}>
  <DialogHeader>
    <DialogTitle>ComfyUI Not Available</DialogTitle>
  </DialogHeader>
  <DialogContent>
    <p>To generate discipline images, configure ComfyUI:</p>
    <ol>
      <li>Install ComfyUI: <code>git clone https://github.com/comfyanonymous/ComfyUI</code></li>
      <li>Start server: <code>python main.py --listen</code></li>
      <li>Verify it's running at: <code>{config.comfy.api_url}</code></li>
      <li>Place workflow JSON at: <code>~/.config/ralph/workflows/{config.comfy.default_workflow}</code></li>
    </ol>
    <Button onClick={openSettings}>Open Settings</Button>
  </DialogContent>
</Dialog>
```

## ComfyUI Integration

### Feature: Discipline Character Imagery

**Trigger:** Manual button on discipline detail page

**UI Flow:**
1. Discipline detail page shows placeholder image (see Placeholder Design below)
2. Button: "✨ Generate Character Art" (disabled if no image generated yet, changes to "🔄 Regenerate" after first generation)
3. On click → Health check → Generate → Display

**Workflow:**
- Ralph provides **positive** and **negative** prompts to existing ComfyUI workflow
- User creates the workflow JSON in ComfyUI (saved to `~/.config/ralph/workflows/discipline_character.json`)
- Workflow must accept `positive_prompt` and `negative_prompt` as inputs
- Output is PNG saved to `.ralph/generated/disciplines/{discipline_name}.png`

**Prompt Generation (Deterministic):**
```rust
fn generate_discipline_prompts(discipline: &Discipline) -> (String, String) {
    let positive = format!(
        "Professional character portrait, {} themed, \
         clean background, studio lighting, \
         high detail, concept art style, \
         color palette: {}",
        discipline.display_name,
        discipline.color
    );

    let negative = format!(
        "blurry, low quality, distorted, \
         multiple characters, cluttered background, \
         watermark, text"
    );

    (positive, negative)
}
```

### Placeholder Image Design

**Request for user:**
> **Placeholder Prompt**: Generate a 512x512 neutral placeholder image for discipline characters. Style: minimalist, geometric icon on gradient background (use CSS `background: linear-gradient(135deg, hsl(var(--muted)) 0%, hsl(var(--muted-foreground) / 0.1) 100%)`), centered icon (from lucide-react based on discipline), subtle shadow. Text below: "Click Generate to create character art". Design should work in both light and dark mode.

**Implementation:**
- SVG component: `src/components/DisciplinePlaceholder.tsx`
- Props: `icon: LucideIcon`, `label: string`
- Uses existing Tailwind design tokens

## Settings UI

**Location:** New "Settings" button in app header (gear icon)

**Modal Tabs:**
1. **General** (future: app preferences)
2. **External Services** ⭐ (this spec)
   - Ollama section (API URL, embedding model/dims, LLM model/temp)
   - ComfyUI section (API URL, workflow path, timeout)
   - Qdrant section (REST/gRPC URLs)
   - "Test Connection" buttons for each service
3. **Advanced** (future: debug settings)

**Form Controls:**
- Use `Input` component from shadcn
- Validate URLs (must start with http/https)
- Show connection status indicators (🟢 Connected, 🔴 Disconnected, ⚪ Not Tested)
- "Save" writes to `~/.config/ralph/external_services.json`

## Future Extensibility

### LLM Features (TBD)

Planned use cases (not implemented in this spec):
- **Task description enhancement**: User writes rough task → LLM expands to structured format
- **Feature suggestion**: Analyze codebase → suggest missing features
- **Learning summarization**: Compress verbose learnings into concise insights

All follow same gating pattern: prompt-on-demand, explicit user trigger, graceful degradation.

### RAG Integration

ralph-rag crate already exists. Future spec will cover:
- Semantic search for tasks/features/learnings
- Context injection for Claude CLI prompts
- Auto-accumulation of touched files

RAG uses same Ollama + Qdrant config defined here.

## Migration Path

**Phase 1 (this spec):**
- Create `ralph-external-services` crate
- Add config loading/saving
- Implement ComfyUI integration for discipline imagery
- Add Settings UI

**Phase 2 (future spec):**
- Wire up ralph-rag with config
- Add semantic search UI

**Phase 3 (future spec):**
- LLM features (task enhancement, etc.)

## Implementation Checklist

- [ ] Create `crates/ralph-external-services/` with config types
- [ ] Add config load/save commands (Tauri)
- [ ] Implement ComfyUI health check + generation
- [ ] Create `DisciplinePlaceholder.tsx` component
- [ ] Add "Generate" button to discipline detail page
- [ ] Build Settings modal with External Services tab
- [ ] Add gear icon to app header
- [ ] Store generated images in `.ralph/generated/disciplines/`
- [ ] Update `.gitignore` to exclude `.ralph/generated/`
- [ ] Write tests for config serialization
- [ ] Document ComfyUI workflow requirements in user guide

## Notes

- **Qdrant embedded**: Consider vendoring Qdrant binary (like Tauri embeds sidecars) to eliminate external dependency for RAG
- **Workflow distribution**: Ship default ComfyUI workflow JSON with Ralph (optional use)
- **Image caching**: Generated images persist across sessions (manual regenerate only)
- **Dark mode**: Ensure generated images look good on dark backgrounds (transparent or neutral bg)
