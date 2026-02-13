# Desktop Templates

**Purpose**: Desktop stack project with routine task templates (no hook system required yet).

This fixture seeds `task_details` + `task_templates` for multiple disciplines, then creates
a few runtime tasks instantiated from those templates (pull model).

## Usage

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test generate_fixtures generate_fixture_05_desktop_templates -- --nocapture
just reset-mock
just dev-mock 05-desktop-templates
```

## Contents

- `.undetect-ralph/db/ralph.db`
  - 3 subsystems
  - 6 active task templates bound to disciplines
  - 3 runtime tasks instantiated from templates

## Intent

- Demonstrate reusable routine templates before hooks exist
- Keep templates as persistent pending definitions
- Show runtime tasks pulling from templates via `template_id`
