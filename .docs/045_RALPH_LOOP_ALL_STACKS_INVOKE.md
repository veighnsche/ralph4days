# Ralph Loop: Generate All 4 Stacks (32 Discipline Images)

**Date:** 2026-02-09

## The Perfect Invoke Command

Copy and paste this EXACT command into Claude Code to start the Ralph Loop:

```
/ralph-loop "crates/predefined-disciplines/CLAUDE.md" --completion-promise "ALL_STACKS_COMPLETE" --max-iterations 120
```

That's it. One line. No backslashes. Just copy and paste.

## What This Does

The Ralph Loop will:

1. **Read `crates/predefined-disciplines/CLAUDE.md`** — The workflow document that defines everything
2. **Execute the complete 4-stack pipeline:**
   - Stack 01 (Generic): 8 discipline images, Wes Anderson style, dev → prod → composite
   - Stack 02 (Desktop): 8 discipline images, watercolor 1980s, dev → prod → composite
   - Stack 03 (SaaS): 8 discipline images, cyberpunk 3D render, dev → prod → composite
   - Stack 04 (Mobile): 8 discipline images, colored pencil elvish fantasy, dev → prod → composite
3. **Validate every image** against `IMAGE_REVIEW_CHECKLIST.md` (G01-G14 global, stack-specific, discipline-specific)
4. **Iterate until perfect** (max 5 attempts per discipline, max 120 total iterations)
5. **Auto-composite** after all 8 in each stack pass
6. **Continue to next stack** without asking for permission
7. **Exit cleanly** when I output `<promise>ALL_STACKS_COMPLETE</promise>` (when genuinely TRUE)

## Key Guarantees

- ✅ **NO `--test` flag** (1 step, blurry garbage) — only `--dev` and `--prod`
- ✅ **Batch generation** for all 8 disciplines per stack
- ✅ **Every image validated** against the full checklist
- ✅ **Composite review** after each stack
- ✅ **All 4 stacks processed** in sequence: 01 → 02 → 03 → 04
- ✅ **No permission requests** between stacks
- ✅ **No false completion promises** — loop exits only when all 32 images are genuinely done

## Timeline Estimate

- Stack 01: ~8-12 iterations (batch dev gen 2 min, iterate 3-4 cycles, prod gen 3 min, composite)
- Stack 02: ~8-12 iterations (same pattern)
- Stack 03: ~10-15 iterations (longer steps, more complex)
- Stack 04: ~10-15 iterations (same as Stack 03)
- **Total: ~40-60 iterations expected, max 120 iterations allowed**

## Files Referenced

- **Workflow doc:** `crates/predefined-disciplines/CLAUDE.md` — Complete execution guide
- **Validation:** `crates/predefined-disciplines/IMAGE_REVIEW_CHECKLIST.md` — Per-image rules (G01-G14, S1-S4, C01-C10)
- **Methodology:** `.docs/040_DISCIPLINE_PORTRAIT_PROMPT_METHODOLOGY.md` — Three-layer prompt system
- **Generation script:** `just gen-image <STACK> <DISCIPLINE> --dev|--prod`
- **Compositor:** `python crates/predefined-disciplines/compose_stack.py <STACK>`

## Success Criteria

Loop completes when:
- ✅ Stack 01: All 8 dev + composite pass, all 8 prod + composite pass
- ✅ Stack 02: All 8 dev + composite pass, all 8 prod + composite pass
- ✅ Stack 03: All 8 dev + composite pass, all 8 prod + composite pass
- ✅ Stack 04: All 8 dev + composite pass, all 8 prod + composite pass
- ✅ All 32 images exist in their discipline folders
- ✅ All 4 composite images created

I will then output: `<promise>ALL_STACKS_COMPLETE</promise>`

## Sandbox Warning

**CRITICAL:** Image generation requires ComfyUI network access.

If you see "Connection refused" errors, **disable the sandbox in Claude Code settings** before running the loop.

The generator detects sandbox mode and skips validation, but the actual generation will still fail without network access to ComfyUI.

## Ready to Go

Just paste the command:

```
/ralph-loop "crates/predefined-disciplines/CLAUDE.md" --completion-promise "ALL_STACKS_COMPLETE" --max-iterations 120
```

No typos. No modifications. No second-guessing.
