# Predefined Disciplines — Image Generation Workflow

**Mission:** Generate and validate all 32 discipline portrait images (4 stacks × 8 disciplines) through dev → prod → composite pipeline.

**DO NOT ask for permission between stacks. Process all 4 stacks in sequence. This is ONE continuous workflow, not 4 separate tasks.**

## Overview: 4 Stacks, 32 Images, One Workflow

| Stack | Name | Medium | Style | Dev Steps | Prod Steps | Images Generated |
|-------|------|--------|-------|-----------|-----------|------------------|
| 01 | Generic | Wes Anderson photo | Symmetrical pastel, headless | 14 | 28 | 8 |
| 02 | Desktop | Watercolor | 1980s blue-collar, serious | 9 | 18 | 8 |
| 03 | SaaS | 3D render | Cyberpunk corporate, neon rim | 20 | 40 | 8 |
| 04 | Mobile | Colored pencil | Elvish fantasy, white paper | 16 | 32 | 8 |

## File Locations (All in `src/`)

```
src/
├── image_prompts.yaml                          # Global layer (all stacks)
├── defaults/disciplines/
│   ├── 01_generic/
│   │   ├── ABOUT.yaml                          # Stack 01 identity
│   │   ├── 00_implementation.yaml              # Implementation discipline
│   │   ├── 01_refactoring.yaml
│   │   ├── 02_investigation.yaml
│   │   ├── 03_testing.yaml
│   │   ├── 04_architecture.yaml
│   │   ├── 05_devops.yaml
│   │   ├── 06_security.yaml
│   │   └── 07_documentation.yaml
│   ├── 02_desktop/
│   │   ├── ABOUT.yaml
│   │   ├── 00_frontend.yaml
│   │   ├── 01_backend.yaml
│   │   ├── 02_data.yaml
│   │   ├── 03_integration.yaml
│   │   ├── 04_platform.yaml
│   │   ├── 05_quality.yaml
│   │   ├── 06_security.yaml
│   │   └── 07_documentation.yaml
│   ├── 03_saas/
│   │   └── ... (same structure)
│   └── 04_mobile/
│       └── ... (same structure)
└── comfyui_workflows/
    └── generate_discipline.json
```

Generated images save directly into each discipline's folder with format:
```
<NN>_<name>_<steps>_<WxH>_<base36_timestamp>.png
Example: 00_implementation_14_512x768_yw3lkm.png
```

## Full Workflow: Per-Stack Pipeline

### For Each Stack (01, 02, 03, 04):

#### Phase 1: Dev Generation & Iteration

**Step 1: Generate all 8 disciplines at dev quality**
```bash
for i in 00 01 02 03 04 05 06 07; do
    just gen-image <STACK> $i --dev
done
```

**Step 2: Review each discipline individually**
- Open each generated PNG in order (00 through 07)
- Check against `IMAGE_REVIEW_CHECKLIST.md`:
  - **Global checks (G01-G14):** Framing, camera, lighting, pose, ground, color, face
  - **Stack checks (S1-S4):** Medium, style, tone (varies by stack)
  - **Discipline checks:** Character match, color dominance, props, environment

**Step 3: If any fails, fix the prompt and regenerate**
- Identify which layer failed (global/stack/discipline)
- Edit the YAML file
- Regenerate: `just gen-image <STACK> <DISCIPLINE> --dev`
- Re-review
- Repeat until ALL checks pass (max 5 attempts per discipline)

**Step 4: Composite review**
- Run compositor: `python compose_stack.py <STACK>`
- Review the composite image against composite checks (C01-C10)
- If any composite check fails, regenerate ONLY the failing discipline(s)
- Re-composite and re-review
- Do NOT proceed to prod until composite is perfect

#### Phase 2: Prod Generation & Final Review

**Step 5: Generate all 8 at prod quality**
```bash
for i in 00 01 02 03 04 05 06 07; do
    just gen-image <STACK> $i --prod
done
```

**Step 6: Review all prod images**
- Prod uses more steps and higher resolution
- Results may differ slightly from dev
- If any prod image fails checklist, iterate at prod quality
- Re-composite if any changes were made

#### Phase 3: Complete and Move to Next Stack

**Step 7: Final composite validation**
- Run compositor one final time: `python compose_stack.py <STACK>`
- Verify composite passes all C01-C10 checks
- Stack is DONE when dev + prod + composite all pass

**Step 8: Move to next stack**
- Do NOT ask for permission
- Proceed immediately to next stack (01 → 02 → 03 → 04)

## Key Rules (Non-Negotiable)

### 1. **NEVER Use `--test`**
- `--test` uses 1 step and produces blurry garbage
- Always use `--dev` for iteration (9-14 steps)
- Only use `--prod` after dev composite is perfect (18-40 steps)

### 2. **Batch Generation is Mandatory**
Use the for-loop pattern for all 8:
```bash
for i in 00 01 02 03 04 05 06 07; do
    just gen-image <STACK> $i --dev
done
```
Do NOT generate one at a time unless debugging a single discipline.

### 3. **Color Dominance is Critical (G12/G13)**
The discipline color must COMPLETELY drench the image:
- Clothing (6-8 color mentions in positive prompt)
- Props
- Lighting
- Environment
- Shadows/highlights

If you see tan, beige, white, brown, or any off-color: FAIL. Add more color to the discipline prompt.

### 4. **Stack-Level Consistency**
All 8 disciplines in a stack must:
- Use the same medium (all watercolor, all 3D, all pencil)
- Have same lighting direction (upper left key light)
- Have same camera angle (eye-level straight on)
- Have similar body scale and eye line
- Be equally dense in prompt detail (no vague/flat discipline)

If one discipline looks different, it failed. Fix it.

### 5. **Composite Failures = Framing Issues in Source**
The compositor doesn't create misalignment. It just arranges images side-by-side.
- Character too small? That discipline's framing is wrong.
- Character offset up/down? Eye line (G02) or ankle line (G03) is wrong.
- Character too wide? Arms are spread (G08 violation).
- Lighting different? Key light not upper left (G07).

Fix the SOURCE image, not the compositor.

### 6. **Prompt Layer Rules**
- **Global** (`image_prompts.yaml`): Technical quality only, affects all 32 images
- **Stack** (`ABOUT.yaml`): Visual identity, affects all 8 in that stack
- **Discipline** (`NN_name.yaml`): Character uniqueness, affects only that one

Edit the right layer. Wrong layer = wasted iterations.

### 7. **Iterate Until PERFECT**
Maximum 5 attempts per discipline. If still failing after 5:
- Document the failure in git commit message
- Move to next discipline
- Revisit later with fresh eyes or ask for human review

Do NOT ship broken images.

## Success Criteria

You are DONE when:

- ✅ Stack 01 Generic: All 8 dev + composite pass, all 8 prod + composite pass
- ✅ Stack 02 Desktop: All 8 dev + composite pass, all 8 prod + composite pass
- ✅ Stack 03 SaaS: All 8 dev + composite pass, all 8 prod + composite pass
- ✅ Stack 04 Mobile: All 8 dev + composite pass, all 8 prod + composite pass
- ✅ All 32 images (4 × 8) exist in their discipline folders
- ✅ All 4 composite images created by compositor

## Sandbox Warning

**Image generation requires ComfyUI access, which requires network access.**

The Claude Code sandbox blocks network with `--unshare-net`. **If generation fails with "Connection refused", you MUST disable the sandbox in Claude Code settings.**

The generate_discipline_image binary checks for `SANDBOX_RUNTIME` and skips validation when sandboxed, but generation will still fail if ComfyUI isn't reachable.

## Tracking Progress

Track progress as you go:

```
## Stack 01: Generic
- [x] Dev generation (all 8)
- [x] Dev review & iteration
- [x] Dev composite
- [ ] Prod generation (all 8)
- [ ] Prod review & iteration
- [ ] Prod composite

## Stack 02: Desktop
- [ ] Dev generation (all 8)
- [ ] Dev review & iteration
- [ ] Dev composite
- [ ] Prod generation (all 8)
- [ ] Prod review & iteration
- [ ] Prod composite

## Stack 03: SaaS
- [ ] Dev generation (all 8)
- [ ] Dev review & iteration
- [ ] Dev composite
- [ ] Prod generation (all 8)
- [ ] Prod review & iteration
- [ ] Prod composite

## Stack 04: Mobile
- [ ] Dev generation (all 8)
- [ ] Dev review & iteration
- [ ] Dev composite
- [ ] Prod generation (all 8)
- [ ] Prod review & iteration
- [ ] Prod composite
```

## Reference Documentation

- **IMAGE_REVIEW_CHECKLIST.md** — Per-image validation rules (G01-G14, S1-S4, C01-C10)
- **.docs/040_DISCIPLINE_PORTRAIT_PROMPT_METHODOLOGY.md** — Deep dive on three-layer prompt system
- **.docs/044_IMAGE_REVIEW_CHECKLIST_ENHANCEMENTS.md** — Recent enhancements to the checklist

## DO NOT

- ❌ Ask for permission between stacks
- ❌ Wait for approval to continue
- ❌ Use `--test` for anything
- ❌ Skip composite review
- ❌ Ship images that fail checklist
- ❌ Edit global prompts when it's a discipline issue
- ❌ Trust the compositor if alignment is off (fix source images)

## DO

- ✅ Process all 4 stacks in sequence: 01 → 02 → 03 → 04
- ✅ Use batch generation (for-loop) for all 8
- ✅ Review every image against the checklist
- ✅ Iterate until perfect or max 5 attempts
- ✅ Validate composite before moving to prod
- ✅ Commit after each stack completes
- ✅ Document failures and fixes in commit messages
