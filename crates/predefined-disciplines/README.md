# predefined-disciplines

Static content library for Ralph discipline definitions. Ships predefined stacks, disciplines, and image generation prompts as compile-time embedded YAML.

## Structure

```
src/
├── lib.rs                          # Public API
├── image_prompts.yaml              # Global image generation prompts
├── bin/
│   └── generate_discipline_image.rs
├── comfyui_workflows/
│   ├── generate_discipline_main.json       # Production (28 steps)
│   └── generate_discipline_main_test.json  # Fast validation (1 step)
└── defaults/disciplines/
    ├── 01_generic/          # Stack 1: Language-agnostic
    │   ├── ABOUT.yaml
    │   ├── 00_implementation.yaml
    │   ├── ...
    │   └── images/
    ├── 02_desktop/          # Stack 2: Tauri + React
    │   ├── ABOUT.yaml
    │   ├── 00_frontend.yaml
    │   ├── ...
    │   └── images/
    ├── 03_saas/             # Stack 3: Next.js + Vercel (ABOUT only)
    └── 04_mobile/           # Stack 4: Flutter + Firebase (ABOUT only)
```

## API

```rust
use predefined_disciplines::*;

// Stack metadata
let stacks = get_all_stack_metadata();
let desktop = get_stack_metadata(2).unwrap();

// Disciplines for a stack
let disciplines = get_disciplines_for_stack(2);

// Image generation prompts
let global = get_global_image_prompts();
```

## Image Generation

Generates discipline character portraits via ComfyUI. Requires ComfyUI running locally (default: `localhost:8188`).

```bash
# Production (28 steps, full quality)
just generate-discipline-image 02 00

# Fast pipeline validation (1 step)
just generate-discipline-image-test 02 00
```

Arguments are zero-padded numbers matching the directory/file prefixes.

### Examples

```bash
# Generate frontend portrait (stack 02, discipline 00)
just generate-discipline-image 02 00

# Test pipeline for generic implementation (stack 01, discipline 00)
just generate-discipline-image-test 01 00
```

Output: `src/defaults/disciplines/{stack}/images/{NN}_{name}.png` (or `{NN}_{name}_test.png` with `--test`).

### Index

| Stack | Disciplines |
|-------|-------------|
| 01_generic | 00 implementation, 01 refactoring, 02 investigation, 03 testing, 04 architecture, 05 devops, 06 security, 07 documentation |
| 02_desktop | 00 frontend, 01 backend, 02 data, 03 integration, 04 platform, 05 quality, 06 security, 07 documentation |

### Prompt layering

Image prompts are assembled from three layers, each defined in YAML:

1. **Global** (`image_prompts.yaml`) — base photographic style
2. **Stack** (`{stack}/ABOUT.yaml` → `image_prompt`) — stack visual identity
3. **Discipline** (`{discipline}.yaml` → `image_prompt`) — subject + style keywords

## Boundaries

This crate owns **content and image generation**. It does NOT touch the database.

- `predefined-disciplines` → provides discipline definitions + generates images
- `sqlite-db` → receives `DisciplineInput` structs and writes them (pure CRUD)
- `ralph-external` → ComfyUI protocol (prompt injection, polling, fetching)
