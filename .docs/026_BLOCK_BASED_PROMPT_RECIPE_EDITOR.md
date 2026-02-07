# 026 — Block-Based Prompt Recipe Editor

**Date:** 2026-02-07
**Status:** Implemented, compiles clean, needs manual QA
**Next team focus:** Performance measures

---

## What Was Built

The naive text-editor prompt builder modal was replaced with a proper **block-based recipe composition tool**. Users compose which prompt sections go into a recipe, in what order, with optional per-section instruction overrides. Custom recipes are saved as JSON in `.ralph/prompts/`.

This was a backend+frontend change across 9 files modified and 2 new files created.

---

## Architecture Overview

```
PromptBuilderModal (React)
  ├── Recipe Picker (Select: built-in + saved custom recipes)
  ├── Left Panel: Sortable section blocks (dnd-kit)
  │   ├── Each block: drag handle | toggle | category badge | name | description
  │   └── Instruction blocks: collapsible textarea for per-section overrides
  ├── Right Panel: Live debounced preview (500ms)
  └── Footer: section count | Delete | Copy | Save/Save As

    ↕ IPC (Tauri invoke)

Backend Commands (Rust):
  get_section_metadata()     → 22 SectionInfo entries
  get_recipe_sections(type)  → SectionConfig[] (enabled in-recipe, disabled otherwise)
  preview_custom_recipe()    → builds prompt from custom section list
  list/load/save/delete_saved_recipes()  → CRUD for .ralph/prompts/*.json
```

---

## Backend Changes

### 1. Per-Section Instruction Overrides

**Before:** `PromptContext.instruction_override: Option<String>` — one override for the entire prompt.
**After:** `PromptContext.instruction_overrides: HashMap<String, String>` — keyed by section name.

**Files changed:**
- `crates/prompt-builder/src/context.rs` — Field type change + `test_context()` updated
- All 6 instruction sections (`braindump.rs`, `yap.rs`, `ramble.rs`, `discuss.rs`, `task_exec.rs`, `opus_review.rs`) — Each now calls `ctx.instruction_overrides.get("braindump_instructions")` etc. instead of `ctx.instruction_override`
- `src-tauri/src/commands.rs` — `build_prompt_context()` signature changed from `Option<String>` to `HashMap<String, String>`. All call sites updated (`preview_prompt`, `generate_mcp_config`).

**How instruction override resolution works per section:**
```rust
fn build(ctx: &PromptContext) -> Option<String> {
    if let Some(text) = ctx.instruction_overrides.get("braindump_instructions") {
        return Some(text.clone());
    }
    Some(default_text())  // static default
}
```

### 2. Section Metadata Registry

**New file:** `crates/prompt-builder/src/sections/metadata.rs`

Provides display metadata for all 22 sections so the frontend knows what to render:

```rust
pub struct SectionInfo {
    pub name: &'static str,        // "project_context"
    pub display_name: &'static str, // "Project Context"
    pub description: &'static str,  // "CLAUDE.RALPH.md and project-level context"
    pub category: &'static str,     // "project"
    pub is_instruction: bool,       // false
}
```

**22 sections across 7 categories:**

| Category | Sections | Count |
|----------|----------|-------|
| project | project_context, project_metadata, codebase_state | 3 |
| feature | feature_listing, feature_context, feature_files, feature_state | 4 |
| task | task_listing, task_details, task_files, dependency_context, previous_attempts | 5 |
| discipline | discipline_listing, discipline_persona | 2 |
| state | state_files | 1 |
| user | user_input | 1 |
| instructions | braindump/yap/ramble/discuss/task_exec/opus_review_instructions | 6 |

**Functions:** `all_sections() -> Vec<SectionInfo>`, `get_info(name) -> Option<SectionInfo>`

Registered in `sections/mod.rs` as `pub mod metadata;`. Re-exported from `lib.rs` as `pub use sections::metadata::SectionInfo`.

### 3. Dynamic Recipe Building

**`recipe.rs` — new function:**
```rust
pub fn build_sections_from_names(section_names: &[&str], ctx: &PromptContext) -> Vec<PromptSection>
```
Iterates names, looks up each via `sections::get_section(name)`, calls `build(ctx)`, collects non-None results. This is how the custom recipe preview works — arbitrary section lists instead of hardcoded recipes.

**`lib.rs` — two new public functions:**
```rust
pub fn build_custom_sections(section_names: &[&str], ctx: &PromptContext) -> Vec<PromptSection>
pub fn get_recipe_section_names(prompt_type: PromptType) -> Vec<&'static str>
```
`get_recipe_section_names` returns the ordered section names for a built-in recipe so the frontend can populate the block list.

### 4. New Tauri Commands

**Types:**
```rust
pub struct SectionConfig {
    pub name: String,
    pub enabled: bool,
    pub instruction_override: Option<String>,
}

pub struct CustomRecipe {
    pub name: String,
    pub base_recipe: Option<String>,
    pub sections: Vec<SectionConfig>,
}
```

**7 new commands registered in `lib.rs`:**

| Command | Args | Returns | Notes |
|---------|------|---------|-------|
| `get_section_metadata` | none | `Vec<SectionInfo>` | Static registry, no state needed |
| `get_recipe_sections` | `prompt_type: String` | `Vec<SectionConfig>` | Returns ALL 22 sections: in-recipe ones as `enabled: true`, rest as `enabled: false` |
| `preview_custom_recipe` | `sections: Vec<SectionConfig>`, `user_input: Option<String>` | `PromptPreview` | Builds only enabled sections in given order |
| `list_saved_recipes` | none | `Vec<String>` | Reads `.ralph/prompts/*.json` filenames |
| `load_saved_recipe` | `name: String` | `CustomRecipe` | Reads + deserializes JSON |
| `save_recipe` | `recipe: CustomRecipe` | `()` | Writes pretty JSON to `.ralph/prompts/{name}.json` |
| `delete_recipe` | `name: String` | `()` | Deletes file, no-op if missing |

**Old commands preserved:** `preview_prompt`, `get_default_instructions`, `save_prompt_instructions`, `load_prompt_instructions`, `reset_prompt_instructions` — all still work for backward compat. `preview_prompt` internally converts its `instruction_override: Option<String>` to the new HashMap format.

### 5. Custom Recipe Storage Format

Saved at `.ralph/prompts/{recipe-name}.json`:
```json
{
  "name": "my-braindump-lite",
  "baseRecipe": "braindump",
  "sections": [
    { "name": "project_context", "enabled": true, "instructionOverride": null },
    { "name": "project_metadata", "enabled": true, "instructionOverride": null },
    { "name": "codebase_state", "enabled": false, "instructionOverride": null },
    { "name": "feature_listing", "enabled": true, "instructionOverride": null },
    { "name": "braindump_instructions", "enabled": true, "instructionOverride": "## Custom Instructions\n\nDo XYZ..." },
    ...
  ]
}
```

---

## Frontend Changes

### Complete Rewrite: `src/components/PromptBuilderModal.tsx`

**Dependencies added:** `@dnd-kit/core@6.3.1`, `@dnd-kit/sortable@10.0.0`, `@dnd-kit/utilities@3.2.2`

**Component structure:**
- `PromptBuilderModal` — main dialog with state, DnD context, recipe picker, preview, footer
- `SortableSectionBlock` — individual block using `useSortable` hook

**UI components used:** Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter, ResizablePanelGroup, ResizablePanel, ResizableHandle, ScrollArea, Select, SelectTrigger, SelectContent, SelectItem, Badge, Button, Switch, Collapsible, CollapsibleTrigger, CollapsibleContent, Textarea, Input, Separator

**State (all local `useState`, no Zustand):**
```typescript
baseRecipe: string                    // "braindump", "yap", etc.
recipeName: string | null             // null = unsaved, set when loading custom recipe
sections: SectionBlock[]              // ordered list of all 22 sections
preview: PromptPreview | null         // debounced preview result
userInput: string                     // simulated user input for preview
customRecipeNames: string[]           // names of saved custom recipes
sectionMeta: SectionMeta[]            // 22 metadata entries from backend
saveDialogOpen: boolean               // nested save-as dialog
saveNameInput: string                 // name input for save-as
```

**DnD implementation:**
- `DndContext` wraps the section list with `closestCenter` collision detection
- `SortableContext` with `verticalListSortingStrategy` and section names as IDs
- `PointerSensor` with 5px distance activation (prevents accidental drags on switch/button clicks)
- `KeyboardSensor` with `sortableKeyboardCoordinates` for accessibility
- Each block has a `GripVertical` drag handle via `setActivatorNodeRef` (only the handle initiates drag)
- Dragging item gets `z-50 shadow-md bg-background` for visual lift
- `handleDragEnd` does array splice reorder

**Category badge colors:**
```
project:      blue-500/15
feature:      violet-500/15
task:         amber-500/15
discipline:   emerald-500/15
state:        slate-500/15
user:         rose-500/15
instructions: orange-500/15
```

**Data flow:**
1. Modal opens → fetch `get_section_metadata()` + `list_saved_recipes()`
2. Metadata loads → fetch `get_recipe_sections("braindump")` → populate blocks
3. User toggles/reorders/edits → triggers 500ms debounced `preview_custom_recipe()`
4. Recipe picker change → `get_recipe_sections()` for built-in, `load_saved_recipe()` for custom
5. Save → `save_recipe()` with current section state
6. Delete → `delete_recipe()` + reload base recipe

**How SectionMeta maps to SectionBlock:**
```typescript
// Wire format from Rust (snake_case via serde)
interface SectionMeta {
  name: string;
  display_name: string;      // snake_case from Rust
  description: string;
  category: string;
  is_instruction: boolean;
}

// Frontend model (camelCase)
interface SectionBlock {
  name: string;
  displayName: string;       // mapped from display_name
  description: string;
  category: string;
  isInstruction: boolean;    // mapped from is_instruction
  enabled: boolean;          // from SectionConfig
  instructionOverride: string | null;  // from SectionConfig
}
```

Note: `SectionInfo` is serialized with default serde (snake_case fields), while `SectionConfig` and `CustomRecipe` use `#[serde(rename_all = "camelCase")]`. The frontend `SectionMeta` interface uses snake_case to match the Rust `SectionInfo` serialization. This is a deliberate asymmetry — `SectionInfo` was added to `prompt-builder` (which doesn't use `rename_all`) while `SectionConfig` lives in `commands.rs` (which does).

---

## Existing Section Registration System

The `sections::get_section(name)` match block in `sections/mod.rs` is the central dispatch for all 22 sections. It maps string names to `Section { name, build }` structs. This is what `build_sections_from_names` uses internally.

**6 built-in recipes and their section compositions:**

| Recipe | Sections (in order) |
|--------|-------------------|
| braindump | project_context, project_metadata, codebase_state, feature_listing, discipline_listing, user_input, braindump_instructions |
| yap | project_context, project_metadata, feature_listing, task_listing, discipline_listing, user_input, yap_instructions |
| ramble | project_context, project_metadata, feature_listing, feature_state, user_input, ramble_instructions |
| discuss | project_context, project_metadata, discipline_listing, user_input, discuss_instructions |
| task_execution | project_context, discipline_persona, feature_context, feature_files, feature_state, state_files, previous_attempts, dependency_context, task_details, task_files, task_exec_instructions |
| opus_review | project_context, feature_context, feature_files, feature_state, task_listing, state_files, opus_review_instructions |

Section order matters — recency bias means most important context should be last (closer to instructions).

---

## Performance Concerns for Next Team

### 1. Preview Debounce
Currently 500ms debounce on every state change (toggle, reorder, instruction edit). Each preview call:
- Opens a new SQLite connection (`SqliteDb::open`)
- Queries features, tasks, disciplines, project metadata
- Reads codebase snapshot from mutex
- Builds all enabled sections
- Returns full prompt text

**Opportunity:** The SQLite connection could be reused from `AppState.db` instead of opening a fresh one each time in `build_prompt_context`. Currently it opens a new connection because `build_prompt_context` takes `&std::path::Path` and constructs a new `SqliteDb`. The app already has `AppState.db: Mutex<Option<SqliteDb>>` with the connection open.

### 2. File Inlining
`feature_files` and `task_files` sections inline file contents from disk. For task_execution recipes with many context files, this can produce 50-100KB+ prompts. The preview renders this in a `<pre>` tag. Large previews may cause scroll/render jank.

**Opportunity:** Truncate or summarize file contents in preview mode. Or lazy-render section cards (only render visible ones in ScrollArea viewport).

### 3. Section Metadata is Static
`get_section_metadata()` returns the same 22 entries every time. Currently fetched on every modal open. Could be fetched once and cached in a React context or module-level variable.

### 4. DnD in ScrollArea
The dnd-kit sortable list lives inside a `ScrollArea`. During drag, if the list is taller than the panel, auto-scroll behavior depends on dnd-kit's defaults. This may need tuning if users have many sections and need to drag from top to bottom.

### 5. Custom Recipe Disk I/O
`list_saved_recipes` does a `read_dir` + filter on every modal open. `save_recipe` does `serde_json::to_string_pretty` + `fs::write`. For typical usage (< 10 custom recipes), this is fine. No caching needed.

### 6. Full Prompt Copy
`handleCopy` copies `preview.fullPrompt` to clipboard via `navigator.clipboard.writeText`. For very large prompts (100KB+), this is synchronous and could cause a brief UI freeze. Unlikely to be an issue in practice.

### 7. MCP Script Generation Not Used in Preview
The preview commands (`preview_custom_recipe`, `preview_prompt`) only build the prompt text — they do NOT generate MCP scripts. MCP generation only happens in `generate_mcp_config` which is called when creating a PTY session. This is correct and efficient.

### 8. Codebase Snapshot Cache
`CodebaseSnapshot` is computed once at project lock time (`set_locked_project`) and stored in `AppState.codebase_snapshot: Mutex<Option<CodebaseSnapshot>>`. It's cloned into every `PromptContext`. The snapshot is lightweight (file counts, language map, dir tree) but the clone happens on every preview call.

**Opportunity:** Could use `Arc<CodebaseSnapshot>` instead of cloning. Or the snapshot could be computed lazily on first braindump recipe use rather than eagerly on project lock.

---

## Compilation Status

- `cargo check -p prompt-builder` — clean
- `cargo check -p ralph4days` — clean
- `npx tsc --noEmit` — clean
- `cargo test -p prompt-builder` — pre-existing failure in `mcp/mod.rs` test (missing `Task` fields from sqlite-db schema migration, unrelated to this work)

---

## Files Created

| File | Lines | Purpose |
|------|-------|---------|
| `crates/prompt-builder/src/sections/metadata.rs` | 140 | Section display metadata registry |

## Files Modified

| File | What Changed |
|------|-------------|
| `crates/prompt-builder/src/context.rs` | `instruction_override` → `instruction_overrides: HashMap` |
| `crates/prompt-builder/src/sections/instructions/braindump.rs` | HashMap lookup |
| `crates/prompt-builder/src/sections/instructions/yap.rs` | HashMap lookup |
| `crates/prompt-builder/src/sections/instructions/ramble.rs` | HashMap lookup |
| `crates/prompt-builder/src/sections/instructions/discuss.rs` | HashMap lookup |
| `crates/prompt-builder/src/sections/instructions/task_exec.rs` | HashMap lookup |
| `crates/prompt-builder/src/sections/instructions/opus_review.rs` | HashMap lookup |
| `crates/prompt-builder/src/sections/mod.rs` | Added `pub mod metadata;` |
| `crates/prompt-builder/src/recipe.rs` | Added `build_sections_from_names()` |
| `crates/prompt-builder/src/lib.rs` | Added `build_custom_sections()`, `get_recipe_section_names()`, re-export `SectionInfo` |
| `src-tauri/src/commands.rs` | Updated `build_prompt_context()` signature, added 7 new commands + types |
| `src-tauri/src/lib.rs` | Registered 7 new commands |
| `src/components/PromptBuilderModal.tsx` | Complete rewrite: block-based editor with dnd-kit |
| `package.json` / `bun.lock` | Added @dnd-kit/core, @dnd-kit/sortable, @dnd-kit/utilities |

---

## Known Gaps / Not Done

1. **No drag overlay** — When dragging, the item moves in-place (transform). No floating overlay/ghost element. Works fine but a `DragOverlay` would look more polished.
2. **No recipe rename** — You can save and delete, but not rename a custom recipe.
3. **Old instruction file commands still exist** — `save_prompt_instructions`, `load_prompt_instructions`, `reset_prompt_instructions` write to `.ralph/prompts/{type}_instructions.md`. These are separate from the new JSON recipe system. They could be consolidated or removed if no longer needed.
4. **SectionInfo serde asymmetry** — `SectionInfo` uses default snake_case serde (from `prompt-builder` crate), while `SectionConfig`/`CustomRecipe` use `rename_all = "camelCase"` (from Tauri `commands.rs`). Frontend handles both formats but it's inconsistent.
5. **No validation on recipe names** — Users can save recipes with names containing spaces, special chars, etc. The filename is `{name}.json` directly.
6. **Pre-existing test failure** — `cargo test -p prompt-builder` fails in `mcp/mod.rs` test due to missing `Task` fields from sqlite-db schema additions (not caused by this work).
