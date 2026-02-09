# Image Review Checklist

Every generated discipline image must pass ALL applicable checks before being accepted.

---

## AGENT WORKFLOW: Generate and Iterate Until All Pass

This section describes how an agent should use this checklist to generate all images and iterate until every single one meets requirements.

### Overview

Process one stack at a time. Within each stack, process one discipline at a time. For each discipline, generate the image, review it against the checklist, and if it fails any check, fix the prompt and regenerate. Only move to the next discipline once the current one passes all checks.

### CRITICAL LESSONS LEARNED

1. **NEVER use `--test` for review** - It only does 1 step and produces blurry abstract results. Always use `--dev` (14 steps for Generic stack) for iteration.

2. **Commands are in justfile** - Use `just gen-image <STACK> <DISCIPLINE> [--dev|--prod]`
   - Stack numbers: 01 (Generic), 02 (Desktop), 03 (SaaS), 04 (Mobile)
   - Discipline numbers: 00-07
   - Example: `just gen-image 01 00 --dev`

3. **Compositor script location** - `crates/predefined-disciplines/compose_stack.py <STACK>`
   - Run AFTER all 8 prod images are accepted
   - Creates composite image showing all 8 disciplines side-by-side

4. **G12/G13 are the hardest checks** - "Color DOMINATES" and "Monochromatic read" require the discipline color to completely drench the image. If you see ANY other colors (tan, beige, white walls), it FAILS. Add color mentions to EVERY element: "blue walls, blue floor, blue ceiling, blue fixtures, blue lighting, blue shadows"

5. **Negative prompts matter** - Add unwanted colors to negatives: "tan walls, beige walls, white walls, brown, orange, warm colors, multi-colored"

6. **Workflow is: dev → iterate → prod → iterate → composite**
   - First pass: Generate all 8 at --dev quality, iterate until each passes
   - Second pass: Regenerate all 8 at --prod quality (28 steps, higher res)
   - Prod images may differ from dev, so review prod images too
   - If prod fails, iterate at prod quality
   - Final: Run compositor on the 8 accepted prod images

7. **CRITICAL: Sandbox blocks ComfyUI access** - The Claude Code sandbox uses `--unshare-net` which blocks network access to ComfyUI. The sandbox MUST be disabled for image generation to work. The preflight check in `generate_discipline_image.rs` now detects sandbox mode (via `SANDBOX_RUNTIME` env var) and skips the check when sandboxed. When running outside the sandbox, it will properly validate ComfyUI availability before attempting generation. If generation fails with "Connection refused" and you see the sandbox skip message, disable the sandbox in Claude Code settings.

### Step 1: Pick a Stack

Process stacks in order: 01 Generic, 02 Desktop, 03 SaaS, 04 Mobile.

### Step 2: Generate One Discipline Image

Run: `just gen-image <STACK> <DISCIPLINE> --dev`

**CRITICAL:** Always use `--dev` for iteration (14 steps for Generic). NEVER use `--test` (1 step produces unusable blurry abstract images). Only use `--prod` after ALL 8 disciplines pass at dev quality.

### Step 3: Review the Image

Open the generated PNG and check it against ALL of these in order:

1. **Global checks G01-G14** — framing, camera, lighting, pose, ground, color, face
2. **Stack checks** — medium, style, tone (use the stack-specific table)
3. **Discipline checks** — color match, character match, props visible, environment match

For each check, record PASS or FAIL with a brief note.

### Step 4: If Any Check Fails — Diagnose and Fix

Identify which prompt layer is responsible for the failure:

| Failure Type | Fix Location |
|---|---|
| Framing wrong (too small, cropped, wrong angle) | `image_prompts.yaml` global positive/negative |
| Medium wrong (3D instead of watercolor, photo instead of pencil) | Stack `ABOUT.yaml` image_prompt positive/negative |
| Tone wrong (too happy, too creepy, wrong era) | Stack `ABOUT.yaml` image_prompt positive/negative |
| Color not dominant enough | Discipline YAML image_prompt positive (add more color mentions) |
| Wrong character (wrong gender, hair, body type) | Discipline YAML image_prompt positive |
| Props missing or blocking body | Discipline YAML image_prompt positive |
| Environment wrong | Discipline YAML image_prompt positive |
| Unwanted element appearing | Add to the appropriate negative prompt (discipline, stack, or global) |

**Prompt editing rules:**
- Positive prompts describe ONLY what you WANT to see. Never use "no X" or "not Y" in positive prompts.
- Negative prompts list ONLY what you DO NOT want. Put unwanted elements here.
- If something keeps appearing despite being in negatives, strengthen the positive description of what should be there instead.
- If the discipline color isn't dominant enough, add more `[COLOR]` mentions to objects in the positive prompt.

### Step 5: Regenerate and Re-review

After editing the prompt, regenerate with `just gen-image <STACK> <DISCIPLINE> --dev` and go back to Step 3.

Repeat until ALL checks pass. Maximum 5 attempts per discipline — if it still fails after 5, flag it and move on.

### Step 6: Accept and Move to Next Discipline

Once a discipline passes all checks, record it as accepted and move to the next discipline in the stack (00, 01, 02, ... 07).

### Step 7: Composite Review After Full Stack

After all 8 disciplines in a stack are accepted, review them together using the **Composite Consistency Checks (C01-C10)**.

If any composite check fails (e.g., one character is much smaller than the others, or one has a different lighting direction), go back and regenerate just the failing discipline(s).

### Step 8: Final Prod Generation

Once all 8 pass both individual and composite checks at dev quality, regenerate ALL 8 at prod quality:

```
just gen-image <STACK> 00 --prod
just gen-image <STACK> 01 --prod
just gen-image <STACK> 02 --prod
just gen-image <STACK> 03 --prod
just gen-image <STACK> 04 --prod
just gen-image <STACK> 05 --prod
just gen-image <STACK> 06 --prod
just gen-image <STACK> 07 --prod
```

Do a final review of the prod images. Prod uses more steps and higher resolution, so results may differ slightly from dev. If any prod image fails, iterate on it at prod quality.

### Step 9: Compose the Stack

After all 8 prod images are accepted, run the compositor:

```bash
python crates/predefined-disciplines/compose_stack.py 01  # For Stack 01 Generic
python crates/predefined-disciplines/compose_stack.py 02  # For Stack 02 Desktop
python crates/predefined-disciplines/compose_stack.py 03  # For Stack 03 SaaS
python crates/predefined-disciplines/compose_stack.py 04  # For Stack 04 Mobile
```

This creates a side-by-side composite showing all 8 disciplines in the stack. Review the composite image and debug overlay. If alignment is off, the issue is in the individual images (framing), not the compositor.

### Step 10: Repeat for Next Stack

Go back to Step 1 and pick the next stack.

### Tracking Template

Use this template to track progress for each discipline:

```
## Stack XX: [Name]

| Disc | Attempt | G01-G14 | Stack | Discipline | Status |
|------|---------|---------|-------|------------|--------|
| 00   | 1       |         |       |            |        |
| 01   | 1       |         |       |            |        |
| 02   | 1       |         |       |            |        |
| 03   | 1       |         |       |            |        |
| 04   | 1       |         |       |            |        |
| 05   | 1       |         |       |            |        |
| 06   | 1       |         |       |            |        |
| 07   | 1       |         |       |            |        |
| COMPOSITE | - |   |       |            |        |
```

Fill in PASS/FAIL for each column. When all show PASS, the stack is done.

---

## GLOBAL CHECKS (every image, every stack)

These come from `image_prompts.yaml` — the base layer concatenated into every prompt.

| ID | Category | Criteria | Pass Condition |
|----|----------|----------|----------------|
| G01 | Framing | Full body visible | Head/top and both feet fully in frame, nothing cropped |
| G02 | Framing | Eye line ~15% | Eyes (or head-object center) at roughly 15% from top |
| G03 | Framing | Ankle line ~90% | Feet/ankles at roughly 90% from top |
| G04 | Framing | Body fills ~75% | Character body occupies approximately 75% of image height |
| G05 | Framing | Single person centered | One character, horizontally centered in frame |
| G06 | Camera | Eye level straight on | Camera at eye height, not looking up or down at character |
| G07 | Lighting | Key light upper left | Highlight on character's left side, shadow falls to lower right |
| G08 | Pose | Arms within shoulder width | Compact silhouette, arms close to body, not spread wide |
| G09 | Pose | Standing upright | Not leaning, tilted, hunched, or crouching |
| G10 | Ground | Feet on visible floor | Clear ground plane, feet planted, not floating |
| G11 | Artifacts | Clean image | No text, watermarks, extra limbs, deformed features, jpeg artifacts |
| G12 | Color | Discipline color DOMINATES | Entire image drenched in the discipline color — clothing, props, lighting, background |
| G13 | Color | Monochromatic read | At thumbnail size (64px), character reads as a single color blob |
| G14 | Face | Face direction | Facing viewer, head up, eyes forward (except Generic — no face) |

### GLOBAL KNOWN CONFLICTS

The global prompt says "dark moody background" but:
- **Generic** needs pastel environments (Wes Anderson aesthetic) — stack overrides global
- **Mobile** needs white paper background (pencil on paper) — stack overrides global

These are intentional overrides. When reviewing, use the STACK background rule, not global.

---

## STACK 01: GENERIC

**Medium:** Wes Anderson style photograph, photorealistic
**Style:** Perfectly symmetrical, pastel environment, deadpan absurdist humor
**Special rule:** Characters have NO HUMAN FACE — head is replaced by an object
**Background:** Pastel-colored interior environment (NOT dark)

| ID | Category | Criteria | Pass Condition |
|----|----------|----------|----------------|
| S1-01 | Medium | Photorealistic Wes Anderson | Looks like a photograph, not illustration/painting/3D |
| S1-02 | Style | Perfect symmetry | Room and composition are obsessively symmetrical |
| S1-03 | Style | Pastel environment | Background is a real pastel-colored interior (bathroom, bakery, etc.) |
| S1-04 | Style | Deadpan humor | Character is comically out of place but standing still like they belong |
| S1-05 | Face | NO human face | Head is entirely replaced by the discipline object — no eyes, nose, mouth visible |
| S1-06 | Style | Warm vintage film look | Slightly desaturated, warm tones, vintage film stock feel |

### Stack 01 Disciplines

| Disc | Color | Head Object | Body | Environment | Key Negatives |
|------|-------|-------------|------|-------------|---------------|
| 00 Implementation | Blue | Blue steel hammer | Stocky man, blue coverall jumpsuit | Blue-tinted bathroom | human face, human head |
| 01 Refactoring | Violet | Violet vernier caliper | Tall thin woman, violet pencil skirt suit | Violet-tinted laundromat | human face, normal head |
| 02 Investigation | Green | Green magnifying glass | Short stocky man, green tweed vest, green corduroy pants | Green-tinted hotel lobby | human face, normal head, human head |
| 03 Testing | Amber | Golden brass hourglass | Tall thin woman, amber turtleneck, amber corduroy pants | Amber-tinted barbershop | human face, normal head, human head |
| 04 Architecture | Indigo | Indigo drafting compass | Tall broad man, indigo three-piece suit | Indigo-tinted elevator | human face, normal head |
| 05 DevOps | Cyan | Cyan toy rocket | Young skinny man, cyan overalls, cyan t-shirt | Cyan-tinted bakery | human face, normal head |
| 06 Security | Red | Red brass padlock | Broad heavyset woman, red blazer, red pants | Red-tinted flower shop | human face, normal head |
| 07 Documentation | Teal | Teal open hardcover book | Elderly thin man, teal cardigan, teal shirt, teal pants | Teal-tinted bowling alley | human face, normal head |

---

## STACK 02: DESKTOP

**Medium:** Watercolor illustration
**Style:** Loose expressive brushstrokes, wet on wet, bleeding edges, visible paper texture, paint dripping
**Character archetype:** 1980s blue-collar heroes — determined, serious, working class
**Background:** Dark moody (global default applies)

| ID | Category | Criteria | Pass Condition |
|----|----------|----------|----------------|
| S2-01 | Medium | Watercolor | Visible brushstrokes, paper texture, bleeding watercolor edges, paint dripping |
| S2-02 | Medium | NOT photorealistic | Clearly an illustration, not a photo or 3D render |
| S2-03 | Medium | NOT digital art | No cel-shading, no flat colors, no thick outlines |
| S2-04 | Tone | Serious determined | Not smiling, not playful, not cheerful — stoic blue-collar hero |
| S2-05 | Tone | NOT horror | Not creepy, not sinister, not menacing — heroic and powerful |
| S2-06 | Era | 1980s aesthetic | Clothing, props, environment read as 1980s |
| S2-07 | Face | Face visible and lit | Facial features clearly readable, not in shadow |

### Stack 02 Disciplines

| Disc | Color | Character | Props | Environment |
|------|-------|-----------|-------|-------------|
| 00 Frontend | Teal #00d9ff | Lean young man, dirty blond hair | Teal tool belt, screwdrivers, wire strippers, soldering iron, dog tags | Teal workshop, teal CRT monitor, teal wiring |
| 01 Backend | Orange | Strong Black male, high-top fade | Orange flannel, orange pager, orange safety glasses | Orange server room, orange mainframe cabinets |
| 02 Data | Violet | Japanese male, silver temples | Violet glasses, violet dress shirt, purple tie, purple file folders, purple pen | Violet data lab, purple CRT terminal, violet dot-matrix printer |
| 03 Integration | Emerald green | Lean female, curly brown hair, green scrunchie | Green Members Only jacket, green rotary phone with coiled cord | Green telephone exchange, green patch cables, green switchboard |
| 04 Platform | Steel-blue | Broad heavyset male, dirty blond mullet | Blue mechanic jumpsuit, steel-toed boots, massive blue wrench | Blue auto garage, blue tool pegboards, blue computer tower |
| 05 Quality | Gold | Petite female, permed brown hair, gold barrettes | Gold blazer with padded shoulders, gold clipboard, gold pencil | Gold inspection room, gold certificates, gold rubber stamps |
| 06 Security | Crimson red | Sharp angular male, slicked-back dark hair | Crimson leather jacket, red aviator sunglasses, red padlock on chain | Red bank vault, red steel doors, red combination dials |
| 07 Documentation | Cream/Ivory | Tall thin older male, balding, gray hair ring | Cream reading glasses, cream cardigan with elbow patches, ivory hardcover book | Cream library, ivory bookshelves, cream reading lamps |

---

## STACK 03: SAAS

**Medium:** Stylized 3D character render, cyberpunk aesthetic
**Style:** Unity quality, high-poly mesh, neon rim lighting, PBR with emissive glow
**Character archetype:** Corporate cyberpunk executives — dominant, imposing, cold authority
**Background:** Dark cyberpunk environments with neon glow

| ID | Category | Criteria | Pass Condition |
|----|----------|----------|----------------|
| S3-01 | Medium | 3D render | Clearly a 3D character, smooth mesh, not a photo or painting |
| S3-02 | Medium | NOT watercolor/pencil/painting | No paper texture, no brushstrokes, no pencil marks |
| S3-03 | Style | Cyberpunk aesthetic | Neon lighting, high-tech environment, sci-fi fashion |
| S3-04 | Style | Corporate luxury | Immaculate clothing, executive presence, not street punk or grungy |
| S3-05 | Tone | Dominant and imposing | Cold authority, not friendly, not welcoming |
| S3-06 | Framing | Character fills frame | Character must be 75% of image — environment must NOT eat the character |
| S3-07 | Glow | Neon rim lighting | Character has visible neon rim light in the discipline color |

### Stack 03 Disciplines

| Disc | Color | Character | Props | Environment |
|------|-------|-----------|-------|-------------|
| 00 Next.js App Router | Electric blue #3b82f6 | Korean female, mid 20s, long straight black hair with blue streaks | Blue trenchcoat, blue visor, glowing blue archway door, blue holographic floor plan | Blue transit hub, blue archway gates |
| 01 React Components | Magenta #ec4899 | Brazilian male, early 30s, wavy dark hair, stubble | Magenta vest, magenta earpiece, floating magenta UI panel | Magenta component assembly hall, magenta holographic panels |
| 02 API Routes | Amber #f59e0b | Ethiopian female, late 20s, short natural afro | Amber jacket with golden circuit traces, gold wrist guards, amber data cube | Amber gateway checkpoint, golden arch gates |
| 03 Database | Emerald #10b981 | Russian male, mid 40s, silver slicked-back hair, silver beard | Emerald green coat with green patterns, green rings, glowing emerald crystal | Green crystalline data vault, emerald crystal columns |
| 04 Authentication | Crimson #ef4444 | Japanese non-binary, early 30s, asymmetric black bob with crimson streak | Crimson jacket, red monocle, crimson keycard badge, palm-out blocking gesture | Red access control chamber, red security gates |
| 05 Monorepo & Deploy | Silver #94a3b8 | Nigerian male, late 30s, short tight curls, broad build, goatee | Chrome silver longcoat, silver chain necklace, chrome launch lever | Silver command deck, chrome pipeline tubes, silver launch rails |
| 06 Testing | Orange #f97316 | Indian female, mid 30s, long dark braid | Orange tactical jacket, orange scanner lines, orange augmented lens, orange tablet | Orange analysis laboratory, orange diagnostic monitors |
| 07 Documentation | Ivory #f1f5f9 | Indigenous Australian male, late 50s, long gray dreadlocks, gray beard | Ivory ceremonial longcoat, ivory glasses, ivory stylus, ivory tome | Ivory archive cathedral, ivory shelves |

---

## STACK 04: MOBILE

**Medium:** Colored pencil illustration
**Style:** Dense pencil hatching, layered shading, visible pencil strokes, crosshatching, pressure variation
**Character archetype:** High elvish fantasy — noble, wise, disciplined
**Background:** White paper (NOT dark)

| ID | Category | Criteria | Pass Condition |
|----|----------|----------|----------------|
| S4-01 | Medium | Colored pencil | Visible pencil strokes, hatching, crosshatching, waxy texture |
| S4-02 | Medium | On white paper | White/off-white paper background, not dark environment |
| S4-03 | Medium | NOT paint/3D/photo | No brushstrokes, no wet media, no 3D render, no photograph |
| S4-04 | Medium | NOT ballpoint | No ink texture, no scribble shading — pencil hatching only |
| S4-05 | Style | Elvish fantasy | Pointed ears, flowing robes, organic architecture, starlight |
| S4-06 | Style | Noble and dignified | Not aggressive, not menacing — serene power |
| S4-07 | Color | Monochrome in discipline color | Entire drawing uses only shades of one color, like a single colored pencil |

### Stack 04 Disciplines

| Disc | Color | Character | Props | Environment |
|------|-------|-----------|-------|-------------|
| 00 Flutter UI | Silver #c0c0c0 | Elvish female, early 30s, silver-white hair, tall willowy | Moonsilver robe, silver circlet, glowing silver threads in both hands | Silver weaving pavilion, silver tapestries, silver lattice archways |
| 01 Dart Logic | Sapphire blue #1e3a8a | Elvish male, mid 40s, dark hair with sapphire streaks, lean angular | Sapphire blue tunic, blue leather bracers, sapphire pendant, sapphire stylus, blue crystal tablet | Blue elvish study, blue crystal tablets, blue runic diagrams |
| 02 Firebase Backend | Ember orange #f97316 | Nord human female (round ears), late 20s, copper-red hair in warrior braid, athletic | Orange tunic with flame embroidery, copper belt, amber earrings, orange leather bracers, living flame in palm | Orange elvish hearthhall, orange hearth, orange pillars |
| 03 State Management | Aquamarine #06b6d4 | Elvish non-binary, mid 30s, dark hair with aquamarine streaks, medium serene build | Aquamarine robe with wave embroidery, aquamarine bracelet, turquoise light ripples from palms | Aquamarine meditation grotto, aquamarine stalactites, aquamarine water channels |
| 04 Platform Integration | Forest green #16a34a | Redguard human male (round ears), late 30s, locs with green ornaments, tall broad muscular | Forest green cloak with vine embroidery, leather armor, green silk sash, jade brooch, scimitar | Green elvish bridge chamber, living-wood gateway arches, green luminous leaves |
| 05 Testing | Amethyst purple #9333ea | Elvish female, early 40s, short silver hair, petite precise | Amethyst purple robe, amethyst pendant, faceted amethyst prism held to light | Purple chamber of truth, amethyst crystal walls, purple truth-mirrors |
| 06 App Distribution | Gold #eab308 | Elvish male, early 50s, long golden hair with white streaks, tall regal, golden beard | Gold ceremonial tunic with sun-ray embroidery, golden arm bands, gold signet ring, golden messenger moths | Gold herald tower, golden dispatch windows, golden horn instruments |
| 07 Documentation | Ivory #fefce8 | Elvish male, late 60s, very long white hair, tall gaunt, long white beard | Ivory ceremonial robe, pearl spectacles, ivory quill, ivory tome | Ivory elvish great library, ivory bookshelves, ivory flowering vines |

### Stack 04 Special Notes

- Disciplines 02 (Firebase) and 04 (Platform) are **humans among elves** — they must have **round ears**, NOT pointed ears
- All other mobile disciplines are elvish with pointed ears

---

## COMPOSITE CONSISTENCY CHECKS

After all 8 disciplines in a stack are generated, review the set together:

| ID | Category | Criteria | Pass Condition |
|----|----------|----------|----------------|
| C01 | Style | Same medium | All 8 look like the same medium (all watercolor, all 3D, all pencil) |
| C02 | Lighting | Same direction | All 8 lit from upper left with shadow to lower right |
| C03 | Camera | Same angle | All 8 at eye level straight on — no mixed angles |
| C04 | Scale | Same body scale | All 8 characters roughly the same percentage of frame height |
| C05 | Ground | Same ground plane | Feet all at roughly the same Y position (~90%) |
| C06 | Eyes | Same eye line | Eyes/head-objects all at roughly the same Y position (~15%) |
| C07 | Background | Same value range | No jarring bright/dark mismatches between adjacent characters |
| C08 | Silhouette | Similar width | No character drastically wider than others (arms spread, etc.) |
| C09 | Color | Each color distinct | At thumbnail size, all 8 colors are distinguishable from each other |
| C10 | Pose | All upright | No character leaning while others stand straight |
