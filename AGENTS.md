# Repository Guidelines

## Project Structure & Module Organization
This repository is a Tauri desktop app with a React frontend and Rust backend workspace.

- `src/`: React 19 + TypeScript UI (pages, hooks, components, terminal client).
- `src-tauri/`: Tauri entrypoint and command/terminal backends.
- `crates/`: shared Rust crates (`sqlite-db`, `prompt-builder`, `ralph-errors`, `ralph-rag`, `ralph-external`, etc.).
- `e2e/`: Playwright end-to-end, visual, and monkey tests.
- `fixtures/`: read-only sample projects; `mock/` is disposable test data copied from fixtures.
- `docs/` and `.docs/`: implementation notes and design records.

## Build, Test, and Development Commands
Use `just` as the primary task runner (`just --list`).

- `just dev`: run full Tauri app in development mode.
- `just dev-frontend`: run Vite frontend only.
- `just refresh-tauri-fixtures-mock`: prepare fresh mocks for GUI.
- `just test`: run Rust + frontend unit tests.
- `just test-e2e` / `just test-visual`: run Playwright suites.
- `just lint` / `just fmt-check`: lint and formatting checks.
- `just check-all`: lint + formatting gate.
- `just build`: production desktop build.
- `just reset-mock` then `just dev-mock 03`: test against fixture-backed mock project.

## Coding Style & Naming Conventions
- TypeScript/TSX formatting and linting are enforced by Biome + oxlint (`biome.json`).
- Formatting defaults: 2 spaces, single quotes in TS, max line width 120, LF endings.
- Rust uses `cargo fmt` and clippy (workspace lints in `Cargo.toml`), with `unsafe_code = deny`.
- Keep naming consistent with existing patterns: React components in PascalCase, hooks as `useX`, tests as `*.test.ts(x)`; Rust tests commonly in `tests/*_test.rs`.

## Testing Guidelines
- Frontend unit tests: Vitest (`bun test:run` or `just test-frontend`).
- Rust tests: `cargo test --manifest-path src-tauri/Cargo.toml` (or `just test-rust`).
- E2E/visual: Playwright in `e2e/`.
- No fixed coverage threshold is currently enforced; all changed behavior should include or update tests.

## Commit & Pull Request Guidelines
- Follow Conventional Commit-style prefixes seen in history: `feat(...)`, `fix(...)`, `refactor(...)`, `chore(...)`, `docs(...)`.
- Keep commits scoped and functional; avoid mixing unrelated frontend/backend changes when possible.
- Before opening a PR, run `just check-all` and `just test` (plus `just test-e2e` for UI-impacting changes).
- PRs should include: clear summary, impacted areas, linked issue/task, and screenshots or recordings for UI changes.
