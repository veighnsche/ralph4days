# Stack Character Portrait Prompts

Z-Image **Base** prompts for discipline character portraits. Generated at **9:16 vertical** (e.g., 1080×1920) for use as character cards in the team roster UI. Each stack has ONE prompt pair that captures its visual identity — all 8 disciplines within that stack share the same aesthetic universe.

> **Tech:** Use **Z-Image Base**, not Turbo. Turbo ignores negative prompts entirely (no CFG). Base supports `negative_prompt` field for consistency.

## Z-Image Formula (from previous research)

**6-part structure:** Subject + Scene + Composition + Lighting + Style + Constraints

**Key principles:**
- Tilt-shift lens for miniature-world softness at edges
- Shallow depth of field, single plane of sharp focus
- Muted/desaturated palette with one dominant accent color
- Consistent grain (medium-format film stock)
- Wes Anderson × product catalog aesthetic: precise, intentional, tactile
- Color dominance is critical — the accent color must saturate reflections, spills, ambient light

## Global Negative Prompt (use for ALL stacks)

```
blurry, out of focus, motion blur, noise, grain clumps, banding, jpeg artifacts, compression, pixelated, low resolution, upscaled, oversaturated, neon, garish, HDR tonemapping, lens flare, chromatic aberration, vignette, fisheye distortion, split image, collage, grid, multiple panels, border, frame, text, words, letters, numbers, typography, caption, subtitle, watermark, signature, logo, brand name, icon overlay, UI elements, buttons, menus, screenshot, browser chrome
```

---

## Stack 1: Generic (Industrial Mechanical)

**Visual Identity:** Industrial mechanical - faceless representations. Tangible objects: servers, circuit boards, terminals, cables, screens. Universal, functional, impersonal tone. Technical diagrams, isometric hardware illustrations, blueprint aesthetics.

### Positive Prompt

```
Vertical 9:16 portrait photograph. A single technical object centered in frame against a deep near-black background (#09090b) — choose from: server blade with glowing status LEDs, exposed circuit board with visible traces and components, vintage terminal with CRT screen, coiled ethernet cables in cable management, rack-mounted equipment front panel. Shot through subtle tilt-shift lens giving miniature-world softness to edges. The object is lit by a single cool-toned industrial light source (blue-white LED or fluorescent spill) creating sharp highlights on metal surfaces and deep shadows that fall into pure black. No human presence — just the object itself, intimate and technical like a product catalog shot. The background occupies 40% of frame (top and bottom), keeping the object centered and isolated. Shallow depth of field makes the background elements dissolve into soft bokeh. Color palette is desaturated grays, gunmetal blacks, with one accent color from status LEDs or indicator lights (blue, amber, green, or red depending on the object). Consistent grain structure resembling medium-format film. The mood is quiet, precise, industrial — a technical diagram brought to life as a tactile photograph. Wes Anderson crossed with an engineering manual.
```

### Negative Prompt

```
people, faces, hands, human figures, cartoon, anime, illustration, 3D render, CGI, plastic toy, glossy, colorful, rainbow, RGB lighting, gaming aesthetic, cluttered, messy, multiple objects, busy composition, bright room, fully lit scene, white background, light background, warm lighting, orange tones, pink tones, purple tones, outdoor scene, nature, plants, wood grain, fabric, soft materials
```

---

## Stack 2: Desktop (1990s Action Cartoons)

**Visual Identity:** 1990s action cartoons - bold character designs. Hannah-Barbera meets action anime, strong silhouettes, dynamic poses. Traditional, powerful, builders and makers tone. 90s Saturday morning cartoons, classic superhero teams.

### Positive Prompt

```
Vertical 9:16 portrait photograph in the style of 1990s action figure photography — dramatic, powerful, larger-than-life. A single stylized figure or character-like object centered in frame against a dark gradient background (deep charcoal fading to black). The subject has bold geometric shapes and strong silhouettes reminiscent of 90s action cartoons: chunky proportions, exaggerated features, clean angular forms like action figures or maquettes. Shot through subtle tilt-shift lens. Lit with theatrical three-point lighting creating deep shadows and bright rim lights that emphasize the character's edges and create a hero pose feeling. Color palette is bold and saturated with one dominant accent color (blue, purple, red, or orange) that defines the character's theme — this color appears in highlights, costume details, and background gradient. The background is simple and dramatic, fading from a colored wash at the top to pure black at the bottom. Composition places the character in a dynamic stance or power pose, silhouette reads clearly even in shadow. Film grain texture. The mood is heroic, energetic, nostalgic — like promotional photos of 90s action figures or Saturday morning cartoon key art. Wes Anderson precision meets superhero team roster photography.
```

### Negative Prompt

```
realistic human, photorealistic person, actual photograph of human, candid photo, portrait, selfie, cute, kawaii, soft, rounded, feminine, pastel colors, muted tones, desaturated, corporate, business suit, office, professional, minimalist, clean modern design, iPhone aesthetic, flat design, vector art, logo, icon, sketch, line art, watercolor, painterly, impressionist, abstract, bokeh background with lights, lens flare, dutch angle, wide angle distortion
```

---

## Stack 3: SaaS (Corporate Sleek)

**Visual Identity:** Corporate sleek - professional office characters. Modern business attire, startup energy, high-stakes environment. Fast, serious, move fast and break things tone. Silicon Valley office culture, tech startup aesthetics.

### Positive Prompt

```
Vertical 9:16 portrait photograph with sleek corporate editorial style. A single professional object or figure-representation centered in frame against a gradient background (deep charcoal blue-gray fading to black). The subject embodies startup culture and corporate professionalism: clean geometric forms, modern materials (brushed aluminum, matte black plastic, glass, carbon fiber), minimal ornamentation, sharp edges and precise angles. Think high-end tech product photography meets professional headshot lighting. Shot through subtle tilt-shift lens. Lit with soft beauty lighting — no harsh shadows, gentle wraparound light that creates subtle dimensionality without drama. One accent color dominates (typically blue #3b82f6 or purple #8b5cf6) appearing in LED indicator lights, subtle screen glows, or material accents. Color palette is cool, desaturated, professional — grays, blacks, navy, with that one vivid accent breaking through. Background is smooth gradient, no texture or detail, creating a studio environment feel. Shallow depth of field. Film grain minimal but present. The mood is polished, confident, fast-moving but controlled — like Bloomberg terminal aesthetic crossed with startup pitch deck photography. Precision and urgency. Wes Anderson discipline meets TechCrunch feature photo.
```

### Negative Prompt

```
people, faces, hands, human figures, casual clothing, cartoon, anime, illustration, bold colors, saturated rainbow, action pose, dynamic angle, heroic lighting, dramatic shadows, warm tones, orange, red, yellow, cute, soft, rounded shapes, organic forms, wood, fabric, leather, vintage, retro, nostalgic, cluttered, messy, multiple objects, consumer products, toys, decorative elements, plants, nature, outdoor, bright daylight
```

---

## Stack 4: Mobile (Anime-Inspired)

**Visual Identity:** Anime-inspired - soft and approachable characters. Cute, feminine, rounded shapes, friendly aesthetic. Accessible, user-friendly, delightful tone. Kawaii culture, modern anime character design, mobile game aesthetics.

### Positive Prompt

```
Vertical 9:16 portrait photograph in kawaii-adjacent product photography style. A single adorable object or figure-representation centered in frame against a soft gradient background (pale cream or blush pink fading to white at top, deeper pastel tone at bottom). The subject has soft rounded forms, no sharp edges — think of bubble-like shapes, pill forms, smooth curves, and friendly proportions reminiscent of kawaii character design or modern anime mascots. Shot through subtle tilt-shift lens creating extra dreamlike softness. Lit with gentle diffused lighting, no harsh shadows, everything is soft and approachable. One pastel accent color dominates (pink #ec4899, mint green, lavender, peach, or sky blue) appearing throughout the object's details and reflected in the background gradient. Color palette is light, airy, feminine — pastels, soft whites, gentle color washes. The object might have cute minimal features (simple eyes, gentle smile, friendly silhouette) but remains a tangible photographed object, not a drawing. Background is clean gradient with possible bokeh of soft pastel lights. Film grain subtle and fine. The mood is delightful, accessible, friendly, gentle — like premium mobile game character renders or Japanese character goods photography. Approachable luxury. Wes Anderson softness meets Sanrio product catalog.
```

### Negative Prompt

```
people, realistic human, photorealistic person, dark, moody, dramatic lighting, harsh shadows, bold contrast, saturated neon, industrial, mechanical, corporate, business, professional, minimalist clean lines, sharp angles, geometric hard edges, masculine, powerful, heroic pose, action stance, retro, vintage, 90s aesthetic, gritty, textured, metal, chrome, glass, carbon fiber, tech product, serious tone, mature, edgy, cool color palette, navy, charcoal, gunmetal, bright white background
```

---

## Usage Notes

1. **Choose your stack** based on the project's discipline set
2. **Copy the positive prompt** for that stack
3. **Concatenate negative prompts:** `[Stack-specific negative], [Global negative]`
4. **Generate at 1080×1920** (9:16) or any 9:16 resolution
5. **Iterate on subject:** Each of the 8 disciplines in a stack can use the same prompt with minor subject variations
6. **Color consistency:** Keep the accent color consistent across all disciplines in a stack for visual cohesion

## Example Subject Variations

For **Stack 2 (Desktop)** - 8 disciplines might be:
- Frontend: Stylized monitor figure with blue accent, power pose
- Backend: Server rack character with purple accent, strong stance
- Data: Database cylinder with green accent, stable foundation pose
- Platform: Cloud-form character with indigo accent, floating stance
- Quality: Testing apparatus with amber accent, measuring pose
- Security: Vault-door character with red accent, guardian stance
- Integration: Cable connector character with cyan accent, linking pose
- Documentation: Blueprint scroll character with teal accent, teaching pose

All use the same Stack 2 prompt — just vary the subject description and accent color.
