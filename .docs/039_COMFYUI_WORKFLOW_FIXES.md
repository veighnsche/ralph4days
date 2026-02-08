# ComfyUI Workflow Fixes for Z-Image HD

Fixed workflow for discipline character portrait generation based on Z-Image HD best practices guide.

## Changes Made

### 1. Resolution (Node 68)
**Before:** `1200 × 752` (incorrect aspect ratio)
**After:** `832 × 1216` (9:16 portrait)

**Reasoning:** Z-Image HD guide recommends 832×1216 or 768×1152 for portrait work. This gives proper 9:16 aspect ratio for character cards.

### 2. Steps (Node 69)
**Before:** `50` (excessive)
**After:** `28` (optimal range)

**Reasoning:** HD guide says 25-30 steps for quality. 50 is overkill and wastes compute without quality gain.

### 3. CFG/Guidance (Node 69)
**Before:** `3` (too low)
**After:** `7` (HD sweet spot)

**Reasoning:** Guide recommends 6.5-7.5 for HD. Below 6 loses prompt adherence.

### 4. Sampler (Node 69)
**Before:** `res_multistep`
**After:** `dpmpp_2m` with `karras` scheduler

**Reasoning:** HD guide specifically calls out DPM++ 2M Karras as the high-quality sampler for Z-Image.

### 5. Typo Fix (Node 81)
**Before:** `__PORITIVE_GLOBAL__`
**After:** `__POSITIVE_GLOBAL__`

**Reasoning:** Typo would break template replacement.

### 6. Missing CLIP Connection (Node 83)
**Before:** No `clip` input
**After:** Connected to node 62 (CLIP loader)

**Reasoning:** Text encoding requires CLIP model connection. This was a broken node.

## Workflow Structure

The workflow correctly concatenates prompts in layers:

```
Positive: Global → Stack → Discipline (ConditioningConcat)
Negative: Global → Stack → Discipline (ConditioningConcat)
```

This matches our 3-tier prompt architecture from `image_prompts.yaml`.

## Template Placeholders

The workflow expects these to be replaced:

- `__POSITIVE_GLOBAL__` - Global positive prompt
- `__NEGATIVE_GLOBAL__` - Global negative prompt
- `__POSITIVE_STACK__` - Stack-specific positive prompt
- `__NEGATIVE_STACK__` - Stack-specific negative prompt
- `__POSITIVE_DISCIPLINE__` - Discipline subject + accent color
- `__NEGATIVE_DISCIPLINE__` - (currently empty in our config, can be omitted)

## Next Steps for ComfyUI Integration

1. **Backend API endpoint** to accept:
   - `stack_id` (1-4)
   - `discipline_name` (e.g., "frontend")
   - `seed` (optional, for reproducibility)

2. **Template replacement**:
   - Load workflow JSON
   - Call `build_image_prompt(stack_id, discipline_name)` from our backend
   - Replace placeholders with actual prompts
   - Set seed in node 69

3. **POST to ComfyUI** at `http://localhost:8188/prompt`:
   ```json
   {
     "prompt": { workflow_json_here }
   }
   ```

4. **Monitor queue** and download result from `/view?filename=...`

5. **Save to** `.ralph/generated/disciplines/{stack_id}/{discipline_name}.png`

## Z-Image HD Parameters Summary

```yaml
Resolution: 832 × 1216 (9:16 portrait)
Steps: 28
CFG: 7.0
Sampler: dpmpp_2m
Scheduler: karras
Denoise: 1.0 (for initial generation)
Shift: 1.5 (AuraFlow model sampling)
```

For 4K upscaling (future enhancement):
```yaml
Upscale Factor: 2x or 4x
Sharpening: 10-20%
Denoise: 0.1-0.25 (low, preserves structure)
```

## File Location

Fixed workflow saved to: `workflows/ralph_discipline_portrait.json`
