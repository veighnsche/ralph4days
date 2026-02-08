# SPEC-061: Bugs and Gaps Report

**Generated:** 2026-02-08
**Reviewed spec:** 061_EXTERNAL_SERVICES_ARCHITECTURE.md

## Critical Issues

### 1. ❌ CLAUDE.md Database Architecture is Outdated
**Location:** CLAUDE.md line ~200
**Issue:** CLAUDE.md describes "Multi-file YAML database in `.ralph/db/`" but actual implementation is **SQLite at `.ralph/db/ralph.db`**
**Impact:** Spec assumes wrong data model. Any implementation following CLAUDE.md will conflict with existing code.
**Fix:** Update CLAUDE.md to reflect SQLite reality or clarify that it's aspirational (but sqlite-db crate is well-established, so likely CLAUDE.md is just stale).

### 2. 🔄 Config Duplication with ralph-rag
**Location:** ralph-rag/src/config.rs:28-88
**Issue:**
- ralph-rag already has `RagConfig` with `qdrant_url`, `qdrant_grpc_url`, `ollama_url`, `embedding_model`, `embedding_dims`
- Spec proposes `ExternalServicesConfig` with overlapping fields
- Two separate config files = drift and user confusion
**Impact:** Users would need to configure same services in two places
**Fix:**
- **Option A (Recommended):** Extend ralph-rag's `RagConfig` to include LLM and ComfyUI sections
- **Option B:** Consolidate into single `~/.config/ralph/services.json` that includes all external services
- **Option C:** Make spec's config a superset that loads ralph-rag config as a subset

### 3. 🏗️ Settings UI Already Exists (Simple)
**Location:** src/components/app-shell/Settings.tsx
**Issue:** Spec says "Add Settings button" but Settings component already exists (just has dark mode toggle)
**Impact:** Minor - just need to extend existing component, not create new one
**Fix:** Update spec to say "Extend Settings component with External Services tab"

### 4. 📦 Missing HTTP Client Dependency
**Location:** src-tauri/Cargo.toml
**Issue:** Spec proposes health checks using `reqwest` but it's not in dependencies
**Impact:** Won't compile
**Fix:** Add to Cargo.toml:
```toml
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1", features = ["time"] }
```

### 5. 🖼️ No Image Storage in .gitignore
**Location:** .gitignore
**Issue:** Spec proposes storing generated images in `.ralph/generated/` but this path isn't gitignored (either for project root or in target projects)
**Impact:** Generated images would be committed to git (large files, not source code)
**Fix:** Add to .gitignore:
```
# Generated discipline images
.ralph/generated/
**/ralph/generated/
```

## Architecture Issues

### 6. 📁 Unclear Crate Organization
**Issue:** Spec proposes new `ralph-external-services` crate but:
- ralph-rag already handles Qdrant + Ollama
- ComfyUI is the only truly "new" service
**Impact:** Over-engineering, extra compilation unit
**Fix:**
- **Option A:** Add comfy module to ralph-rag (rename crate to `ralph-external` later if needed)
- **Option B:** Create thin `ralph-comfy` crate just for ComfyUI, keep using ralph-rag config for the rest

### 7. 🔐 Security: No URL Validation
**Location:** Spec's config schema
**Issue:** User-provided URLs (`api_url` fields) have no validation
**Attack vectors:**
- `file:///etc/passwd` - local file access
- `javascript:alert(1)` - XSS if rendered in webview
- Malformed URLs causing panics
**Fix:** Add validation in Rust:
```rust
fn validate_service_url(url: &str) -> Result<(), String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("URL must use http:// or https://".into());
    }
    reqwest::Url::parse(url)
        .map_err(|e| format!("Invalid URL: {}", e))?;
    Ok(())
}
```

### 8. 🛡️ Security: Path Traversal in Workflow Paths
**Location:** Spec's `ComfyConfig.default_workflow`
**Issue:** User can specify `../../.ssh/id_rsa` as workflow path
**Impact:** Arbitrary file read if workflow path isn't sanitized
**Fix:**
```rust
fn validate_workflow_path(path: &str) -> Result<PathBuf, String> {
    let workflows_dir = dirs::config_dir()
        .ok_or("No config dir")?
        .join("ralph")
        .join("workflows");

    let full_path = workflows_dir.join(path);

    // Prevent path traversal
    if !full_path.starts_with(&workflows_dir) {
        return Err("Workflow path escapes workflows directory".into());
    }

    Ok(full_path)
}
```

### 9. 📂 No Config Directory Creation
**Issue:** Spec assumes `~/.config/ralph/` exists but never creates it
**Impact:** First-run failure when trying to save config
**Fix:** Add initialization:
```rust
fn ensure_config_dir() -> Result<PathBuf, String> {
    let config_dir = dirs::config_dir()
        .ok_or("No config directory on this platform")?
        .join("ralph");

    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config dir: {}", e))?;

    Ok(config_dir)
}
```

### 10. 🗂️ Project-level Image Storage Needs .gitignore Handling
**Issue:** Spec says images go in `.ralph/generated/disciplines/` within **target projects** but:
- Ralph doesn't control target project's .gitignore
- Generated images would pollute project git history
**Fix Options:**
- **A)** Document this as user responsibility (add to README/docs)
- **B)** Ralph auto-appends to project's .gitignore on first image generation
- **C)** Store images in `~/.cache/ralph/projects/{project_hash}/disciplines/` instead (cleaner but harder to share)

## Missing Error Handling

### 11. ⚠️ No ComfyUI Workflow Schema Validation
**Issue:** Spec assumes workflow JSON accepts `positive_prompt` and `negative_prompt` but doesn't validate
**Impact:** Silent failures or cryptic ComfyUI errors
**Fix:**
- Provide reference workflow JSON with spec
- Validate workflow on load:
```rust
fn validate_comfy_workflow(workflow: &serde_json::Value) -> Result<(), String> {
    // Check that workflow has expected input fields
    let inputs = workflow.get("inputs")
        .ok_or("Workflow missing 'inputs' field")?;

    if !inputs.get("positive_prompt").is_some() {
        return Err("Workflow missing 'positive_prompt' input".into());
    }
    if !inputs.get("negative_prompt").is_some() {
        return Err("Workflow missing 'negative_prompt' input".into());
    }

    Ok(())
}
```

### 12. 🔄 No Concurrent Generation Protection
**Issue:** User clicks "Generate" multiple times rapidly
**Impact:**
- Multiple ComfyUI jobs queued
- Wasted compute
- Race condition on file write
**Fix:** Add state machine in frontend:
```tsx
const [generationState, setGenerationState] = useState<'idle' | 'generating' | 'success' | 'error'>('idle')

// Disable button when generating
<Button
  disabled={generationState === 'generating'}
  onClick={handleGenerate}
>
  {generationState === 'generating' ? 'Generating...' : 'Generate'}
</Button>
```

### 13. 💾 No Disk Space Check
**Issue:** Generated images can be 1-10MB each, no check before generation
**Impact:** Fills disk, crashes app
**Fix:** Check available space:
```rust
use fs4::available_space;

fn check_disk_space(path: &Path) -> Result<(), String> {
    let available = available_space(path)
        .map_err(|e| format!("Cannot check disk space: {}", e))?;

    const MIN_REQUIRED: u64 = 50 * 1024 * 1024; // 50MB
    if available < MIN_REQUIRED {
        return Err(format!(
            "Insufficient disk space. Need {}MB, have {}MB",
            MIN_REQUIRED / 1024 / 1024,
            available / 1024 / 1024
        ));
    }

    Ok(())
}
```

### 14. 📊 No Progress Indicator for Long Operations
**Issue:** ComfyUI generation takes 30-300 seconds, no feedback to user
**Impact:** User thinks app is frozen, clicks away or force-quits
**Fix:**
- **Option A:** Stream progress events from Tauri backend (if ComfyUI API supports it)
- **Option B:** Show indeterminate progress bar with "This may take 1-5 minutes..." message
- **Option C:** Poll ComfyUI queue status API every 5 seconds

### 15. 🖼️ Image Format Assumption
**Issue:** Spec assumes PNG output but ComfyUI workflows can output PNG, JPG, WEBP, etc.
**Impact:** Wrong file extension, broken image display
**Fix:**
- Detect format from ComfyUI response headers
- OR: Force PNG in workflow JSON
- OR: Make format configurable in ComfyConfig

### 16. 🔧 Missing TypeScript Types
**Issue:** New Tauri commands need TypeScript bindings
**Impact:** Frontend can't call commands without type errors
**Fix:** Add to ts-rs generation or create manual types:
```typescript
export type ExternalServicesConfig = {
  version: number
  qdrant: { rest_url: string; grpc_url: string }
  ollama: {
    api_url: string
    embedding: { model: string; dimensions: number }
    llm: { model: string; temperature: number }
  }
  comfy: {
    api_url: string
    default_workflow: string
    timeout_secs: number
  }
}

export type ServiceStatus = {
  available: boolean
  error?: string
}
```

## UX Issues

### 17. 📝 No Placeholder Image Provided
**Issue:** Spec asks user to provide placeholder image design but doesn't include one
**Impact:** Blocks implementation
**Fix:** Create simple SVG component:
```tsx
export function DisciplinePlaceholder({
  icon: Icon,
  label
}: {
  icon: LucideIcon
  label: string
}) {
  return (
    <div className="aspect-square w-full max-w-sm mx-auto flex flex-col items-center justify-center bg-gradient-to-br from-muted to-muted/10 rounded-lg border-2 border-dashed border-muted-foreground/20">
      <Icon className="h-24 w-24 text-muted-foreground/40 mb-4" />
      <p className="text-sm text-muted-foreground text-center px-4">
        Click <strong>Generate Character Art</strong> below to create custom imagery
      </p>
    </div>
  )
}
```

### 18. 📍 Button Placement Unclear
**Issue:** Spec says "add button to discipline detail page" but doesn't specify where
**Current structure:** DisciplineDetailTabContent has:
- Header (name, icon, Edit button)
- ScrollArea with Cards (Properties, System Prompt, Skills, Conventions, MCP Servers)
**Recommendation:** Add new "Character Art" card at top of ScrollArea, before Properties

### 19. ❌ No Workflow Not Found Handling
**Issue:** Spec references workflow JSON but doesn't handle file missing
**Impact:** Cryptic error on first use
**Fix:**
- Check if workflow exists on Settings save
- Show helpful error: "Workflow file not found at {path}. Create it in ComfyUI and export as JSON."
- Provide "Browse" button to select workflow file

### 20. 🔍 No Service Discovery
**Issue:** User must manually enter URLs (http://localhost:11434, etc.)
**UX improvement:** Auto-detect services:
```rust
async fn discover_services() -> DiscoveredServices {
    let ollama = try_connect("http://localhost:11434").await;
    let comfy = try_connect("http://localhost:8188").await;
    let qdrant = try_connect("http://localhost:6333").await;

    DiscoveredServices { ollama, comfy, qdrant }
}
```
Show discovered services with "Use Detected" buttons in Settings UI.

## Spec Inconsistencies

### 21. 📏 Prompt Generation Not Deterministic
**Issue:** Spec says "Deterministic prompt generation" but example uses:
```rust
format!("Professional character portrait, {} themed, ...", discipline.display_name)
```
This IS deterministic, but spec should clarify "template-based, not LLM-generated"

### 22. 🎨 Color Palette Usage Unclear
**Issue:** Spec mentions `color: {}` in prompt but doesn't explain how HSL/hex color translates to natural language
**Example:** `#6366f1` → what string goes in prompt? "indigo"? "blue-purple"? Hex value?
**Fix:** Add color-to-word mapping:
```rust
fn color_to_description(hex: &str) -> &str {
    // Simple hue-based mapping
    // Better: use color-name library
    match hex {
        x if x.starts_with("#63") => "indigo",
        x if x.starts_with("#ef") => "red",
        // etc.
        _ => "vibrant"
    }
}
```

### 23. 🔄 Migration Path Missing
**Issue:** Spec says "Phase 1, 2, 3" but doesn't explain:
- What if user already has ralph-rag config somewhere?
- What if future phases change config schema?
**Fix:** Add schema versioning:
```rust
pub struct ExternalServicesConfig {
    pub version: u32, // Start at 1, increment on breaking changes
    // ... rest of fields
}

fn migrate_config(old: ExternalServicesConfig) -> ExternalServicesConfig {
    match old.version {
        1 => old, // Current version
        0 => /* migrate from unversioned */,
        _ => panic!("Unknown config version"),
    }
}
```

## Documentation Gaps

### 24. 📚 No User Guide for Workflows
**Issue:** Spec assumes user knows how to create ComfyUI workflows
**Fix:** Add to user docs:
1. Open ComfyUI web UI
2. Create workflow with these nodes:
   - CLIPTextEncode (for positive prompt)
   - CLIPTextEncode (for negative prompt)
   - KSampler
   - VAEDecode
   - SaveImage
3. Name text input widgets "positive_prompt" and "negative_prompt"
4. Export workflow as JSON (right-click → Save Workflow)
5. Place in `~/.config/ralph/workflows/discipline_character.json`

### 25. 📖 No ComfyUI Installation Instructions
**Issue:** Setup modal shows commands but doesn't link to full guide
**Fix:** Add link to ComfyUI docs or include quick start in Ralph docs

### 26. ⚙️ No Example Config File
**Issue:** Users must construct JSON from spec's schema
**Fix:** Include `external_services.example.json`:
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
      "temperature": 0.7
    }
  },
  "comfy": {
    "api_url": "http://localhost:8188",
    "default_workflow": "discipline_character.json",
    "timeout_secs": 300
  }
}
```

## Testing Gaps

### 27. 🧪 No Test Plan
**Issue:** Spec has implementation checklist but no test scenarios
**Fix:** Add test cases:
- [ ] Config loads defaults when file missing
- [ ] Config validates URLs on load
- [ ] Health check times out after 2 seconds
- [ ] Generate button disabled during generation
- [ ] Error modal shows on service unavailable
- [ ] Generated image displays correctly
- [ ] Clicking regenerate overwrites existing image
- [ ] Path traversal blocked in workflow paths
- [ ] Settings UI persists changes
- [ ] Dark mode doesn't break placeholder image

### 28. 🔬 No Integration Test Strategy
**Issue:** How to test ComfyUI integration without running actual ComfyUI?
**Fix:** Mock ComfyUI server:
```rust
#[cfg(test)]
mod tests {
    use wiremock::{MockServer, Mock, ResponseTemplate};

    #[tokio::test]
    async fn test_comfy_health_check() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/system_stats"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let status = check_comfy_health(&mock_server.uri()).await.unwrap();
        assert!(status.available);
    }
}
```

## Performance Issues

### 29. ⏱️ No Timeout Configuration
**Issue:** Spec hardcodes 300s timeout but some workflows may need longer
**Impact:** Complex workflows fail, no way to extend timeout
**Fix:** Make timeout configurable per discipline or globally adjustable in UI

### 30. 💾 No Image Caching Strategy
**Issue:** Spec says "images persist across sessions" but:
- What if discipline color/name changes? Stale image?
- What if user wants to regenerate with different workflow?
**Fix:** Add metadata:
```rust
struct GeneratedImageMeta {
    discipline_name: String,
    generated_at: String,
    workflow_name: String,
    prompt_hash: String, // SHA256 of prompts used
}
```
Show "Generated on {date} using {workflow}" in UI

## Revised Architecture Recommendation

Based on these findings, I recommend:

1. **Extend ralph-rag instead of new crate:**
   - Add `llm` and `comfy` modules to ralph-rag
   - Rename crate to `ralph-external` in future if needed
   - Single config at `~/.config/ralph/services.json`

2. **Image storage location:**
   - Store in `~/.cache/ralph/projects/{project_hash}/disciplines/{name}.png`
   - Avoids polluting project git
   - OS-managed cache cleanup

3. **Config priority:**
   - Per-project: `.ralph/services.json` (if exists)
   - Global: `~/.config/ralph/services.json`
   - Built-in defaults

4. **Settings UI structure:**
   ```
   Settings Dialog
   ├─ General (theme, etc.)
   ├─ External Services
   │  ├─ Auto-Detect Services [Button]
   │  ├─ Ollama (API URL, embedding model/dims, LLM model/temp)
   │  ├─ ComfyUI (API URL, workflow [Browse], timeout)
   │  └─ Qdrant (REST URL, gRPC URL)
   └─ Advanced (future)
   ```

5. **Security hardening:**
   - Validate all URLs (must be http/https)
   - Validate workflow paths (no traversal)
   - Timeout all HTTP requests (2s health check, 300s+ generation)
   - Sanitize prompts before sending to ComfyUI

## Next Steps

1. ✅ Review this bug report with stakeholder
2. 🔧 Fix critical issues (config duplication, security)
3. 📝 Update spec with corrections
4. 🚀 Implement Phase 1 with hardened design
