# Predefined Disciplines — Image Generation Workflow

## The Rule: ONE Image at a Time

Generate ONE image. Review it. Fix it. Only when it passes ALL checks, move to the next. This is a serial loop, not a batch job.

```
for each stack (01, 02, 03, 04):
  for each discipline (00 through 07):
    loop:
      generate ONE image at --dev
      review against the flat checklist below
      if ALL checks pass → accept, break
      if any check fails → fix the prompt, regenerate (max 5 attempts)
    end loop
  end for
  run composite review (C01-C10)
  if composite fails → fix ONLY the failing discipline(s), re-composite
  regenerate all 8 at --prod
  review each prod image (may differ from dev)
  final composite at prod quality
end for
```

**DO NOT batch generate all 8 then review.** You WILL miss failures and waste iterations.
**DO NOT skip to --prod.** Dev iteration is cheaper. Validate at --dev first.
**DO NOT ask permission between stacks.** Process 01 → 02 → 03 → 04 continuously.
**NEVER use `--test`.** It produces 1-step blurry garbage.

## Commands

```bash
just gen-image <STACK> <DISCIPLINE> --dev    # Iteration (9-20 steps depending on stack)
just gen-image <STACK> <DISCIPLINE> --prod   # Final quality (18-40 steps)
python crates/predefined-disciplines/compose_stack.py <STACK>   # Composite after all 8 accepted
```

Stack numbers: 01 (Generic), 02 (Desktop), 03 (SaaS), 04 (Mobile). Discipline numbers: 00-07.

## File Locations

```
src/
├── image_prompts.yaml                          # Global layer (all stacks)
├── defaults/disciplines/
│   ├── 01_generic/
│   │   ├── ABOUT.yaml                          # Stack layer (shared by all 8)
│   │   ├── 00_implementation.yaml              # Discipline layer (unique)
│   │   └── ... (01-07)
│   ├── 02_desktop/  (same structure)
│   ├── 03_saas/     (same structure)
│   └── 04_mobile/   (same structure)
└── comfyui_workflows/
    └── generate_discipline.json
```

Generated images: `<NN>_<name>_<steps>_<WxH>_<base36_timestamp>.png` in each discipline folder.

## Three-Layer Prompt System

Every image = `global + stack + discipline` concatenated.

| Layer | File | Scope | Contains |
|-------|------|-------|----------|
| Global | `src/image_prompts.yaml` | All 32 images | Technical quality only (lighting, DoF, studio) |
| Stack | `src/defaults/disciplines/<STACK>/ABOUT.yaml` → `image_prompt` | All 8 in stack | Art style, framing, composition, mood |
| Discipline | `src/defaults/disciplines/<STACK>/<NN>_<name>.yaml` → `image_prompt` | One image | Character, color, clothing, props, environment |

**Fix the right layer.** Wrong layer = wasted iterations:

| Failure | Fix |
|---------|-----|
| Framing wrong (cropped, wrong angle) | Global: `image_prompts.yaml` |
| Medium wrong (3D instead of watercolor) | Stack: `ABOUT.yaml` |
| Tone wrong (too happy, wrong era) | Stack: `ABOUT.yaml` |
| Color not dominant enough | Discipline: `<NN>_<name>.yaml` (add 6-8 color mentions) |
| Wrong character/props/environment | Discipline: `<NN>_<name>.yaml` |
| Unwanted element keeps appearing | Add to appropriate negative prompt |
| Style drift (wrong art medium) | Stack: `ABOUT.yaml` negative |

**Prompt rules:**
- Positive prompts: ONLY what you WANT. Never "no X" in positives.
- Negative prompts: ONLY what you DON'T want.
- If something persists despite negatives, strengthen the positive instead.
- All 8 disciplines in a stack must be equally dense in prompt detail.

---

## Flat Checklist: Every Single Image

Review each generated image against ALL of these. Record PASS/FAIL for each.

### Global Checks (G01-G14)

| ID | Check | Pass When |
|----|-------|-----------|
| G01 | Full body visible | Head and both feet fully in frame, nothing cropped |
| G02 | Eye line ~15% | Eyes (or head-object center) at ~15% from top |
| G03 | Ankle line ~90% | Feet/ankles at ~90% from top |
| G04 | Body fills ~75% | Character occupies ~75% of image height |
| G05 | Single person centered | One character, horizontally centered |
| G06 | Eye level camera | Camera at eye height, not looking up/down |
| G07 | Key light upper left | Highlight on character's left, shadow falls lower right |
| G08 | Arms compact | Arms within shoulder width, not spread wide |
| G09 | Standing upright | Not leaning, tilted, hunched, or crouching |
| G10 | Feet on floor | Clear ground plane, feet planted, not floating |
| G11 | Clean image | No text, watermarks, extra limbs, deformed features |
| G12 | Color DOMINATES | Entire image drenched in discipline color — clothing, props, lighting, background |
| G13 | Monochromatic read | At 64px thumbnail, character reads as a single color blob |
| G14 | Face direction | Facing viewer, head up, eyes forward (Generic: no face — head is object) |

**G12/G13 are the hardest.** If you see tan, beige, white, brown, or any off-color: FAIL. Add the discipline color to EVERY element: walls, floor, ceiling, fixtures, lighting, shadows. Target 6-8 color mentions in the positive prompt. Add unwanted colors to negatives.

**Global known conflicts:** Global says "dark moody background" but Generic needs pastel interiors and Mobile needs white paper. Stack layer overrides global for background — use the stack rule.

### Stack Checks

After global checks pass, verify the image matches its stack identity.

#### Stack 01: Generic — Wes Anderson photograph

| ID | Check | Pass When |
|----|-------|-----------|
| S1-01 | Photorealistic Wes Anderson | Looks like a photograph, not illustration/painting/3D |
| S1-02 | Perfect symmetry | Room and composition obsessively symmetrical |
| S1-03 | Pastel environment | Background is a real pastel-colored interior |
| S1-04 | Deadpan humor | Character comically out of place but standing still |
| S1-05 | NO human face | Head entirely replaced by discipline object — no eyes/nose/mouth |
| S1-06 | Warm vintage film | Slightly desaturated, warm tones, vintage film stock |

#### Stack 02: Desktop — Watercolor illustration

| ID | Check | Pass When |
|----|-------|-----------|
| S2-01 | Watercolor | Visible brushstrokes, paper texture, bleeding edges, dripping |
| S2-02 | NOT photorealistic | Clearly an illustration |
| S2-03 | NOT digital art | No cel-shading, no flat colors, no thick outlines |
| S2-04 | Serious determined | Stoic blue-collar hero, not smiling or playful |
| S2-05 | NOT horror | Heroic and powerful, not creepy or sinister |
| S2-06 | 1980s aesthetic | Clothing, props, environment read as 1980s |
| S2-07 | Face visible and lit | Facial features clearly readable |

#### Stack 03: SaaS — 3D cyberpunk render

| ID | Check | Pass When |
|----|-------|-----------|
| S3-01 | 3D render | Clearly a 3D character, smooth mesh |
| S3-02 | NOT watercolor/pencil/painting | No paper texture, brushstrokes, or pencil marks |
| S3-03 | Cyberpunk aesthetic | Neon lighting, high-tech environment, sci-fi fashion |
| S3-04 | Corporate luxury | Immaculate clothing, executive presence, not street punk |
| S3-05 | Dominant and imposing | Cold authority, not friendly |
| S3-06 | Character fills frame | Character is 75% of image — environment doesn't eat the character |
| S3-07 | Neon rim lighting | Visible rim light in discipline color |

#### Stack 04: Mobile — Colored pencil on white paper

| ID | Check | Pass When |
|----|-------|-----------|
| S4-01 | Colored pencil | Visible pencil strokes, hatching, crosshatching, waxy texture |
| S4-02 | White paper background | White/off-white, not dark |
| S4-03 | NOT paint/3D/photo | No brushstrokes, wet media, 3D render, or photograph |
| S4-04 | NOT ballpoint | No ink texture, no scribble shading — pencil hatching only |
| S4-05 | Elvish fantasy | Pointed ears, flowing robes, organic architecture |
| S4-06 | Noble and dignified | Serene power, not aggressive |
| S4-07 | Monochrome in discipline color | Entire drawing uses shades of one color only |

**Stack 04 special:** Disciplines 02 (Firebase) and 04 (Platform) are humans with ROUND ears. All others are elves with pointed ears.

### Discipline Checks

After global and stack checks pass, verify the character matches its discipline spec.

| Check | Pass When |
|-------|-----------|
| Correct color | Image is dominated by the discipline's specific color |
| Correct character | Gender, build, hair, features match the table below |
| Props visible | Listed props are present and identifiable |
| Environment matches | Background matches the listed environment |
| No cross-contamination | No elements from other disciplines leaked in |

---

## Per-Stack Discipline Tables

### Stack 01: Generic

| Disc | Color | Head Object | Body | Environment |
|------|-------|-------------|------|-------------|
| 00 Implementation | Blue | Blue steel hammer | Stocky man, blue coverall jumpsuit | Blue-tinted bathroom |
| 01 Refactoring | Violet | Violet vernier caliper | Tall thin woman, violet pencil skirt suit | Violet-tinted laundromat |
| 02 Investigation | Green | Green magnifying glass | Short stocky man, green tweed vest, green corduroy pants | Green-tinted hotel lobby |
| 03 Testing | Amber | Golden brass hourglass | Tall thin woman, amber turtleneck, amber corduroy pants | Amber-tinted barbershop |
| 04 Architecture | Indigo | Indigo drafting compass | Tall broad man, indigo three-piece suit | Indigo-tinted elevator |
| 05 DevOps | Cyan | Cyan toy rocket | Young skinny man, cyan overalls, cyan t-shirt | Cyan-tinted bakery |
| 06 Security | Red | Red brass padlock | Broad heavyset woman, red blazer, red pants | Red-tinted flower shop |
| 07 Documentation | Teal | Teal open hardcover book | Elderly thin man, teal cardigan, teal shirt, teal pants | Teal-tinted bowling alley |

### Stack 02: Desktop

| Disc | Color | Character | Props | Environment |
|------|-------|-----------|-------|-------------|
| 00 Frontend | Teal #00d9ff | Lean young man, dirty blond hair | Teal tool belt, screwdrivers, wire strippers, soldering iron, dog tags | Teal workshop, teal CRT monitor |
| 01 Backend | Orange | Strong Black male, high-top fade | Orange flannel, orange pager, orange safety glasses | Orange server room, orange mainframes |
| 02 Data | Violet | Japanese male, silver temples | Violet glasses, violet dress shirt, purple tie, purple file folders | Violet data lab, purple CRT terminal |
| 03 Integration | Emerald | Lean female, curly brown hair, green scrunchie | Green Members Only jacket, green rotary phone with coiled cord | Green telephone exchange, green patch cables |
| 04 Platform | Steel-blue | Broad heavyset male, dirty blond mullet | Blue mechanic jumpsuit, steel-toed boots, massive blue wrench | Blue auto garage, blue tool pegboards |
| 05 Quality | Gold | Petite female, permed brown hair, gold barrettes | Gold blazer with padded shoulders, gold clipboard, gold pencil | Gold inspection room, gold certificates |
| 06 Security | Crimson | Sharp angular male, slicked-back dark hair | Crimson leather jacket, red aviator sunglasses, red padlock on chain | Red bank vault, red steel doors |
| 07 Documentation | Cream/Ivory | Tall thin older male, balding, gray hair ring | Cream reading glasses, cream cardigan with elbow patches, ivory book | Cream library, ivory bookshelves |

### Stack 03: SaaS

| Disc | Color | Character | Props | Environment |
|------|-------|-----------|-------|-------------|
| 00 Next.js App Router | Electric blue #3b82f6 | Korean female, mid 20s, long straight black hair with blue streaks | Blue trenchcoat, blue visor, glowing blue archway door | Blue transit hub, blue archway gates |
| 01 React Components | Magenta #ec4899 | Brazilian male, early 30s, wavy dark hair, stubble | Magenta vest, magenta earpiece, floating magenta UI panel | Magenta component assembly hall |
| 02 API Routes | Amber #f59e0b | Ethiopian female, late 20s, short natural afro | Amber jacket with golden circuit traces, gold wrist guards, amber data cube | Amber gateway checkpoint |
| 03 Database | Emerald #10b981 | Russian male, mid 40s, silver slicked-back hair, silver beard | Emerald coat with green patterns, green rings, glowing emerald crystal | Green crystalline data vault |
| 04 Authentication | Crimson #ef4444 | Japanese non-binary, early 30s, asymmetric black bob with crimson streak | Crimson jacket, red monocle, crimson keycard badge | Red access control chamber |
| 05 Monorepo & Deploy | Silver #94a3b8 | Nigerian male, late 30s, short tight curls, broad build, goatee | Chrome silver longcoat, silver chain necklace, chrome launch lever | Silver command deck, chrome pipeline tubes |
| 06 Testing | Orange #f97316 | Indian female, mid 30s, long dark braid | Orange tactical jacket, orange scanner lines, orange augmented lens | Orange analysis laboratory |
| 07 Documentation | Ivory #f1f5f9 | Indigenous Australian male, late 50s, long gray dreadlocks, gray beard | Ivory ceremonial longcoat, ivory glasses, ivory stylus, ivory tome | Ivory archive cathedral |

### Stack 04: Mobile

| Disc | Color | Character | Props | Environment |
|------|-------|-----------|-------|-------------|
| 00 Flutter UI | Silver #c0c0c0 | Elvish female, early 30s, silver-white hair, tall willowy | Moonsilver robe, silver circlet, glowing silver threads | Silver weaving pavilion, silver tapestries |
| 01 Dart Logic | Sapphire #1e3a8a | Elvish male, mid 40s, dark hair with sapphire streaks, lean angular | Sapphire tunic, blue leather bracers, sapphire pendant, blue crystal tablet | Blue elvish study, blue runic diagrams |
| 02 Firebase Backend | Ember orange #f97316 | **Human female** (round ears), copper-red warrior braid, athletic | Orange tunic with flame embroidery, copper belt, living flame in palm | Orange elvish hearthhall |
| 03 State Management | Aquamarine #06b6d4 | Elvish non-binary, mid 30s, dark hair with aquamarine streaks | Aquamarine robe with wave embroidery, turquoise light from palms | Aquamarine meditation grotto |
| 04 Platform Integration | Forest green #16a34a | **Human male** (round ears), locs with green ornaments, tall broad muscular | Forest green cloak with vine embroidery, leather armor, jade brooch, scimitar | Green elvish bridge chamber |
| 05 Testing | Amethyst #9333ea | Elvish female, early 40s, short silver hair, petite precise | Amethyst robe, amethyst pendant, faceted amethyst prism | Purple chamber of truth, amethyst crystal walls |
| 06 App Distribution | Gold #eab308 | Elvish male, early 50s, long golden hair with white streaks, golden beard | Gold ceremonial tunic, golden arm bands, gold signet ring, golden moths | Gold herald tower, golden dispatch windows |
| 07 Documentation | Ivory #fefce8 | Elvish male, late 60s, very long white hair, tall gaunt, long white beard | Ivory ceremonial robe, pearl spectacles, ivory quill, ivory tome | Ivory elvish great library |

---

## Composite Checks (C01-C10)

Run AFTER all 8 in a stack are individually accepted. Composite failures are framing issues in the SOURCE images, not compositor bugs.

| ID | Check | Pass When |
|----|-------|-----------|
| C01 | Same medium | All 8 look like the same medium |
| C02 | Same lighting direction | All 8 lit from upper left |
| C03 | Same camera angle | All 8 at eye level straight on |
| C04 | Same body scale | All 8 characters roughly same % of frame height |
| C05 | Same ground plane | Feet all at ~90% Y position |
| C06 | Same eye line | Eyes/heads all at ~15% Y position |
| C07 | Same background value | No jarring bright/dark mismatches between adjacent characters |
| C08 | Similar silhouette width | No character drastically wider than others |
| C09 | Distinct colors | At thumbnail, all 8 colors distinguishable |
| C10 | All upright | No character leaning while others stand straight |

**Common composite fix patterns:**
- Character too small → Discipline framing wrong, regenerate with stronger framing
- Character offset up/down → Eye line (G02) or ankle line (G03) wrong
- Character too wide → Arms spread (G08 violation)
- Lighting different → Key light not upper left (G07)
- Style inconsistency → Add style-blocking negatives to stack ABOUT.yaml

---

## Sandbox Warning

Image generation requires ComfyUI network access. The Claude Code sandbox blocks network with `--unshare-net`. If generation fails with "Connection refused", disable the sandbox in Claude Code settings.

## Reference

- `.docs/040_DISCIPLINE_PORTRAIT_PROMPT_METHODOLOGY.md` — Deep dive on the three-layer prompt system
