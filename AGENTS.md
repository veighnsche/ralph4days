# Repository Guidelines

## Hard-Fail Policy (Mandatory)
- This repository is fail-fast by default. Any violated invariant, invalid assumption, or untrusted state must fail loudly and immediately.
- Do not add defensive branches, silent default values, or fallback behavior for internal defects.
- Bugs must surface as explicit failures (error return, panic in impossible states, rejected request, or equivalent hard stop), then be fixed.
- It is an error to hide issues behind compatibility wrappers, "graceful" no-op paths, or broad catch-all recovery.
- Only allow defensive handling when the edge is truly external and unreliable (network, file system, user input) and still emit observable failure signals.
- The acceptance bar for internal code is: correctness first, then resilience.

## Project Structure & Module Organization
This repository is a Tauri desktop app with a React frontend and Rust backend workspace.

- `src/`: React 19 + TypeScript UI (pages, hooks, components, terminal client).
- `src-tauri/`: Tauri entrypoint and command/terminal backends.
- `crates/`: shared Rust crates (`sqlite-db`, `prompt-builder`, `ralph-errors`, `ralph-rag`, `ralph-external`, etc.).
- `fixtures/`: read-only sample projects; disposable mock data is copied outside the repo (default: `/tmp/ralph4days-mock`).
- `.docs/`: canonical location for implementation notes and design records.
  New docs must use the next chronological numeric prefix in the filename (for example `068_SOME_TITLE.md`).

## Build, Test, and Development Commands
Use `just` as the primary task runner (`just --list`).

- `just dev`: run full Tauri app in development mode.
- `just dev-frontend`: run Vite frontend only.
- `just refresh-tauri-fixtures-mock`: prepare fresh mocks for GUI.
- `just test`: run Rust + frontend unit tests.
- `just lint` / `just fmt-check`: lint and formatting checks.
- `just check-all`: lint + formatting gate.
- `just build`: production desktop build.
- `just reset-mock` then `just dev-mock 03`: test against fixture-backed mock project.

## Coding Style & Naming Conventions
- TypeScript/TSX formatting and linting are enforced by Biome + oxlint (`biome.json`).
- Formatting defaults: 2 spaces, single quotes in TS, max line width 120, LF endings.
- Rust uses `cargo fmt` and clippy (workspace lints in `Cargo.toml`), with `unsafe_code = deny`.
- Keep naming consistent with existing patterns: React components in PascalCase, hooks as `useX`, tests as `*.test.ts(x)`; Rust tests commonly in `tests/*_test.rs`.
- Fact-check baseline (as of 2026-02-13): React manual memoization APIs (`useMemo`, `useCallback`, `React.memo`) are not deprecated in React docs; React Compiler reduces the need for manual memoization.
- Repository policy: treat manual memoization as deprecated for this codebase. Do not add new `useMemo`, `useCallback`, or `React.memo` usage unless explicitly requested by the user.
- Process rule: do not re-run this memoization deprecation fact-check during normal work; only re-check if the user explicitly asks to re-verify.
- Style must not be used to paper over correctness bugs; code quality checks are for enforcing explicit intent, not adding defensive complexity.

## State & Compatibility Policy
- Keep shared UI state centralized with a single source of truth.
- Do not introduce fallback state initialization or compatibility branching that masks missing/invalid state.
- Avoid compatibility layers for low-value local data unless there is clear product impact.
- Any compatibility path must have documented value, owner, and removal criteria.
- Zustand state updates must no-op on same-value writes (no redundant transitions).
- Any store changed in a PR must include/maintain tests that assert no-op writes produce zero transitions.
- For shared state and reducers, remove branches that silently ignore impossible transitions.

## Agent Failure Mode
- Common failure mode: under uncertainty, agents overproduce defensive fallbacks to avoid visible breakage.
- This masks contract violations and creates silent-error paths that accumulate over time.
- Default standard: fail loud on broken invariants and boundary contract mismatches.
- Do not use defensive handling to preserve operation when contracts are broken. Replace it with explicit failure and remediation work.
- Allow defensive handling only for genuinely unreliable external edges, with explicit error surfacing and metrics/logging.
- Non-actionable defensive code is a defect that must be filed and fixed before shipping.

## Global Constraint Policy (All Domains)
- Treat DRY as a hard invariant: every fact must have exactly one canonical owner.
- Before editing schema/state/contracts, define and preserve ownership boundaries (source-of-truth map).
- Do not duplicate semantically equivalent fields across entities/tables/stores unless explicit snapshot or override behavior is requested.
- If duplication is required, document the canonical owner and synchronization/override rules in the same change.
- When requirements conflict, prioritize invariants over convenience and ask for clarification rather than introducing parallel ownership.
- Under uncertainty, default to stricter normalization and explicit ownership, not defensive duplication.
- If ownership is unclear or violated, fail the change and request clarification instead of adding synchronization shims.

## Testing Guidelines
- Frontend unit tests: Vitest (`bun test:run` or `just test-frontend`).
- Rust tests: `cargo test --manifest-path src-tauri/Cargo.toml` (or `just test-rust`).
- No fixed coverage threshold is currently enforced; all changed behavior should include or update tests.
- Frontend runtime-debug rule: for debugging terminal/front-end regressions, use deterministic component and workspace-level tests plus direct app/runtime assertions through the Tauri process. Never use browser-only UI automation as the primary signal.

## Auto-Generated Files Policy
- Do not manually edit auto-generated files (for example `bun.lock`, lockfiles from package managers, compiled outputs, and build artifacts).
- Update auto-generated files only through the owning tool or package manager command that produces them, and only when that update is explicitly requested and directly scoped to that change.
- Treat generated-file diffs as high-cost noise during debugging: if a task does not require regeneration, avoid touching generated files entirely and avoid spending token/tooling cycles on them.
- If an auto-generated file changes unexpectedly, fix the underlying workflow or dependency contract first; do not workaround by manual edits.

## Environment Notes
- This system is Wayland-only for GUI runs.
- Do not suggest or depend on X11/XWayland fallbacks (`GDK_BACKEND=x11`, etc.); X11 is not available here.
- `fixtures/04-desktop-dev/.undetect-ralph/db/ralph.db` is a known persistent diff artifact and can be ignored in change lists unless explicitly asked to handle it.
- If a known artifact or drift is encountered during validation, report it directly rather than adding guardrails to avoid detection.

## Commit & Pull Request Guidelines
- Follow Conventional Commit-style prefixes seen in history: `feat(...)`, `fix(...)`, `refactor(...)`, `chore(...)`, `docs(...)`.
- Keep commits scoped and functional; avoid mixing unrelated frontend/backend changes when possible.
- Before opening a PR, run `just check-all` and `just test`.

- Strict harness policy:
  - Do not introduce browser-automation tooling or harnesses for production validation.
  - Validate frontend/runtime behavior through native Tauri process assertions and unit/integration tests, never browser-side e2e automation.
- PRs should include: clear summary, impacted areas, linked issue/task, and screenshots or recordings for UI changes.
- PRs should include a short "Failure posture" note describing any assumptions and why no new silent fallbacks were introduced.

## Communication Gap Note (UI Placement)
- Known user gap: translating spatial UI intent into unambiguous implementation constraints can be inconsistent under frustration.
- Agent requirement: when UI placement wording is directional (`under`, `inside`, `outside`, `between`) and there is any ambiguity, convert the request into a strict ordered structure before coding.
- Use this format in replies before implementation:
  - exact render order list,
  - explicit inside/outside container rules,
  - explicit "must not" rules,
  - short acceptance checklist.
- Treat the user's ASCII layout as authoritative when provided.
