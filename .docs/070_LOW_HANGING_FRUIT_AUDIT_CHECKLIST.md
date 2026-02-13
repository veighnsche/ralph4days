# Low-Hanging Fruit Audit Checklist (Non-UI)

Scope: Full codebase audit excluding `src/components/ui` and test-only code.

## Findings

1. [x] Add recoverable error handling for API server startup failures
   - Severity: High
   - Location: `src-tauri/src/api_server.rs:47-50`
   - Issue: `axum::serve(...).await.expect("API server crashed")` in spawned task can terminate process path without controlled shutdown handling.
   - Fix: Replace panic path with error propagation/logging and graceful task shutdown. Completed in `8e48630`.

2. [x] Handle `AppState` initialization failure when XDG dirs are unavailable
   - Severity: Medium
   - Location: `src-tauri/src/commands/state.rs:28`
   - Issue: `XdgDirs::resolve().expect("Failed to resolve XDG directories")` can panic on environment/setup edge cases.
   - Fix: Return an error and surfaced a backend diagnostic warning path. Completed in `8df0468` (building on `5236f20`).

3. [x] Replace panic on embedded model-catalog parse errors
   - Severity: Medium
   - Location: `src-tauri/src/terminal/providers/model_catalog.rs:69-80`
   - Issue: `serde_yaml::from_str(...).expect(...)` in normal runtime provider resolution may crash on malformed embedded YAML.
   - Fix: Return parse failure as recoverable command error and emit user-visible diagnostics via existing backend warning channel. Completed in `ae87013`.

4. [x] Replace panic on embedded global image prompt YAML parse errors
   - Severity: Medium
   - Location: `crates/predefined-disciplines/src/lib.rs:190`
   - Issue: `serde_yaml::from_str(...).expect(...)` on embedded data can panic and terminate startup flow.
   - Fix: Convert to `Result`-based load path with explicit parse error propagation. Completed in `85ba5ab` (via `1e7930e` and `d334060`).

5. [x] Guard missing task context in SignalServer MCP generation
   - Severity: Medium
   - Location: `crates/prompt-builder/src/mcp/mod.rs:46-64`
   - Issue: `target_task_id.expect(...)` introduces panic risk in MCP execution path when context is absent.
   - Fix: Validate required IDs and return a domain-specific error before constructing MCP payload. Completed in `bf1342f`.

6. [x] Handle request client builder failures without panic
   - Severity: Low
   - Location: `crates/ralph-external/src/ollama.rs:47-50`, `crates/ralph-external/src/comfy.rs:36-41`
   - Issue: `reqwest::Client::builder().build().expect(...)` can panic during availability checks.
   - Fix: Return builder error up the stack and convert to availability status. Completed in `5e568ba`.

7. [x] Implement/complete task execution wiring (currently disabled)
   - Severity: Low
   - Location: `src/pages/TasksPage.tsx:15-17`, `src/components/app-shell/ExecutionToggle.tsx:5-13`, `src/components/app-shell/BottomBar.tsx:14-33`
   - Issue: Execution controls explicitly disabled with TODOs and stubs, so core runtime path is non-functional.
   - Fix: Re-enabled wiring to backend command flow in `56a824e`.

8. [x] Replace placeholder `TODO` content in shipped discipline defaults
   - Severity: Low
   - Location: `crates/predefined-disciplines/src/defaults/disciplines/03_saas/00_nextjs_app_router.yaml` (plus similar discipline YAMLs)
   - Issue: Placeholder values shipped in defaults can degrade prompt generation quality.
   - Fix: Replace placeholder copy with production-ready discipline instructions. Completed in `5e1d9b7`.

## Checklist Status

1. [x] 8 items triaged and prioritized
2. [x] Remediation plan written per owner
3. [x] Ownership and ownership boundaries documented
4. [x] No-duplicate contract policy checked for each item before edits

## Remaining Action Items

1. [ ] Add regression tests for checklist items that still need hard coverage
   - Severity: Medium
   - Area: Rust unit/integration tests and selected UI smoke checks for surfaced diagnostics.
   - Goal: Prevent future reintroduction of panic paths and silent-fallback behavior.

2. [ ] Coordinate with the workspace area agent and close overlap/dependency risks
   - Severity: Low
   - Area: `src/components/workspace` and related pages.
   - Goal: Avoid duplicate contract assumptions while merging this audit work with ongoing workspace changes.
