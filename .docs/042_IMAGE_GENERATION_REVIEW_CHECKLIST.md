# Image Generation Review Checklist

## Generation Timing Strategy

**Daytime (NOW):** Generate all `--dev` images
- Lower quality/faster generation
- Use for initial validation and prompt tuning
- Settings: Stack-specific dev steps and megapixels (typically 14 steps, 1.0MP)

**Nighttime:** Generate all `--prod` images
- Higher quality/slower generation
- Use for final production assets
- Settings: Stack-specific prod steps and megapixels (typically 28 steps, 2.0MP)

## 🚨 CRITICAL: Complete ALL Stacks in One Session

**DO NOT STOP after Stack 01!**

When generating images, you MUST:
1. Generate ALL 8 disciplines for Stack 01
2. Review and fix Stack 01 images
3. **IMMEDIATELY continue to Stack 02** (do not wait for permission)
4. Generate ALL 8 disciplines for Stack 02
5. Review and fix Stack 02 images
6. **IMMEDIATELY continue to Stack 03** (do not wait for permission)
7. Generate ALL 8 disciplines for Stack 03
8. Review and fix Stack 03 images
9. **IMMEDIATELY continue to Stack 04** (do not wait for permission)
10. Generate ALL 8 disciplines for Stack 04
11. Review and fix Stack 04 images
12. ONLY THEN mark the session complete

**This is a continuous workflow. Do not stop between stacks. Do not ask permission to continue. Just keep going until all 32 images (4 stacks × 8 disciplines) are complete.**

## Workflow for Each Stack

For each stack (01_generic, 02_desktop, 03_saas, 04_mobile):

### 1. Generate All Disciplines (Dev Mode)

```bash
just gen-image <STACK> 00 --dev
just gen-image <STACK> 01 --dev
just gen-image <STACK> 02 --dev
just gen-image <STACK> 03 --dev
just gen-image <STACK> 04 --dev
just gen-image <STACK> 05 --dev
just gen-image <STACK> 06 --dev
just gen-image <STACK> 07 --dev
```

### 2. Review Each Image Against Requirements

For each generated image, verify:

#### Technical Requirements
- [ ] Resolution matches stack dev settings
- [ ] Aspect ratio is 1:2 (portrait)
- [ ] File size reasonable for dev quality
- [ ] No corruption or artifacts

#### Visual Requirements (from DISCIPLINE_PORTRAIT_PROMPT_METHODOLOGY.md)
- [ ] **Monochromatic blue palette** - Dominant blue tones matching discipline color
- [ ] **Single subject** - One character, centered, clear focus
- [ ] **Portrait framing** - Head and shoulders visible, professional composition
- [ ] **Minimal background** - Clean, uncluttered, supports subject
- [ ] **Professional lighting** - Even, studio-quality lighting
- [ ] **Subject matches discipline** - Visual representation aligns with role
- [ ] **No text/watermarks** - Clean image without overlays
- [ ] **Consistent style** - Matches other images in the stack

#### Stack-Specific Requirements
Check stack ABOUT.yaml for any additional visual themes or constraints.

### 3. Fix Issues

If image doesn't meet requirements:

**Option A: Adjust prompt** (if systematic issue with all images in discipline/stack)
1. Edit prompt in `crates/predefined-disciplines/src/defaults/image_prompts.yaml`
2. Regenerate: `just gen-image <STACK> <DISCIPLINE> --dev`

**Option B: Regenerate with different seed** (if random variance issue)
1. Just regenerate: `just gen-image <STACK> <DISCIPLINE> --dev`
2. Seeds are randomized each run, so you'll get a different result

**Option C: Increase quality temporarily** (if dev quality too low to judge)
1. Use `--prod` flag for one-off higher quality test
2. Return to `--dev` once prompt is validated

### 4. Iterate Until Correct

- Keep regenerating until image meets all requirements
- Document any prompt changes needed
- Track which disciplines needed multiple attempts

### 5. Mark Stack Complete

Once all 8 disciplines in a stack pass review:
- [ ] Stack 01 (Generic) - 8/8 disciplines approved
- [ ] Stack 02 (Desktop) - 8/8 disciplines approved
- [ ] Stack 03 (SaaS) - 8/8 disciplines approved
- [ ] Stack 04 (Mobile) - 8/8 disciplines approved

## Stack Order

Recommended order (based on prompt methodology maturity):

1. **Stack 01 (Generic)** - Baseline, most mature prompts
2. **Stack 02 (Desktop)** - Desktop-specific themes validated
3. **Stack 03 (SaaS)** - Cloud/web service themes
4. **Stack 04 (Mobile)** - Mobile-specific themes

## Notes

- Save rejected images with notes on what was wrong (for learning)
- If 3+ regenerations fail, consider prompt architecture issue
- Dev images are stored in: `crates/predefined-disciplines/src/defaults/disciplines/<STACK>/images/`
- Naming format: `<DISCIPLINE_ID>_<NAME>_<STEPS>_<WIDTH>x<HEIGHT>_<SEED>.png`

## Night Shift TODO

Once all dev images approved, schedule for nighttime:
- [ ] Run all `--prod` generations (slow, 28 steps, 2MP)
- [ ] Replace dev images with prod versions
- [ ] Final review of prod quality
- [ ] Commit final image set

## Current Session Status

**Date:** 2026-02-09 (Daytime)
**Mode:** Dev image generation and review

### Stack 01 - Generic
- [ ] 00_implementation
- [ ] 01_architecture
- [ ] 02_testing
- [ ] 03_deployment
- [ ] 04_documentation
- [ ] 05_security
- [ ] 06_performance
- [ ] 07_maintenance

### Stack 02 - Desktop
- [ ] 00_frontend
- [ ] 01_backend
- [ ] 02_database
- [ ] 03_api
- [ ] 04_infrastructure
- [ ] 05_monitoring
- [ ] 06_analytics
- [ ] 07_integration

### Stack 03 - SaaS
- [ ] 00_product
- [ ] 01_engineering
- [ ] 02_platform
- [ ] 03_data
- [ ] 04_reliability
- [ ] 05_growth
- [ ] 06_compliance
- [ ] 07_support

### Stack 04 - Mobile
- [ ] 00_ios
- [ ] 01_android
- [ ] 02_crossplatform
- [ ] 03_backend_mobile
- [ ] 04_ux
- [ ] 05_performance_mobile
- [ ] 06_distribution
- [ ] 07_crash_monitoring
