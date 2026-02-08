# 040 - Discipline Portrait Prompt Methodology

## Summary

Complete methodology for generating consistent, high-quality discipline character portraits using ComfyUI with Stable Diffusion. Developed iteratively across ~20 generation cycles on Stack 02 (Desktop). The cyberpunk aesthetic was built for Desktop but should actually be applied to **Stack 03 (SaaS)** — Desktop should get a medieval theme instead. This document captures everything learned so it can be applied to any stack.

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
- Art style (illustration, vector, painting, etc.)
- Camera framing (three-quarter portrait, waist up, centered, etc.)
- Subject positioning ("subject occupies lower two-thirds")
- Overall aesthetic/mood (cyberpunk, medieval, corporate, etc.)
- Shared clothing style language (immaculate, sleek, etc.)
- Shared negatives for composition (no multiple people, no photo-realism, etc.)
- Posture energy (dominant, peaceful, aggressive — via negatives)

**What does NOT belong here:**
- Any specific discipline color
- Any character-specific details (gender, ethnicity, clothing items)
- Background details specific to one discipline

**Example (Cyberpunk — intended for SaaS):**
```yaml
image_prompt:
  positive: |
    Stylized illustration, three-quarter portrait from waist up, single person centered in frame,
    subject occupies lower two-thirds, dark sleek corporate cyberpunk aesthetic, polished obsidian
    and glass surfaces, cold ambient lighting, immaculate high-tech executive style, neon accent
    lighting, volumetric fog, dramatic cinematic composition, absurdly luxurious sci-fi fashion,
    clean vector art style with smooth gradients.
  negative: |
    multiple characters, multiple people, crowd, group, two people, photograph, photorealistic,
    realistic skin texture, superhero, weapon, text, words, letters, typography, action figure,
    3D render, plastic, military, writing, caption, title, worn clothing, patches, duct tape,
    dirty, grungy, street punk, smiling, friendly, warm, gentle, soft, relaxed, casual, peaceful,
    kind eyes, open smile, welcoming, empty background, solid color background, plain backdrop,
    photo studio
```

### Layer 3: Discipline (per-discipline YAML → `image_prompt`)
**Scope:** What makes THIS character unique.
**Contains:** Character description, discipline color, clothing, pose, cyberpunk feature, physical environment.

**What belongs here:**
- Character demographics (gender, ethnicity, age, hair, build, facial hair)
- Discipline accent color (mentioned 6-8 times across clothing, lighting, environment, effects)
- Specific clothing items with color
- Specific pose/body language
- Discipline-specific cyberpunk feature (holographic wireframes, circuit tattoos, etc.)
- Specific physical environment (server vault, design studio, library, etc.)
- Discipline-specific negatives

## Color Dominance Rule

Each discipline has an accent color. That color MUST appear multiple times throughout the prompt in different forms:

1. **Clothing** — the main garment is the color
2. **Clothing accent** — secondary detail (piping, trim, accessories)
3. **Accessory** — glasses, earrings, belt buckle in tinted color
4. **Cyberpunk feature** — the hologram/projection/tattoo glows in the color
5. **Rim lighting** — dramatic [color] rim lighting
6. **Ambient glow** — [color] ambient glow
7. **Environment** — "bathed in [color] light"
8. **Environment details** — specific background elements glow in color variants

**Example (Frontend — teal/cyan):**
> teal jacket, cyan LED piping, teal-tinted glasses, teal holographic wireframes, cyan rim lighting, teal ambient glow, bathed in teal light, cyan holographic mockups, teal-lit monitors

This ensures the discipline's color DOMINATES the image regardless of what the model decides to do with the dark background.

## Team Diversity Grid

Each stack has 8 disciplines = 8 unique characters. Design them as a cohesive team with maximum diversity across these axes:

| Variable | Purpose | Examples |
|----------|---------|---------|
| Gender | 50/50 or close | Male, Female, Non-binary |
| Ethnicity | Max variety | East Asian, Black, South Asian, Latina, White, Middle Eastern, Mixed-race |
| Age | Range of decades | Early 20s through late 40s |
| Hair | Distinct silhouettes | Short styled, natural curls, braids, buzz cut, undercut, gray-streaked |
| Build | Variety | Slim, athletic, stocky, broad, tall |
| Facial hair | Where applicable | Clean-shaven, trimmed beard, stubble, full beard |
| Accent color | Discipline identity | Teal, orange, violet, emerald, steel-blue, gold, crimson, cream |
| Clothing | Same style language, different items | Jacket, bomber, turtleneck, technical coat, leather jacket, overcoat |
| Pose | Distinct per person | Chin raised, arms crossed, squared shoulders, leaning forward, arms folded, feet planted, imperious |
| Cyberpunk feature | Discipline-specific | Holographic wireframes, circuit tattoos, data-viz holograms, fiber optic braids, mechanical braces, augmented monocle, shield projections, glowing stylus |

**Rule:** No two characters should share more than 1-2 variables. The team should look like 8 completely different people who happen to share the same aesthetic.

## Pose & Energy Design

### Dominant, Not Aggressive
Characters should project dominance and authority without explicit aggression. Achieved through:

**In positive prompts:**
- "chin raised and piercing gaze"
- "staring down the viewer"
- "squared shoulders and commanding gaze"
- "sharp deliberate gaze"
- "intense narrowed eyes"
- "cold scrutinizing stare"
- "sharp unwavering glare"
- "standing imperiously"

**In stack-level negatives:**
- smiling, friendly, warm, gentle, soft, relaxed, casual, peaceful, kind eyes, open smile, welcoming

This creates characters that look like they OWN their domain — corporate power fantasy, not warmth.

### Preventing Multiple People
Stack-level negatives MUST include: "multiple characters, multiple people, crowd, group, two people" — without this, backgrounds often generate bystanders.

## Physical Environment Design

Every discipline needs a SPECIFIC physical place — not a solid color backdrop or generic photo studio.

**Pattern:** "inside a dark [adjective] [place] bathed in [color] light with [large-scale environment details] behind [him/her/them]"

| Discipline | Environment |
|-----------|-------------|
| Frontend | Dark towering glass-walled design atrium |
| Backend | Massive dark underground server vault |
| Data | Cavernous dark circular data operations center |
| Integration | Vast dark network operations bridge |
| Platform | Dark cavernous industrial control room |
| Quality | Dark sprawling quality assurance lab |
| Security | Dark fortified security operations center |
| Documentation | Dark impossibly vast digital library cathedral |

**Rule:** Environments should be LARGE and IMPOSING — "towering", "massive", "cavernous", "vast", "impossibly vast". The character dominates the foreground; the environment shows the scale of their domain.

## UNSOLVED: Physical Mirror Metaphor

**Problem:** All software disciplines are fundamentally "person typing on keyboard." We need a physical-world metaphor for each discipline that translates their digital work into something tangible and visually distinct.

**The question:** If each discipline ran a physical operation instead of writing code, what would they be doing?

**NOT a store, NOT an office** — something more evocative. Think: what physical action or craft mirrors what this discipline actually does?

| Discipline | Digital Reality | Physical Mirror? |
|-----------|----------------|-----------------|
| Frontend | Building UI components, layouts, interactions | ??? (Architect? Interior designer? Glassblower?) |
| Backend | Processing requests, data pipelines, business logic | ??? (Engineer? Factory floor? Forge?) |
| Data | Schema design, queries, migrations, storage | ??? (Librarian? Vault keeper? Cartographer?) |
| Integration | Connecting systems, API bridges, protocol translation | ??? (Diplomat? Bridge builder? Switchboard operator?) |
| Platform | OS-level, builds, CI/CD, infrastructure | ??? (Construction foreman? Power plant operator?) |
| Quality | Testing, validation, catching bugs | ??? (Inspector? Jeweler with loupe? Quality control?) |
| Security | Threat detection, encryption, access control | ??? (Guard? Locksmith? Sentinel?) |
| Documentation | Knowledge capture, guides, references | ??? (Scribe? Librarian? Chronicler?) |

**This needs to be solved per-stack** because the metaphor should match the stack's aesthetic:
- **Medieval Desktop:** Could lean into medieval crafts (blacksmith, cartographer, sentinel, scribe)
- **Cyberpunk SaaS:** Could lean into corporate sci-fi roles (architect, engineer, oracle, diplomat)

**Why it matters:** Right now the characters look cool but generic — they're just standing there in a room. The physical mirror would give them something to DO that visually communicates their discipline at a glance, even to someone who doesn't read the label.

## Stack Aesthetic Assignments (CORRECTED)

| Stack | Aesthetic | Status |
|-------|----------|--------|
| 01_generic | TBD | Not started |
| 02_desktop | **Medieval** | Prompts need rewrite — currently has cyberpunk (mistake) |
| 03_saas | **Dark corporate cyberpunk (Arasaka Corp)** | Move current cyberpunk prompts here, adapt to SaaS disciplines |
| 04_mobile | TBD | Not started |

## SaaS Stack Disciplines (different from Desktop!)

| # | Name | Focus |
|---|------|-------|
| 00 | Next.js App Router | File-based routing, RSC, server actions |
| 01 | React Components | Client components, interactivity, shadcn/ui |
| 02 | API Routes | Route handlers, server actions, API design |
| 03 | Database (Postgres/Prisma) | Prisma ORM, migrations, type-safe queries |
| 04 | Authentication | NextAuth.js, Clerk, protected routes, RBAC |
| 05 | Monorepo & Deployment | Turborepo, Vercel platform, CI/CD |
| 06 | Testing | Vitest, Playwright, MSW, component/E2E tests |
| 07 | Documentation | API docs, setup guides, Storybook |

These need their own team grid, colors, characters, environments, and physical mirrors — all within the Arasaka Corp cyberpunk aesthetic.

## Technical Setup

- **Workflow:** 2-CLIP node workflow (`generate_discipline.json`) — one `__POSITIVE__` and one `__NEGATIVE__` CLIPTextEncode node
- **Default ratio:** 9:16 (portrait)
- **Default megapixels:** 1.5MP
- **Half steps:** 14 (for iteration), full steps: 28 (for final)
- **Prompt concatenation:** `"{global} {stack} {discipline}"` for both positive and negative
- **Filename format:** `{NN}_{name}_{steps}_{WxH}_{base36_timestamp}.png`
- **Companion file:** `.txt` with full prompts saved alongside each `.png`
- **Command:** `just gen-image <stack> <discipline> [--half] [--test] [--ratio W H] [--mp N]`

## Next Steps

1. Solve the physical mirror metaphor for SaaS disciplines (cyberpunk)
2. Solve the physical mirror metaphor for Desktop disciplines (medieval)
3. Build SaaS team grid (8 new characters for 8 different disciplines)
4. Build Desktop team grid (8 characters with medieval aesthetic)
5. Add ControlNet to workflow for consistent character positioning
6. Generate final portraits with ControlNet pose normalization

## Iteration Lessons Learned

1. **Start with 1 sentence per prompt section** — long prompts produce abstract mush
2. **Lock art style at stack level** — "Stylized illustration, clean vector art style" prevents random style drift
3. **Negative prompts are as important as positive** — peaceful/friendly in negatives creates dominance without aggression
4. **Color must be hammered** — mentioning a color once means the model might ignore it. 6-8 mentions across different elements locks it in.
5. **"magazine cover" triggers text** — avoid publication-related terms
6. **"blurred X behind them" creates empty space** — use "inside a [place] with [things]" instead for rich environments
7. **Always test at half steps first** — 14 steps is enough to evaluate composition and color
8. **Each image costs ~15 seconds at half steps** — fast enough for rapid iteration
9. **Stack negatives for multiple people are essential** — without them, backgrounds spawn random figures
10. **"absurdly" is a great prompt modifier** — pushes details toward over-the-top fashion editorial without breaking the image
