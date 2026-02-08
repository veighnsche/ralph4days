# 040 - Discipline Portrait Prompt Methodology

## Summary

Complete methodology for generating consistent, high-quality discipline character portraits using ComfyUI with Stable Diffusion. Developed iteratively across ~40 generation cycles on Stack 02 (Desktop) and Stack 03 (SaaS). SaaS stack is the most complete reference implementation with all 8 disciplines generated in consistent cel-shaded cyberpunk style.

## Three-Layer Prompt System

Prompts are concatenated: `global + stack + discipline` for both positive and negative.

### Layer 1: Global (`image_prompts.yaml`)
**Scope:** Universal image quality. Applies to ALL stacks, ALL disciplines.
**Contains:** ONLY technical image quality terms. Nothing about style, subject, or content.

```yaml
global:
  positive: |
    Professional polished illustration, shallow depth of field, studio lighting, clean composition.
  negative: |
    blurry, noise, jpeg artifacts, text, watermark, logo, multiple panels, collage, low resolution
```

**Rule:** If it describes the subject, the style, or the setting — it does NOT belong here.

### Layer 2: Stack (`ABOUT.yaml` → `image_prompt`)
**Scope:** Shared visual identity for all 8 disciplines within a stack.
**Contains:** Art style, framing, composition, setting mood, shared aesthetic.

This is the **consistency layer** — it locks the look so all 8 portraits feel like they belong together.

**What belongs here:**
- **Precise art style** (not "illustration" — say exactly: "flat cel-shaded style with bold clean outlines, sharp defined edges, minimal texture, smooth flat color fills")
- Camera framing (three-quarter portrait, waist up, centered, etc.)
- Subject positioning ("subject occupies lower two-thirds")
- Overall aesthetic/mood (cyberpunk, medieval, corporate, etc.)
- Shared clothing style language (immaculate, sleek, absurdly luxurious, etc.)
- Dominant posture language ("dominant imposing stance")
- Environment framing ("inside a dark imposing environment")
- Shared negatives for composition (no multiple people, no photo-realism, etc.)
- Posture energy (dominant via negatives: smiling, friendly, warm, gentle, peaceful, etc.)
- **Style drift prevention negatives** (anime, manga, watercolor, oil painting, painterly brushstrokes, sketch, pencil, impressionist, abstract)

**What does NOT belong here:**
- Any specific discipline color
- Any character-specific details (gender, ethnicity, clothing items)
- Background details specific to one discipline
- Specific actions or poses per character

**Reference Implementation (SaaS Cyberpunk):**
```yaml
image_prompt:
  positive: |
    Digital illustration in flat cel-shaded style with bold clean outlines, sharp defined
    edges, minimal texture, smooth flat color fills with subtle gradients, three-quarter
    portrait from waist up, single person centered in frame, subject occupies lower
    two-thirds, dark sleek corporate cyberpunk aesthetic, polished obsidian and glass
    surfaces, cold ambient lighting, immaculate high-tech executive style, neon accent
    lighting, volumetric fog, dramatic cinematic composition, absurdly luxurious sci-fi
    fashion, dominant imposing stance, inside a dark imposing environment, consistent
    thick linework, no painterly brushstrokes.
  negative: |
    multiple characters, multiple people, crowd, group, two people, photograph,
    photorealistic, realistic skin texture, superhero, weapon, text, words, letters,
    typography, action figure, 3D render, plastic, military, writing, caption, title,
    worn clothing, patches, duct tape, dirty, grungy, street punk, smiling, friendly,
    warm, gentle, soft, relaxed, casual, peaceful, kind eyes, open smile, welcoming,
    empty background, solid color background, plain backdrop, photo studio, shooting
    energy, magic powers, combat, action pose, anime, manga, chibi, watercolor, oil
    painting, painterly brushstrokes, soft edges, sketch, pencil, rough texture, grainy,
    impressionist, abstract
```

### Layer 3: Discipline (per-discipline YAML → `image_prompt`)
**Scope:** What makes THIS character unique.
**Contains:** Character description, discipline color, clothing, pose, action, physical environment.

**What belongs here:**
- Character demographics (gender, ethnicity, age, hair, build, facial hair)
- Discipline accent color (mentioned 6-8 times across clothing, lighting, environment, effects)
- Specific clothing items with color AND detailed material/trim descriptions
- Specific physical action with a concrete object (not just a pose)
- Discipline-specific cyberpunk feature (holographic wireframes, circuit tattoos, etc.)
- Specific physical environment (server vault, design studio, library, etc.)
- Discipline-specific negatives (what would make THIS character uncool)

## Color Dominance Rule

Each discipline has an accent color. That color MUST appear multiple times throughout the prompt in different forms:

1. **Clothing** — the main garment is the color
2. **Clothing accent** — secondary detail (piping, trim, seam detailing)
3. **Accessory** — glasses, earrings, wrist guards, rings in tinted color
4. **Cyberpunk feature** — the hologram/projection/tattoo glows in the color
5. **Rim lighting** — "dramatic [color] rim lighting"
6. **Ambient glow** — "[color] ambient glow"
7. **Environment** — "bathed in [color] light"
8. **Environment details** — specific background elements glow in color variants

**Example (API Routes — amber/gold):**
> black jacket with glowing golden circuit traces, amber-lit shoulder epaulettes, gold-tinted wrist guards with amber status indicators, amber rim lighting, warm gold ambient glow, bathed in amber light, massive golden arch gates, streams of gold data packets

This ensures the discipline's color DOMINATES the image regardless of what the model decides to do with the dark background.

## Physical Actions (SOLVED)

**Problem was:** Characters were just posing — looking cool but not doing anything. They need a physical action that communicates their discipline without going full supervillain.

**The sweet spot:** Physical interaction with a discipline-specific object. Not magic powers, not just standing. Doing something.

**Method:** Ask "if this discipline were a supervillain, what would their first attack be?" — then dial it back to a grounded physical action with a concrete object.

### SaaS Reference Implementation

| # | Discipline | Supervillain Energy | Grounded Action |
|---|-----------|-------------------|-----------------|
| 00 | App Router | Tears open portals | Gripping a massive archway door, holding a holographic floor plan |
| 01 | React Components | Summons component swarm | Hands raised like a conductor, repositioning floating UI panels |
| 02 | API Routes | Intercepts & transmutes | Holding a data cube between fingers, inspecting it, other hand on validation console |
| 03 | Database | Crystallizes chaos | Hand resting possessively on a crystal data column like a vault proprietor |
| 04 | Authentication | Revokes access | Holding up a keycard badge, other arm blocking path palm-out |
| 05 | Monorepo/Deploy | Parallel launch | Gripping a massive launch lever on a steel console |
| 06 | Testing | Reveals fault lines | Leaning forward, peering through augmented lens, finger tracing cracks on a surface |
| 07 | Documentation | Rewrites reality | Writing with a luminous stylus into a glowing tome |

**Key principle:** Each action involves a CONCRETE OBJECT (archway, panels, data cube, crystal column, keycard, lever, cracked surface, tome+stylus). Objects give the model something specific to render.

## Discipline-Specific Negatives (SOLVED)

**Problem was:** Generic negatives ("shooting energy, magic powers, action pose, combat") are lazy and don't protect what makes each character cool.

**Solution:** Each discipline's negatives describe what would make THAT specific character uncool — the opposite of their power fantasy.

| # | Discipline | What Makes Them Uncool |
|---|-----------|----------------------|
| 00 | App Router | Lost, confused, dead ends, closed doors, hesitant |
| 01 | React Components | Empty hands, nothing floating, bare room, bored, idle |
| 02 | API Routes | Waving things through unchecked, careless, distracted |
| 03 | Database | Messy chaos, crumbling structures, neglected dusty shelves |
| 04 | Authentication | Open unlocked gates, welcoming everyone in, no badge, passive |
| 05 | Monorepo/Deploy | Waiting, hesitating, broken machinery, hands off controls |
| 06 | Testing | Eyes closed, ignoring problems, rubber stamping approval |
| 07 | Documentation | Blank pages, closed book, empty shelves, confused by text |

## Visual Consistency Checklist

Before generating a full team, verify each discipline prompt has ALL of these at equal density:

- [ ] **Specific clothing** — not just "jacket" but "fitted magenta vest with glowing pink geometric accents over a black shirt"
- [ ] **Clothing trim/detail** — piping, seam detailing, epaulettes, buttons, clasps
- [ ] **Small accessory** — earpiece, wrist guards, rings, glasses, monocle, chain
- [ ] **Concrete action object** — something held, gripped, rested upon, written in
- [ ] **Action description** — what the hands are doing with the object
- [ ] **Color 6-8x** — clothing, accent, accessory, feature, rim lighting, ambient glow, environment "bathed in", environment details
- [ ] **Environment type** — "inside a dark [adjective] [specific place]"
- [ ] **Environment scale** — towering, massive, enormous, infinite, wall-sized
- [ ] **Environment details** — specific objects lining/filling the space (monitors, columns, shelves, pipelines)
- [ ] **Anti-uncool negatives** — what would ruin THIS character specifically

**If one discipline has less detail than another, the less-detailed one will come out vague/flat.** Every prompt must be equally dense.

## Team Diversity Grid

Each stack has 8 disciplines = 8 unique characters. Design them as a cohesive team with maximum diversity across these axes:

| Variable | Purpose | Examples |
|----------|---------|---------|
| Gender | 50/50 or close | Male, Female, Non-binary |
| Ethnicity | Max variety | East Asian, Black, South Asian, Latina, White, Middle Eastern, Mixed-race, Indigenous |
| Age | Range of decades | Early 20s through late 50s |
| Hair | Distinct silhouettes | Short styled, natural curls, braids, buzz cut, undercut, gray-streaked, dreadlocks, asymmetric bob |
| Build | Variety | Slim, athletic, stocky, broad, tall, petite, heavy, lean |
| Facial hair | Where applicable | Clean-shaven, trimmed beard, stubble, full beard, goatee |
| Accent color | Discipline identity | Electric blue, magenta, amber, emerald, crimson, silver, orange, white |
| Clothing | Same style language, different items | Trenchcoat, vest, jacket, longcoat, high-collar jacket, tactical jacket, ceremonial coat |
| Action | Distinct per person | Gripping door, conducting, inspecting object, resting hand possessively, blocking path, gripping lever, peering through lens, writing in tome |

**Rule:** No two characters should share more than 1-2 variables. The team should look like 8 completely different people who happen to share the same aesthetic.

### SaaS Team Grid (Reference)

| # | Discipline | Gender | Ethnicity | Age | Build | Color | Clothing |
|---|-----------|--------|-----------|-----|-------|-------|----------|
| 00 | App Router | F | Korean | Mid 20s | Tall lean | Electric blue | Trenchcoat |
| 01 | React Components | M | Brazilian | Early 30s | Medium | Magenta | Fitted vest |
| 02 | API Routes | F | Ethiopian | Late 20s | Athletic | Amber/gold | Tailored jacket |
| 03 | Database | M | Russian | Mid 40s | Heavy stocky | Emerald | Longcoat |
| 04 | Authentication | NB | Japanese | Early 30s | Slim | Crimson | High-collar jacket |
| 05 | Monorepo/Deploy | M | Nigerian | Late 30s | Broad powerful | Silver/chrome | Chrome longcoat |
| 06 | Testing | F | Indian | Mid 30s | Petite | Orange | Tactical jacket |
| 07 | Documentation | M | Indigenous Australian | Late 50s | Tall dignified | White/ivory | Ceremonial longcoat |

## Pose & Energy Design

### Dominant, Not Aggressive
Characters should project dominance and authority without explicit aggression. NOT supervillains shooting energy — executives who own their domain.

**In positive prompts (examples from SaaS):**
- "standing with chin raised" (App Router)
- "sharp evaluating stare" (React Components)
- "staring down the viewer with cold authority" (API Routes)
- "commanding downward gaze" (Database)
- "blocks the path forward palm-out" (Authentication)
- "surveying the deck with cold authority" (Monorepo/Deploy)
- "cold scrutinizing stare" (Testing)
- "standing imperiously" (Documentation)

**In stack-level negatives:**
- smiling, friendly, warm, gentle, soft, relaxed, casual, peaceful, kind eyes, open smile, welcoming

### Preventing Multiple People
Stack-level negatives MUST include: "multiple characters, multiple people, crowd, group, two people" — without this, backgrounds often generate bystanders.

## Physical Environment Design

Every discipline needs a SPECIFIC physical place — not a solid color backdrop or generic photo studio.

**Pattern:** "inside a dark [adjective] [place] bathed in [color] light with [large-scale environment details] behind [him/her/them]"

### SaaS Environments (Reference)

| # | Discipline | Environment |
|---|-----------|-------------|
| 00 | App Router | Dark towering transit hub with archway gates receding into distance |
| 01 | React Components | Dark vast component assembly hall with floating holographic panels |
| 02 | API Routes | Dark imposing gateway checkpoint hall with golden arch gates |
| 03 | Database | Dark cavernous crystalline data vault with emerald crystal columns in rows |
| 04 | Authentication | Dark imposing access control chamber with red-lit security gates |
| 05 | Monorepo/Deploy | Dark massive launch platform command deck with chrome pipeline tubes |
| 06 | Testing | Dark sprawling analysis laboratory with orange diagnostic monitors |
| 07 | Documentation | Dark impossibly vast archive cathedral with towering shelves of documents |

**Rule:** Environments should be LARGE and IMPOSING — "towering", "massive", "cavernous", "vast", "impossibly vast". The character dominates the foreground; the environment shows the scale of their domain.

## Stack Aesthetic Assignments

| Stack | Aesthetic | Status |
|-------|----------|--------|
| 01_generic | TBD | Not started |
| 02_desktop | **Medieval** | Prompts need rewrite — currently has cyberpunk placeholders |
| 03_saas | **Dark corporate cyberpunk (Arasaka Corp)** | COMPLETE — all 8 generated |
| 04_mobile | TBD | Not started |

## Technical Setup

- **Workflow:** 2-CLIP node workflow (`generate_discipline.json`) — one `__POSITIVE__` and one `__NEGATIVE__` CLIPTextEncode node
- **Default:** 14 steps, 1MP, 9:16 ratio (fast iteration)
- **Production:** `--prod` flag → 28 steps, 2MP (final quality)
- **Test:** `--test` flag → 1 step (pipeline validation)
- **Prompt concatenation:** `"{global} {stack} {discipline}"` for both positive and negative
- **Filename format:** `{NN}_{name}_{steps}_{WxH}_{base36_timestamp}.png`
- **Companion file:** `.txt` with full prompts saved alongside each `.png`
- **Command:** `just gen-image <stack> <discipline> [--prod] [--test] [--ratio W H] [--ratio-square] [--ratio-landscape] [--mp N]`

## Remaining Work

1. ~~Solve the physical mirror metaphor for SaaS~~ DONE — actions with concrete objects
2. ~~Build SaaS team grid~~ DONE — 8 diverse characters
3. ~~Generate SaaS portraits~~ DONE — all 8 in consistent cel-shaded style
4. Build Desktop team grid with medieval aesthetic (different disciplines than SaaS)
5. Solve medieval physical mirrors for Desktop disciplines
6. Add ControlNet to workflow for consistent character positioning across all 8 portraits
7. Generate final portraits with ControlNet pose normalization
8. Fill in non-image fields for SaaS disciplines (display_name, description, conventions, skills, system_prompt still TODO)

## Iteration Lessons Learned

### From Desktop iteration (early)
1. **Start with 1 sentence per prompt section** — long prompts produce abstract mush
2. **Lock art style at stack level** — prevents random style drift between disciplines
3. **Negative prompts are as important as positive** — peaceful/friendly in negatives creates dominance without aggression
4. **Color must be hammered** — mentioning a color once means the model might ignore it. 6-8 mentions across different elements locks it in
5. **"magazine cover" triggers text** — avoid publication-related terms
6. **"blurred X behind them" creates empty space** — use "inside a [place] with [things]" instead for rich environments
7. **Always test at default steps first** — 14 steps is enough to evaluate composition and color
8. **Each image costs ~10 seconds at default** — fast enough for rapid iteration
9. **Stack negatives for multiple people are essential** — without them, backgrounds spawn random figures
10. **"absurdly" is a great prompt modifier** — pushes details toward over-the-top fashion editorial without breaking the image

### From SaaS iteration (refined)
11. **Be EXTREMELY precise about art style** — "stylized illustration" is vague and causes drift. "Digital illustration in flat cel-shaded style with bold clean outlines, sharp defined edges, minimal texture" is precise and consistent
12. **Block style drift in negatives** — add anime, manga, watercolor, oil painting, painterly brushstrokes, sketch, pencil, impressionist to stack negatives
13. **Supervillain thought experiment works** — ask "what's their first attack?" then dial back to a grounded action with a concrete object. The energy informs the vibe without being literal
14. **Every prompt must be equally dense** — if one discipline has less visual detail than another, it comes out vague while the detailed ones look sharp. Audit all 8 side by side before generating
15. **Actions need concrete objects** — "directing traffic" is abstract and produces vague results. "Gripping the edge of a massive glowing blue archway door while holding a translucent blue holographic floor plan" gives the model something to render
16. **Discipline negatives should be role-specific** — not generic "no combat" but "what would make THIS character look pathetic?" (e.g., Testing with eyes closed ignoring problems, Database with messy chaos)
17. **1MP is sufficient for iteration** — saves VRAM and time vs 1.5MP, resolution difference is minor at preview stage
18. **Batch generate with a for loop** — `for i in 00 01 02 03 04 05 06 07; do cargo run ... -- 03 $i; done` generates all 8 in ~2 minutes
