# SPEC-061: Remaining Bugs & Implementation TODO

**Created:** 2026-02-08
**Status:** TODO (work deferred due to multi-agent branch activity)
**Related:** `.specs/061_EXTERNAL_SERVICES_ARCHITECTURE.md`, `.specs/061_EXTERNAL_SERVICES_ARCHITECTURE_BUGS.md`

This doc tracks bugs and implementation tasks for the External Services Architecture (ComfyUI + Ollama LLM + Qdrant) that remain unfixed after initial audit.

## ✅ Completed

- [x] **Bug #1:** Added .gitignore entry for `.ralph/generated/` (commit d800168)
- [x] **CLAUDE.md fix:** Updated database architecture description from YAML to SQLite (commit 55559b5)

## 🔧 Straightforward Fixes (Ready to implement)

### Fix #2: Add HTTP Client Dependencies
**File:** `src-tauri/Cargo.toml`
**Change:**
```toml
reqwest = { version = "0.12", features = ["json"] }
# tokio already present with "full" features
```
**Why:** Health checks for external services need HTTP client

### Fix #3: Create Placeholder SVG Component
**File:** `src/components/DisciplinePlaceholder.tsx`
**What:** SVG component showing discipline icon + "Click Generate Character Art" text
**Design:**
- Gradient background using CSS design tokens
- Lucide icon (passed as prop)
- Centered layout, aspect-square
- Works in light + dark mode
**Code sketch:**
```tsx
export function DisciplinePlaceholder({ icon: Icon, label }: { icon: LucideIcon; label: string }) {
  return (
    <div className="aspect-square w-full max-w-sm mx-auto flex flex-col items-center justify-center bg-gradient-to-br from-muted to-muted/10 rounded-lg border-2 border-dashed border-muted-foreground/20">
      <Icon className="h-24 w-24 text-muted-foreground/40 mb-4" />
      <p className="text-sm text-muted-foreground text-center px-4">
        Click <strong>Generate Character Art</strong> below
      </p>
    </div>
  )
}
```

### Fix #4: Add Example Config JSON
**File:** `.specs/061_external_services.example.json`
**Content:** Full example config with comments explaining each field
**Purpose:** Users can copy this to `~/.config/ralph/services.json` and customize

### Fix #5: Security Validation Helpers
**File:** `src-tauri/src/commands/external_services.rs` (new file)
**Functions:**
```rust
fn validate_service_url(url: &str) -> Result<(), String> {
    // Must be http:// or https://
    // Must parse as valid URL (use reqwest::Url)
}

fn validate_workflow_path(path: &str) -> Result<PathBuf, String> {
    // Must be within ~/.config/ralph/workflows/
    // Prevent path traversal (../)
}
```

### Fix #6: TypeScript Types for New Commands
**File:** `src/types/generated.ts` or new `src/types/external-services.ts`
**Types needed:**
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

### Fix #7: Frontend State for Concurrent Generation Protection
**File:** `src/components/workspace/DisciplineDetailTabContent.tsx` (extend existing)
**Pattern:**
```tsx
const [generationState, setGenerationState] = useState<'idle' | 'generating' | 'success' | 'error'>('idle')

<Button
  disabled={generationState === 'generating'}
  onClick={handleGenerate}
>
  {generationState === 'generating' ? (
    <>
      <Loader2 className="mr-2 h-4 w-4 animate-spin" />
      Generating...
    </>
  ) : (
    'Generate Character Art'
  )}
</Button>
```

## 🏗️ Medium Complexity (Needs design + coding)

### Task #8: Config Loading with Validation
**Scope:** New module for loading/saving external services config
**Location:** `src-tauri/src/commands/external_services.rs` or new crate
**Features:**
- Load from `~/.config/ralph/services.json`
- Fall back to defaults if missing
- Validate all URLs (security)
- Validate workflow paths (security)
- Create config dir if needed
- Schema versioning for migrations

### Task #9: Health Check Implementation
**Scope:** Quick health pings to external services
**Endpoints:**
- Ollama: `GET /api/tags` (list models)
- ComfyUI: `GET /system_stats`
- Qdrant: `GET /` or `/collections`
**Timeout:** 2 seconds (fail fast)
**Return:** `ServiceStatus { available: bool, error?: string }`

### Task #10: Settings UI Extension
**File:** `src/components/app-shell/Settings.tsx` (extend existing)
**Add tabs:**
- General (existing: dark mode)
- **External Services** (new)
  - Auto-Detect button (tries common localhost ports)
  - Ollama section (URL, models, temp)
  - ComfyUI section (URL, workflow path [Browse button], timeout)
  - Qdrant section (REST/gRPC URLs)
  - Test Connection buttons (shows green/red status)
- Advanced (future)

### Task #11: Discipline Detail Image Section
**File:** `src/components/workspace/DisciplineDetailTabContent.tsx` (extend)
**Add before Properties card:**
```tsx
<Card>
  <CardHeader className="pb-3">
    <CardTitle className="text-sm">Character Art</CardTitle>
    <CardDescription className="text-xs">
      AI-generated imagery for this discipline
    </CardDescription>
  </CardHeader>
  <CardContent>
    {hasGeneratedImage ? (
      <img src={imageUrl} alt={discipline.displayName} />
    ) : (
      <DisciplinePlaceholder icon={Icon} label={discipline.displayName} />
    )}
    <Button
      onClick={handleGenerate}
      disabled={generationState === 'generating'}
      className="mt-4 w-full"
    >
      {/* ... */}
    </Button>
  </CardContent>
</Card>
```

### Task #12: Config Dir Creation Helper
**File:** `src-tauri/src/commands/external_services.rs`
**Function:**
```rust
fn ensure_config_dir() -> Result<PathBuf, String> {
    let config_dir = dirs::config_dir()
        .ok_or("No config dir on this platform")?
        .join("ralph");

    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config dir: {}", e))?;

    Ok(config_dir)
}
```

## 🤔 Need Design Decisions

### Decision #13: Image Storage Location
**Options:**
- **A) `.ralph/generated/disciplines/{name}.png`** (in project)
  - Pro: Easy to share, version control optional
  - Con: Pollutes project git, need to document .gitignore
- **B) `~/.cache/ralph/projects/{hash}/disciplines/{name}.png`** (global cache)
  - Pro: Clean separation, no git pollution
  - Con: Harder to share, cache cleanup needed
- **C) Hybrid:** Store in cache, symlink in project
  - Pro: Best of both
  - Con: Complexity, Windows symlink issues

**Recommendation:** Option B (global cache) for cleanliness

### Decision #14: Config Architecture
**Options:**
- **A) Extend ralph-rag crate** with `comfy` module
  - Pro: One config, one crate
  - Con: ralph-rag becomes "external services" crate (rename later?)
- **B) New `ralph-external-services` crate**
  - Pro: Clean separation
  - Con: Config duplication with ralph-rag
- **C) Consolidate into single `ralph-external` crate** (merge ralph-rag into it)
  - Pro: Future-proof, clean API
  - Con: Big refactor now

**Recommendation:** Option A short-term (extend ralph-rag), Option C long-term (rename crate)

### Decision #15: Where Does ComfyUI Code Live?
**Options:**
- A) In ralph-rag (as `comfy` module)
- B) In src-tauri/src/commands/comfy.rs (no crate)
- C) New thin crate `ralph-comfy`

**Recommendation:** Option B (keep it simple, no new crate for just ComfyUI API calls)

## 📝 Documentation Tasks

### Doc #16: ComfyUI Workflow Guide
**File:** `.docs/038_COMFYUI_WORKFLOW_SETUP.md`
**Content:**
- How to create workflow in ComfyUI web UI
- Required nodes: CLIPTextEncode x2, KSampler, VAEDecode, SaveImage
- Naming convention for text inputs: `positive_prompt`, `negative_prompt`
- Export as JSON
- Place in `~/.config/ralph/workflows/discipline_character.json`
- Screenshots/diagrams

### Doc #17: External Services User Guide
**File:** `.docs/039_EXTERNAL_SERVICES_USER_GUIDE.md`
**Content:**
- What are external services?
- Installation instructions (Ollama, ComfyUI, Qdrant)
- Configuration via Settings UI
- Troubleshooting common issues
- What happens if services are unavailable?

## ⚠️ Known Issues (Lower Priority)

### Issue #18: No Progress Indicator for Long Operations
**Problem:** ComfyUI generation takes 30-300 seconds, no feedback
**Solutions:**
- A) Poll ComfyUI queue API every 5s, show progress %
- B) Indeterminate progress bar + "This may take 1-5 minutes" message
- C) Stream events from backend (complex)
**Recommendation:** Option B (simple, good UX)

### Issue #19: Image Format Assumption
**Problem:** Spec assumes PNG but ComfyUI can output JPG, WEBP, etc.
**Solution:** Detect from Content-Type header or make configurable
**Priority:** Low (most workflows output PNG by default)

### Issue #20: No Disk Space Check
**Problem:** Generated images are 1-10MB, no check before generation
**Solution:** Add `fs4::available_space()` check before generating
**Priority:** Medium (rare but serious)

### Issue #21: No Workflow Schema Validation
**Problem:** User's workflow JSON might not have required inputs
**Solution:** Parse workflow on Settings save, check for `positive_prompt` and `negative_prompt` fields
**Priority:** Medium (prevents cryptic runtime errors)

### Issue #22: No Service Auto-Discovery
**Problem:** User must manually type URLs (http://localhost:11434, etc.)
**Solution:** On Settings open, try connecting to common ports, show "Use Detected" buttons
**Priority:** Low (nice-to-have UX improvement)

### Issue #23: Color-to-Word Mapping Unclear
**Problem:** How does `#6366f1` become a prompt word? "indigo"? "blue-purple"?
**Solution:**
- Use color-name crate to map hex → human name
- OR: Just use hex directly in prompt
- OR: Add color name field to discipline config
**Priority:** Low (prompts work without it, just less precise)

### Issue #24: No Config Migration Strategy
**Problem:** If config schema changes (version 2), how to migrate?
**Solution:** Add migration logic in config loader (match on version field)
**Priority:** Low (v1 is first version, no migrations yet)

### Issue #25: Concurrent Image Generation Race Condition
**Problem:** User clicks Generate, then clicks again before first completes
**Solution:** Frontend button is disabled during generation (Fix #7 addresses this)
**Priority:** Fixed by #7

### Issue #26: Generated Image Caching/Invalidation
**Problem:** If discipline color changes, is old image stale?
**Solution:**
- Store metadata JSON alongside image (workflow, prompt hash, timestamp)
- Show "Generated on {date} using {workflow}" in UI
- Add "Regenerate" button to replace
**Priority:** Medium (good UX, prevents confusion)

### Issue #27: No Workflow File Not Found Handling
**Problem:** User sets workflow path but file doesn't exist → cryptic error
**Solution:** Validate file exists on Settings save, show friendly error with instructions
**Priority:** Medium (common first-time user issue)

## 🧪 Testing Gaps

### Test #28: Integration Tests with Mock ComfyUI
**Need:** Use wiremock to mock ComfyUI server for tests
**Scope:**
- Health check succeeds/fails
- Generation request/response
- Timeout handling
- Error handling

### Test #29: Config Validation Tests
**Need:** Unit tests for URL validation, path traversal prevention
**Scope:**
- Valid URLs pass
- `file://`, `javascript:` rejected
- `../../etc/passwd` rejected
- Workflow path must be in workflows dir

### Test #30: Frontend State Machine Tests
**Need:** Vitest tests for generation button state machine
**Scope:**
- Button disabled during generation
- Button re-enabled on success/error
- Error message shown on failure

## Implementation Order Recommendation

**Phase 0 (done):**
- [x] CLAUDE.md fix
- [x] .gitignore fix

**Phase 1 (straightforward, no dependencies):**
1. Fix #2 (add reqwest)
2. Fix #3 (placeholder component)
3. Fix #4 (example config)
4. Fix #5 (validation helpers)
5. Fix #6 (TypeScript types)

**Phase 2 (core functionality):**
6. Decision #14 (config architecture) + Task #8 (config loading)
7. Decision #15 (where comfy code lives)
8. Task #9 (health checks)
9. Task #10 (Settings UI)
10. Task #11 (discipline image section)
11. Fix #7 (concurrent protection)

**Phase 3 (polish):**
12. Decision #13 (image storage location)
13. Doc #16 (workflow guide)
14. Doc #17 (user guide)
15. Issue #26 (metadata/caching)

**Phase 4 (nice-to-haves):**
16. Issue #18 (progress indicator)
17. Issue #22 (auto-discovery)
18. Test #28-30 (integration tests)

## Notes

- Work paused due to multiple agents on branch simultaneously
- SPEC-061 main document has full architectural details
- SPEC-061 bugs document has complete bug analysis (30 total)
- This doc is the actionable subset for implementation
