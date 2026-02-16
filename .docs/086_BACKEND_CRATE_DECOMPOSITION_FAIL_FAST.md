# 086 Backend Crate Decomposition + Fail-Fast Hardening (2026-02-16)

Date: 2026-02-16
Author: Codex
Scope: replace the ad-hoc monolithic `crates/ralph-backend` crate with domain-focused crates and preserve explicit failure behavior.

## Why This Change
- `crates/ralph-backend` mixed unrelated concerns (runtime diagnostics, project/session lifecycle, task domain logic, prompt composition, terminal transport, API server).
- The mixed tree made ownership unclear and increased coupling between desktop command adapters and backend internals.
- The split creates explicit boundaries and keeps Tauri/daemon command layers thin adapters over reusable domain crates.

## New Crate Ownership Map
- `crates/ralph-backend-runtime`
  - Diagnostics/event sink registration
  - XDG/runtime filesystem concerns
- `crates/ralph-backend-project`
  - Project path validation/init
  - Project scan and project/session DB helpers
- `crates/ralph-backend-tasks`
  - Task domain services
  - Agent session services
  - Internal API server bootstrap
- `crates/ralph-backend-subsystems`
  - Subsystems and disciplines contracts/services
- `crates/ralph-backend-prompts`
  - MCP config generation
  - Prompt builder preview + prompt config services
- `crates/ralph-backend-terminal`
  - Terminal bridge/session/provider orchestration
  - PTY manager/stream/replay behavior

## Fail-Fast Decisions Applied
- Terminal agent and MCP mode are now typed enums in `ralph-contracts` (`TerminalAgent`, `TerminalMcpMode`) instead of free-form strings.
- Provider/model catalog resolution returns `RalphResult` and hard-fails on invalid YAML or unknown model/provider.
- Task terminal start context now validates DB availability before task MCP config generation to preserve deterministic error ordering.
- Runtime/local command utilities panic loudly on impossible local-state violations (invalid provider input in internal helper paths).

## Integration Changes
- Workspace membership now references the six new backend crates.
- `src-tauri` and `src-daemon` depend on the split crates directly.
- Backend imports were remapped from `ralph_backend::...` to explicit crate paths by domain.
- Mobile dependency gate keeps blocking backend crates via `ralph-backend(-|$)` pattern.

## Verification
- `cargo check --workspace`
- `cargo test -p ralph-backend-terminal`
- `cargo test -p ralph-backend-runtime -p ralph-backend-project -p ralph-backend-tasks -p ralph-backend-subsystems -p ralph-backend-prompts`
- `just types`
- `bun test:run src/lib/terminal/terminalBridgeClient.test.ts src/lib/terminal/session.test.ts`

## Follow-Up Guardrail
- Keep backend logic out of Tauri command modules; command modules should only decode/dispatch and map errors.
- If a new backend concern appears, add it to the existing owner crate or create a new crate with a single explicit domain owner instead of re-introducing a mixed utility crate.
