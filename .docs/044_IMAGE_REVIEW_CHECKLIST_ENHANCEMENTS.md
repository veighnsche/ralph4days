# Image Review Checklist Enhancements

**Date:** 2026-02-09
**File Updated:** `crates/predefined-disciplines/IMAGE_REVIEW_CHECKLIST.md`

## Summary

Enhanced the IMAGE_REVIEW_CHECKLIST.md with clarifications, file path references, batch generation examples, and deeper integration with the established prompt methodology. These improvements make the checklist more actionable and reduce ambiguity for agents executing the image generation workflow.

## Changes Made

### 1. Added Cross-Reference to Methodology Document

Added prominent reference to `.docs/040_DISCIPLINE_PORTRAIT_PROMPT_METHODOLOGY.md` at the top of the checklist. This connects the practical checklist to the deep theoretical foundation.

**Why:** The methodology doc contains crucial context about the three-layer prompt system, color dominance rules (6-8 mentions), and visual consistency principles. Agents need to know this exists before starting work.

### 2. Added Quick Reference Section for File Locations

Created a visual directory tree showing exactly where each prompt layer lives:
- Global layer: `src/image_prompts.yaml`
- Stack layer: `src/defaults/disciplines/<STACK>/ABOUT.yaml`
- Discipline layer: `src/defaults/disciplines/<STACK>/<NN>_<name>.yaml`

**Why:** Previous fix location table said "Stack `ABOUT.yaml`" without full paths. Ambiguous paths waste time and tokens. The tree structure is unambiguous.

### 3. Enhanced Step 2 with Batch Generation Options

Split generation into two clear options:
- **Option A:** One at a time (for validating first discipline)
- **Option B:** Batch loop for all 8 (for iteration after validation)

Added timing info: ~10 seconds per image, ~2 minutes for batch of 8.

**Why:** The methodology doc (lesson #18) showed batch generation with a for loop was effective for SaaS stack generation. Making this explicit in the workflow saves time.

### 4. Improved Fix Location Table with Full Paths

Converted vague references to complete paths:
- Before: "Discipline YAML image_prompt positive"
- After: "Discipline: `src/defaults/disciplines/<STACK>/<NN>_<name>.yaml` image_prompt positive (add 6-8 color mentions)"

Added new row for "Style drift" failures pointing to stack ABOUT.yaml negatives.

**Why:** Specificity prevents confusion. The "6-8 color mentions" guidance is from the methodology and critical for passing G12/G13 checks.

### 5. Enhanced Prompt Editing Rules

Added two critical rules from methodology doc:
- Target 6-8 color mentions with specific examples (clothing, trim, accessory, feature, rim lighting, ambient glow, "bathed in [color] light", environment details)
- Each discipline prompt must be equally dense — audit all 8 side by side before generating

**Why:** These are hard-won lessons from 40+ generation cycles on SaaS stack. Unequal prompt density causes vague/flat results (methodology lesson #14).

### 6. Expanded Composite Review Section

Added critical clarification: **Composite failures are framing issues in individual images, NOT compositor bugs.**

Documented 5 common composite failure patterns with specific diagnoses:
- One character too small → framing issue in that discipline
- One character offset up/down → eye line (G02) or ankle line (G03) wrong
- One character much wider → arms spread wide (G08 violation)
- Different lighting direction → key light not upper left (G07)
- Style inconsistency → missing style-blocking negatives in stack ABOUT.yaml

**Why:** This prevents agents from trying to "fix the compositor" when the issue is in the source images. Directs debugging to the right layer.

### 7. Added Batch Generation Example to Prod Workflow

Step 8 now shows both for-loop batch generation AND individual commands.

**Why:** Consistency with Step 2 and efficiency for final prod generation pass.

### 8. Added Three-Layer Prompt System Explanation

Created a new section explaining the three-layer concatenation system with concrete examples:
- Layer 1 (Global): Technical quality only
- Layer 2 (Stack): Shared visual identity for consistency
- Layer 3 (Discipline): What makes THIS character unique

Includes the key principle: "If a fix applies to just one discipline, edit the discipline YAML. If it applies to all 8 in the stack, edit the stack ABOUT.yaml. If it applies to all stacks, edit the global image_prompts.yaml."

**Why:** Understanding the prompt architecture is essential for effective debugging. This was scattered across methodology doc — now it's in the workflow doc where agents need it.

## Impact

These enhancements transform the checklist from a reference document into a more complete execution guide. Agents now have:

1. **Unambiguous file paths** — no guessing where to edit
2. **Batch generation patterns** — faster iteration workflow
3. **Composite debugging guide** — direct path from symptom to fix
4. **Prompt density rules** — prevents unequal detail causing flat results
5. **Three-layer system clarity** — understand why edits go where they go

## Files Modified

- `crates/predefined-disciplines/IMAGE_REVIEW_CHECKLIST.md` - Enhanced with clarifications, examples, and methodology integration

## Related Documentation

- `.docs/040_DISCIPLINE_PORTRAIT_PROMPT_METHODOLOGY.md` - Source of the three-layer system, color dominance rules, and iteration lessons
- `.docs/042_IMAGE_GENERATION_REVIEW_CHECKLIST.md` - High-level session workflow (different from per-image checklist)
- `.docs/043_SANDBOX_BLOCKS_COMFYUI_ACCESS.md` - Sandbox issue that blocks image generation

## Next Steps

The checklist is now comprehensive and actionable. Ready for agents to execute image generation workflow for:
- Stack 01 (Generic) - Wes Anderson photorealistic style
- Stack 02 (Desktop) - Watercolor 1980s blue-collar style
- Stack 03 (SaaS) - Cel-shaded cyberpunk style
- Stack 04 (Mobile) - Colored pencil elvish fantasy style
