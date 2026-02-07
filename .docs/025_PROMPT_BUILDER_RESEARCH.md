# 025: Prompt Builder — Research & Design Direction

## Status: NEEDS DESIGN (backend wiring done, UI not designed)

## Core Concept

The prompt builder is a **recipe editor**, not a prompt editor. The workflow is:

1. Human experiments with prompt composition (which sections, what order, what instruction text)
2. Human codifies what works into a reusable recipe
3. System follows the recipe automatically on every future run

**Think once, automate forever.** The UI should make step 1-2 easy. Step 3 is already handled by the Rust `Recipe` system.

## What Exists Today (Rust)

- `Recipe` = ordered `Vec<Section>` + `Vec<McpTool>`
- `Section` = named build function that takes `PromptContext` → `Option<String>`
- 6 hardcoded recipes: braindump, yap, ramble, discuss, task_execution, opus_review
- ~17 section types (project_context, feature_listing, task_listing, user_input, instructions, etc.)
- Each instruction section now has `default_text()` and checks `ctx.instruction_override`
- `build_sections()` returns `Vec<PromptSection>` for preview
- Backend commands wired: `preview_prompt`, `get_default_instructions`, `save/load/reset_prompt_instructions`

## What's Missing

The UI. We built the plumbing without designing the experience.

## Research: How Others Solve This

### Pattern 1: Block-Based Assembly (Best fit for Ralph)
**Localforge, PromptBuilder.space** — Sections are discrete cards/blocks. Toggle on/off, drag to reorder, edit text within each block. Maps directly to Ralph's Section concept.

### Pattern 2: Template Text with Inline Tokens
**Zapier "data pills", Salesforce Prompt Builder** — Free-form text with colored bubble tokens (`{{feature.name}}`, `{{task.title}}`) inserted from a picker. Good for editing instruction text within a section.

### Pattern 3: Resolution Preview with Real Data
**Salesforce** — Replace all placeholders with actual DB data so you see exactly what the LLM receives. Ralph can do this already via `preview_prompt` command.

### Pattern 4: Dual-Mode (Visual + Raw)
**SendGrid, PromptLayer** — Visual block editor for structure, raw text toggle for power users. Solves the casual-vs-power-user tension.

### Pattern 5: Split-Pane Editor + Live Preview
**Humanloop, OpenAI Playground, LangSmith** — Template on left, chat/preview on right. Most common pattern. What we naively built first.

### Pattern 6: Side-by-Side Comparison
**Agenta, Langfuse** — Run same input through multiple prompt variants simultaneously. Good for A/B testing recipes.

## Open Questions (for future design session)

1. **Scope**: Is the user configuring which sections go into a recipe, or editing the text within sections, or both?
2. **Custom recipes**: Can users create entirely new recipes, or just modify the 6 existing ones?
3. **Variable insertion**: Do instruction sections need `{{variable}}` support (e.g., `{{project.title}}`), or is the current "build function pulls from PromptContext" approach sufficient?
4. **MCP tools**: Should the recipe editor also let you configure which MCP tools are available, or just the prompt sections?
5. **Storage format**: Recipes as YAML/JSON files in `.ralph/prompts/`? Or extend the SQLite DB?
6. **Sharing**: Should custom recipes be exportable/importable between projects?

## Recommended Approach (when ready)

Given Ralph's architecture, the **Localforge block model + Salesforce resolution preview** is the best fit:

- Left panel: Blocks representing sections, draggable to reorder, toggleable
- Each block expandable to edit instruction text within
- Right panel: Live preview with real project data substituted
- Save as custom recipe to `.ralph/prompts/{name}.yaml`
- Dual-mode toggle: block view vs raw assembled text

But this needs proper UX thinking first. Don't build until the design is clear.

## References

- Localforge (MIT, Electron, block-based): https://github.com/rockbite/localforge
- Langfuse (MIT, prompt management): https://github.com/langfuse/langfuse
- Agenta (open source, playground IDE): https://github.com/Agenta-AI/agenta
- PromptL (LGPL-3, templating DSL): https://github.com/latitude-dev/promptl
- Helicone (open source, `{{hc:name:type}}` syntax): https://github.com/Helicone/helicone
