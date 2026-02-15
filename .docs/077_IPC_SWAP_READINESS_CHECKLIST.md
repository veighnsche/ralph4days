# IPC Swap Readiness Checklist (Make Current IPC “Perfect”)

Date: 2026-02-15

Goal: make the existing frontend-facing IPC contract (Tauri `invoke` + events) stable, typed, drift-proof, and transport-agnostic so it can be re-hosted behind `ralphd` with minimal/no UI changes.

## Checklist Policy (Atomic Items)
- Every checkbox is one standalone task that a small model can complete in one focused change.
- Every checkbox must specify exactly one `Owner:` (no `Owners:`). If it needs more than one file/dir, split it.
- Each checkbox must specify an `Acceptance:`:
  - a deterministic test/command/grep (`cargo test ...` / `bun test ...` / `just ...` / `rg ...`), or
  - an observable runtime behavior with concrete manual steps.
- Split decisions from implementations:
  - Add a "Decision" checkbox when choosing between meaningful alternatives.
  - Add one implementation checkbox per file/module touched.
- DRY is an invariant: do not duplicate canonical facts across docs; reference the canonical owner instead.

## Current Snapshot (As Of 2026-02-15)
1. Backend portability (can we reuse “backend core” in both Tauri and `ralphd` today?): **Partial** (roughly **6/10**).
   1. Major blockers: business logic still lives in `src-tauri/src/commands/*`, and some subsystems still hard-depend on Tauri runtime types (notably `src-tauri/src/api_server.rs`).
   2. Progress: project path validation + project lock/session + tasks + prompt builder preview + prompt builder config + subsystems/disciplines domain logic are now Tauri-free and reusable (`crates/ralph-backend/src/{project.rs,session.rs,tasks.rs,prompt_builder_preview.rs,prompt_builder_configs_service.rs,subsystems_service.rs,disciplines_service.rs}`).
2. IPC contract maturity (typed + stable + drift-tested enough to proxy 1:1): **Partial** (roughly **8/10**).
   1. Strongest area: terminal bridge contract (wire types + drift tests + buffering/replay semantics) plus the remote proxy transport (`RemoteWireFrame` + hard-fail protocol handshake).
   2. Weakest area: drift testing for non-terminal event domains, plus error-shape standardization across all commands.

## 0. Definitions
1. “IPC contract” = command names + request/response JSON shapes + event names + event payload shapes.
2. “Swap-ready” = you can run the same contract over a different transport (HTTP/WS/stdio) without rewriting product logic or UI behavior.

## 1. Contract Freeze + Inventory (Must-Have)

### 1.1 Command List Drift
- [x] Canonical local command list owner is `src-tauri/src/lib.rs` (`tauri::generate_handler![...]`).
  - Owner: `src-tauri/src/lib.rs`.
  - Acceptance: `cargo test --manifest-path src-tauri/Cargo.toml --test invoke_command_list_contract_test`.
- [x] Drift test snapshots the canonical command list (intentional changes require updating the snapshot).
  - Owner: `src-tauri/tests/invoke_command_list_contract_test.rs`.
  - Acceptance: `cargo test --manifest-path src-tauri/Cargo.toml --test invoke_command_list_contract_test`.

### 1.2 Event Name Drift
- [x] Canonical Rust terminal-event constants exist for every terminal event the frontend listens to.
  - Owner: `crates/ralph-contracts/src/terminal.rs`.
  - Acceptance: `cargo test -p ralph-contracts`.
- [x] Canonical Rust non-terminal event constants exist for every non-terminal event the frontend listens to.
  - Owner: `crates/ralph-contracts/src/events.rs`.
  - Acceptance: `cargo test -p ralph-contracts`.
- [x] Canonical Rust `FRONTEND_EVENT_NAMES` includes every frontend-listened event name.
  - Owner: `crates/ralph-contracts/src/frontend.rs`.
  - Acceptance: `cargo test -p ralph-contracts`.
- [x] Frontend drift test: listened terminal event names match canonical Rust constants.
  - Owner: `src/lib/terminal/terminalBridgeContract.test.ts`.
  - Acceptance: `bun test:run src/lib/terminal/terminalBridgeContract.test.ts` (or `just test-frontend`).
- [x] Frontend drift test: listened non-terminal event names match canonical Rust constants.
  - Owner: `src/lib/tauri/eventsContract.test.ts`.
  - Acceptance: `bun test:run src/lib/tauri/eventsContract.test.ts` (or `just test-frontend`).

### 1.3 v1 `ralphd` Parity Surface (Doc Contract)
- [x] Enumerate the v1 remote UI parity subset (RPC + events), plus explicitly list local-only and nice-to-have surfaces.
  - Owner: `.docs/077_IPC_SWAP_READINESS_CHECKLIST.md`.
  - Acceptance: the enumerations below exist and are updated whenever IPC changes.

#### Supported Surface For v1 `ralphd` Parity (Canonical)
- Scope must be explicit: which commands/events are required for “remote UI parity” vs “nice-to-have”.
- Canonical owners:
  - Full local IPC command list: `src-tauri/src/lib.rs` (`tauri::generate_handler![...]`).
  - Frontend-listened event names: `crates/ralph-contracts/src/frontend.rs` (`FRONTEND_EVENT_NAMES`) + `src/lib/tauri/listenedEventsContract.ts`.
  - This section is the canonical owner of the v1 `ralphd` parity *subset*.

##### Remote UI parity (v1 MUST support in `ralphd`) — RPC commands
- `protocol_version_get`
- Project:
  - `project_lock_get`
  - `project_lock_set`
  - `project_recent_list`
  - `project_scan`
  - `project_initialize`
  - `project_info_get`
- Tasks:
  - `tasks_get`
  - `tasks_list_items`
  - `tasks_update`
  - `tasks_set_status`
  - `tasks_signal_add`
  - `tasks_signal_update`
  - `tasks_signal_delete`
  - `tasks_signal_summaries_get`
  - `tasks_comment_reply_add`
- Subsystems:
  - `subsystems_list`
  - `subsystems_comment_add`
  - `subsystems_comment_update`
  - `subsystems_comment_delete`
- Disciplines:
  - `disciplines_list`
  - `disciplines_create`
  - `disciplines_update`
  - `disciplines_delete`
  - `disciplines_cropped_image_get`
- Prompt builder:
  - `prompt_builder_preview`
  - `prompt_builder_config_list`
  - `prompt_builder_config_get`
  - `prompt_builder_config_save`
  - `prompt_builder_config_delete`
- Terminal bridge:
  - `terminal_start_session`
  - `terminal_start_task_session`
  - `terminal_start_human_session`
  - `terminal_list_model_form_tree`
  - `terminal_send_input`
  - `terminal_resize`
  - `terminal_set_stream_mode`
  - `terminal_replay_output`
  - `terminal_terminate`

##### Remote UI parity (v1 MUST support in `ralphd`) — event stream
- `backend-diagnostic`
- `terminal:output`
- `terminal:closed`

##### Local-only commands (not implemented by `ralphd`, even in remote mode)
- `window_splash_close`
- `window_open_new`
- `remote_connect`
- `remote_disconnect`
- `remote_status_get`
- `terminal_emit_system_message` (UI-only terminal UX injection; emitted locally)
- `stacks_metadata_list` (static local data from `predefined-disciplines`)

##### Nice-to-have / out-of-scope for v1 `ralphd` parity
- Execution engine: `execution_start`, `execution_pause`, `execution_resume`, `execution_stop`, `execution_state_get`
- Project/FS helpers: `system_home_dir_get`, `project_validate_path`
- Extra tasks: `tasks_create`, `tasks_delete`, `tasks_list`, `tasks_ask_answer`, `tasks_signal_comment_*`, `tasks_signal_comments_list`
- Subsystem management: `subsystems_create`, `subsystems_update`, `subsystems_delete`
- Disciplines: `disciplines_image_data_get`
- Agent sessions: `agent_sessions_*`

## 2. Protocol Versioning + Handshake (Must-Have)
- [x] Single canonical `PROTOCOL_VERSION` constant.
  - Owner: `crates/ralph-contracts/src/protocol.rs`.
  - Acceptance: `cargo test -p ralph-contracts`.
- [x] Local IPC command exists to retrieve the protocol version (`protocol_version_get`).
  - Owner: `src-tauri/src/commands/protocol.rs`.
  - Acceptance: `cargo test --manifest-path src-tauri/Cargo.toml`.
- [x] Remote connect hard-fails on protocol mismatch (includes local + remote versions).
  - Owner: `src-tauri/src/remote.rs` (`RemoteWireFrameConnection::connect`).
  - Acceptance: `cargo test --manifest-path src-tauri/Cargo.toml remote::tests::connect_hard_fails_on_protocol_mismatch`.

## 3. Wire-Type Canonicalization (Must-Have)

### 3.1 TS Type Export Pipeline
- [x] `#[ipc_type]` macro exports TS bindings via `ts-rs`.
  - Owner: `crates/ralph-macros/src/lib.rs`.
  - Acceptance: `just types` produces `target/ts-bindings/*.ts`.
- [x] Type regeneration is the only way wire types change.
  - Owner: `justfile` recipes `types` + `types-check`.
  - Acceptance: `just types-check` fails if `src/types/generated.ts` is stale.

### 3.2 Single Owner Per Wire Type (No Duplicates)
- [x] Protocol types are owned by `crates/ralph-contracts/src/protocol.rs`.
  - Owner: `crates/ralph-contracts/src/protocol.rs`.
  - Acceptance: `just types-check`.
- [x] Remote wire framing types are owned by `crates/ralph-contracts/src/transport.rs`.
  - Owner: `crates/ralph-contracts/src/transport.rs`.
  - Acceptance: `just types-check`.
- [x] Terminal event payload types are owned by `crates/ralph-contracts/src/terminal.rs`.
  - Owner: `crates/ralph-contracts/src/terminal.rs`.
  - Acceptance: `just types-check`.
- [x] Non-terminal event payload types are owned by `crates/ralph-contracts/src/events.rs`.
  - Owner: `crates/ralph-contracts/src/events.rs`.
  - Acceptance: `just types-check`.
- [x] Project lock/session args are owned by `crates/ralph-backend/src/session.rs`.
  - Owner: `crates/ralph-backend/src/session.rs`.
  - Acceptance: `just types-check`.
- [x] Project validate/initialize args are owned by `crates/ralph-backend/src/project.rs`.
  - Owner: `crates/ralph-backend/src/project.rs`.
  - Acceptance: `just types-check`.
- [x] Task-domain command args are owned by `crates/ralph-backend/src/tasks.rs`.
  - Owner: `crates/ralph-backend/src/tasks.rs`.
  - Acceptance: `just types-check`.
- [x] Task-domain result DTOs (task rows, list items, signals, etc) are owned by `crates/sqlite-db/src/types.rs`.
  - Owner: `crates/sqlite-db/src/types.rs`.
  - Acceptance: `just types-check`.
- [x] Prompt builder config DTOs are owned by `crates/ralph-backend/src/prompt_builder_configs_contract.rs`.
  - Owner: `crates/ralph-backend/src/prompt_builder_configs_contract.rs`.
  - Acceptance: `just types-check`.
- [x] Prompt builder config DB DTOs are owned by `crates/sqlite-db/src/prompt_builder_configs.rs`.
  - Owner: `crates/sqlite-db/src/prompt_builder_configs.rs`.
  - Acceptance: `just types-check`.
- [x] Subsystems args/results are owned by `crates/ralph-backend/src/subsystems_contract.rs`.
  - Owner: `crates/ralph-backend/src/subsystems_contract.rs`.
  - Acceptance: `just types-check`.
- [x] Disciplines args/results are owned by `crates/ralph-backend/src/disciplines_contract.rs`.
  - Owner: `crates/ralph-backend/src/disciplines_contract.rs`.
  - Acceptance: `just types-check`.
- [x] Prompt builder preview args/results are owned by `crates/ralph-backend/src/prompt_builder_preview.rs`.
  - Owner: `crates/ralph-backend/src/prompt_builder_preview.rs`.
  - Acceptance: `just types-check`.
- [x] Create backend-owned DTOs for `project_recent_list`.
  - Owner: (proposed) `crates/ralph-backend/src/project_contract.rs` (new).
  - Acceptance: `just types-check` exports the DTOs exactly once.
- [x] Create backend-owned DTOs for `project_scan`.
  - Owner: (proposed) `crates/ralph-backend/src/project_contract.rs` (new).
  - Acceptance: `just types-check` exports the DTOs exactly once.
- [x] Create backend-owned DTOs for `project_info_get`.
  - Owner: (proposed) `crates/ralph-backend/src/project_contract.rs` (new).
  - Acceptance: `just types-check` exports the DTOs exactly once.
- [x] Create backend-owned DTOs for `agent_sessions_*` commands (even if the commands remain Tauri-only).
  - Owner: (proposed) `crates/ralph-backend/src/agent_sessions_contract.rs` (new).
  - Acceptance: `just types-check` exports the DTOs exactly once.
- [x] Rewire `src-tauri/src/commands/project.rs` to use backend-owned project DTOs (and delete the local DTOs).
  - Owner: `src-tauri/src/commands/project.rs`.
  - Acceptance: `rg \"#\\[ipc_type\\]\" src-tauri/src/commands/project.rs` returns no project DTOs that are supposed to live in backend.
- [x] Rewire `src-tauri/src/commands/agent_sessions.rs` to use backend-owned agent session DTOs (and delete the local DTOs).
  - Owner: `src-tauri/src/commands/agent_sessions.rs`.
  - Acceptance: `rg \"#\\[ipc_type\\]\" src-tauri/src/commands/agent_sessions.rs` returns no DTOs that are supposed to live in backend.

### 3.3 Strict Decode Policy (Remote)
- [x] Decision: strict JSON decode policy for all remote-exposed DTOs.
  - Owner: `.docs/077_IPC_SWAP_READINESS_CHECKLIST.md`.
  - Acceptance: policy is written down, including what must use `deny_unknown_fields`.

#### Strict JSON Decode Policy (Remote-Exposed DTOs)
- Default: every DTO that is deserialized across a transport boundary (remote RPC args, RPC results, remote events) **must** use `#[serde(deny_unknown_fields)]`.
- `#[serde(default)]` is only allowed when the field is truly optional for forward-compat, and its absence is semantically meaningful. It must not be used to mask missing required fields.
- Required collections must serialize as arrays even when empty; do not use `skip_serializing_if = "Vec::is_empty"` for required wire fields.
- Envelopes (`RemoteWireFrame`, `RemoteEventFrame`) must remain strict; unknown tags/fields are protocol errors and must hard-fail.
- [x] Remote transport envelopes decode strictly (unknown fields hard-fail).
  - Owner: `crates/ralph-contracts/src/transport.rs`.
  - Acceptance: `cargo test -p ralph-contracts` (tests cover unknown tags/fields).
- [x] Strict-decode is enabled for protocol DTOs (`deny_unknown_fields` on deserialized structs).
  - Owner: `crates/ralph-contracts/src/protocol.rs`.
  - Acceptance: `rg \"deny_unknown_fields\" crates/ralph-contracts/src/protocol.rs` has hits.
- [x] Strict-decode is enabled for terminal event DTOs.
  - Owner: `crates/ralph-contracts/src/terminal.rs`.
  - Acceptance: `rg \"deny_unknown_fields\" crates/ralph-contracts/src/terminal.rs` has hits.
- [x] Strict-decode is enabled for backend-diagnostic event DTOs.
  - Owner: `crates/ralph-contracts/src/events.rs`.
  - Acceptance: `rg \"deny_unknown_fields\" crates/ralph-contracts/src/events.rs` has hits.
- [x] Strict-decode is enabled for backend subsystems DTOs.
  - Owner: `crates/ralph-backend/src/subsystems_contract.rs`.
  - Acceptance: `rg \"deny_unknown_fields\" crates/ralph-backend/src/subsystems_contract.rs` has hits.
- [x] Strict-decode is enabled for backend disciplines DTOs.
  - Owner: `crates/ralph-backend/src/disciplines_contract.rs`.
  - Acceptance: `rg \"deny_unknown_fields\" crates/ralph-backend/src/disciplines_contract.rs` has hits.
- [x] Strict-decode is enabled for backend prompt preview DTOs.
  - Owner: `crates/ralph-backend/src/prompt_builder_preview.rs`.
  - Acceptance: `rg \"deny_unknown_fields\" crates/ralph-backend/src/prompt_builder_preview.rs` has hits.
- [x] Strict-decode is enabled for backend prompt builder config DTOs.
  - Owner: `crates/ralph-backend/src/prompt_builder_configs_contract.rs`.
  - Acceptance: `rg \"deny_unknown_fields\" crates/ralph-backend/src/prompt_builder_configs_contract.rs` has hits.
- [x] Strict-decode is enabled for Tauri remote connect/status DTOs.
  - Owner: `src-tauri/src/commands/remote.rs`.
  - Acceptance: `rg \"deny_unknown_fields\" src-tauri/src/commands/remote.rs` has hits.
- [x] Strict-decode is enabled for stack metadata DTOs.
  - Owner: `src-tauri/src/commands/subsystems.rs`.
  - Acceptance: `rg \"deny_unknown_fields\" src-tauri/src/commands/subsystems.rs` has hits.
- [x] Add `deny_unknown_fields` to `ProjectValidatePathArgs`.
  - Owner: `crates/ralph-backend/src/project.rs`.
  - Acceptance: `rg \"ProjectValidatePathArgs\" -n crates/ralph-backend/src/project.rs` shows `deny_unknown_fields` on its serde attrs.
- [x] Add `deny_unknown_fields` to `ProjectLockSetArgs`.
  - Owner: `crates/ralph-backend/src/session.rs`.
  - Acceptance: `rg \"ProjectLockSetArgs\" -n crates/ralph-backend/src/session.rs` shows `deny_unknown_fields` on its serde attrs.
- [x] Add `deny_unknown_fields` to tasks create/update DTOs.
  - Owner: `crates/ralph-backend/src/tasks.rs`.
  - Acceptance: `rg -n -C 2 \"pub struct TasksCreateArgs\" crates/ralph-backend/src/tasks.rs | rg \"deny_unknown_fields\"` and `rg -n -C 2 \"pub struct TasksUpdateArgs\" crates/ralph-backend/src/tasks.rs | rg \"deny_unknown_fields\"` both succeed.
- [x] Add `deny_unknown_fields` to tasks read/delete/status DTOs.
  - Owner: `crates/ralph-backend/src/tasks.rs`.
  - Acceptance: `rg -n -C 2 \"pub struct TasksSetStatusArgs\" crates/ralph-backend/src/tasks.rs | rg \"deny_unknown_fields\"` and `rg -n -C 2 \"pub struct TasksGetArgs\" crates/ralph-backend/src/tasks.rs | rg \"deny_unknown_fields\"` and `rg -n -C 2 \"pub struct TasksDeleteArgs\" crates/ralph-backend/src/tasks.rs | rg \"deny_unknown_fields\"` all succeed.
- [x] Add `deny_unknown_fields` to tasks signal mutation DTOs.
  - Owner: `crates/ralph-backend/src/tasks.rs`.
  - Acceptance: `rg -n -C 2 \"pub struct TasksSignalAddArgs\" crates/ralph-backend/src/tasks.rs | rg \"deny_unknown_fields\"` and `rg -n -C 2 \"pub struct TasksSignalUpdateArgs\" crates/ralph-backend/src/tasks.rs | rg \"deny_unknown_fields\"` and `rg -n -C 2 \"pub struct TasksSignalDeleteArgs\" crates/ralph-backend/src/tasks.rs | rg \"deny_unknown_fields\"` all succeed.
- [x] Add `deny_unknown_fields` to tasks signal list/summarize DTOs.
  - Owner: `crates/ralph-backend/src/tasks.rs`.
  - Acceptance: `rg -n -C 2 \"pub struct TasksSignalSummariesGetArgs\" crates/ralph-backend/src/tasks.rs | rg \"deny_unknown_fields\"` and `rg -n -C 2 \"pub struct TasksSignalCommentsListArgs\" crates/ralph-backend/src/tasks.rs | rg \"deny_unknown_fields\"` both succeed.
- [x] Add `deny_unknown_fields` to tasks ask/reply/comment DTOs.
  - Owner: `crates/ralph-backend/src/tasks.rs`.
  - Acceptance: `rg -n -C 2 \"pub struct TasksAskAnswerArgs\" crates/ralph-backend/src/tasks.rs | rg \"deny_unknown_fields\"` and `rg -n -C 2 \"pub struct TasksCommentReplyAddArgs\" crates/ralph-backend/src/tasks.rs | rg \"deny_unknown_fields\"` and `rg -n -C 2 \"pub struct TasksSignalCommentUpdateArgs\" crates/ralph-backend/src/tasks.rs | rg \"deny_unknown_fields\"` and `rg -n -C 2 \"pub struct TasksSignalCommentDeleteArgs\" crates/ralph-backend/src/tasks.rs | rg \"deny_unknown_fields\"` all succeed.
- [x] Add `deny_unknown_fields` to terminal session-start DTOs.
  - Owner: `crates/ralph-backend/src/terminal/contract.rs`.
  - Acceptance: `rg -n -C 2 \"pub struct TerminalBridgeStartSessionArgs\" crates/ralph-backend/src/terminal/contract.rs | rg \"deny_unknown_fields\"` and `rg -n -C 2 \"pub struct TerminalBridgeStartTaskSessionArgs\" crates/ralph-backend/src/terminal/contract.rs | rg \"deny_unknown_fields\"` and `rg -n -C 2 \"pub struct TerminalBridgeStartHumanSessionArgs\" crates/ralph-backend/src/terminal/contract.rs | rg \"deny_unknown_fields\"` all succeed.
- [x] Add `deny_unknown_fields` to terminal model-list DTOs.
  - Owner: `crates/ralph-backend/src/terminal/contract.rs`.
  - Acceptance: `rg -n -C 2 \"pub struct TerminalBridgeModelOption\" crates/ralph-backend/src/terminal/contract.rs | rg \"deny_unknown_fields\"` and `rg -n -C 2 \"pub struct TerminalBridgeListModelFormTreeResult\" crates/ralph-backend/src/terminal/contract.rs | rg \"deny_unknown_fields\"` both succeed.
- [x] Add `deny_unknown_fields` to terminal control/input DTOs.
  - Owner: `crates/ralph-backend/src/terminal/contract.rs`.
  - Acceptance: `rg -n -C 2 \"pub struct TerminalBridgeSendInputArgs\" crates/ralph-backend/src/terminal/contract.rs | rg \"deny_unknown_fields\"` and `rg -n -C 2 \"pub struct TerminalBridgeResizeArgs\" crates/ralph-backend/src/terminal/contract.rs | rg \"deny_unknown_fields\"` and `rg -n -C 2 \"pub struct TerminalBridgeSetStreamModeArgs\" crates/ralph-backend/src/terminal/contract.rs | rg \"deny_unknown_fields\"` all succeed.
- [x] Add `deny_unknown_fields` to terminal replay-output DTOs.
  - Owner: `crates/ralph-backend/src/terminal/contract.rs`.
  - Acceptance: `rg -n -C 2 \"pub struct TerminalBridgeReplayOutputArgs\" crates/ralph-backend/src/terminal/contract.rs | rg \"deny_unknown_fields\"` and `rg -n -C 2 \"pub struct TerminalBridgeReplayOutputResult\" crates/ralph-backend/src/terminal/contract.rs | rg \"deny_unknown_fields\"` both succeed.
- [x] Add `deny_unknown_fields` to prompt builder config section settings DTO.
  - Owner: `crates/sqlite-db/src/prompt_builder_configs.rs`.
  - Acceptance: `rg -n -C 2 \"pub struct SectionSettingsData\" crates/sqlite-db/src/prompt_builder_configs.rs | rg \"deny_unknown_fields\"` succeeds.
- [x] Add `deny_unknown_fields` to prompt builder config DTOs.
  - Owner: `crates/sqlite-db/src/prompt_builder_configs.rs`.
  - Acceptance: `rg -n -C 2 \"pub struct PromptBuilderConfigInput\" crates/sqlite-db/src/prompt_builder_configs.rs | rg \"deny_unknown_fields\"` and `rg -n -C 2 \"pub struct PromptBuilderConfigData\" crates/sqlite-db/src/prompt_builder_configs.rs | rg \"deny_unknown_fields\"` both succeed.
- [x] Add `deny_unknown_fields` to task signal DTOs.
  - Owner: `crates/sqlite-db/src/types.rs`.
  - Acceptance: `rg -n -C 2 \"pub struct TaskSignal\" crates/sqlite-db/src/types.rs | rg \"deny_unknown_fields\"` and `rg -n -C 2 \"pub struct TaskSignalSummary\" crates/sqlite-db/src/types.rs | rg \"deny_unknown_fields\"` both succeed.
- [x] Add `deny_unknown_fields` to MCP config DTO.
  - Owner: `crates/sqlite-db/src/types.rs`.
  - Acceptance: `rg -n -C 2 \"pub struct McpServerConfig\" crates/sqlite-db/src/types.rs | rg \"deny_unknown_fields\"` succeeds.
- [x] Add `deny_unknown_fields` to task DTOs.
  - Owner: `crates/sqlite-db/src/types.rs`.
  - Acceptance: `rg -n -C 2 \"pub struct Task\" crates/sqlite-db/src/types.rs | rg \"deny_unknown_fields\"` and `rg -n -C 2 \"pub struct TaskListItem\" crates/sqlite-db/src/types.rs | rg \"deny_unknown_fields\"` both succeed.
- [x] Add `deny_unknown_fields` to task template DTO.
  - Owner: `crates/sqlite-db/src/types.rs`.
  - Acceptance: `rg -n -C 2 \"pub struct TaskTemplate\" crates/sqlite-db/src/types.rs | rg \"deny_unknown_fields\"` succeeds.
- [x] Add `deny_unknown_fields` to subsystem comment DTO.
  - Owner: `crates/sqlite-db/src/types.rs`.
  - Acceptance: `rg -n -C 2 \"pub struct SubsystemComment\" crates/sqlite-db/src/types.rs | rg \"deny_unknown_fields\"` succeeds.
- [x] Add `deny_unknown_fields` to agent session DTOs.
  - Owner: `crates/sqlite-db/src/types.rs`.
  - Acceptance: `rg -n -C 2 \"pub struct AgentSession\" crates/sqlite-db/src/types.rs | rg \"deny_unknown_fields\"` and `rg -n -C 2 \"pub struct AgentSessionCreateInput\" crates/sqlite-db/src/types.rs | rg \"deny_unknown_fields\"` and `rg -n -C 2 \"pub struct AgentSessionUpdateInput\" crates/sqlite-db/src/types.rs | rg \"deny_unknown_fields\"` all succeed.
- [x] Add `deny_unknown_fields` to task signal comment DTOs.
  - Owner: `crates/sqlite-db/src/types.rs`.
  - Acceptance: `rg -n -C 2 \"pub struct TaskSignalComment\" crates/sqlite-db/src/types.rs | rg \"deny_unknown_fields\"` and `rg -n -C 2 \"pub struct TaskSignalCommentCreateInput\" crates/sqlite-db/src/types.rs | rg \"deny_unknown_fields\"` both succeed.
- [x] Add `deny_unknown_fields` to `ProjectScanArgs`.
  - Owner: `src-tauri/src/commands/project.rs`.
  - Acceptance: `rg \"ProjectScanArgs\" -n src-tauri/src/commands/project.rs` shows `deny_unknown_fields` on its serde attrs.
- [x] Add `deny_unknown_fields` to `AgentSessionsByIdArgs`.
  - Owner: `src-tauri/src/commands/agent_sessions.rs`.
  - Acceptance: `rg \"AgentSessionsByIdArgs\" -n src-tauri/src/commands/agent_sessions.rs` shows `deny_unknown_fields` on its serde attrs.
- [x] Add at least one strict-decode regression test for each crate that decodes remote RPC results (`src-tauri`, `ralph-backend`, `sqlite-db`).
  - Owner: (proposed) `src-tauri/tests/remote_strict_decode_contract_test.rs` (new).
  - Acceptance: `cargo test --manifest-path src-tauri/Cargo.toml --test remote_strict_decode_contract_test` rejects unknown fields in at least one result DTO.

### 3.4 Required Collections Must Serialize Even When Empty
- [x] `Task` has a serialization-shape test for required arrays.
  - Owner: `crates/sqlite-db/src/types.rs`.
  - Acceptance: `cargo test -p sqlite-db`.
- [x] `McpServerConfig` has a serialization-shape test for `args` and `env`.
  - Owner: `crates/sqlite-db/src/types.rs`.
  - Acceptance: `cargo test -p sqlite-db`.
- [x] Add serialization-shape test for `DisciplineConfig` required arrays (`skills`, `mcpServers`, `taskTemplates`).
  - Owner: `crates/ralph-backend/src/disciplines_contract.rs`.
  - Acceptance: `cargo test -p ralph-backend`.
- [x] Add serialization-shape test for `SubsystemData.comments` required array.
  - Owner: `crates/ralph-backend/src/subsystems_contract.rs`.
  - Acceptance: `cargo test -p ralph-backend`.
- [x] Inventory `skip_serializing_if = \"Vec::is_empty\"` / `HashMap::is_empty` usages in any `#[ipc_type]` DTOs.
  - Owner: `.docs/077_IPC_SWAP_READINESS_CHECKLIST.md`.
  - Acceptance: this section contains an explicit list of `rg` hits (file + line) and whether each field is required vs optional.

#### `skip_serializing_if = "Vec::is_empty"` / `HashMap::is_empty` inventory
- Command: `rg -n 'skip_serializing_if = \"(Vec|HashMap)::is_empty\"' crates src-tauri`
- Hits:
  - `crates/sqlite-db/src/types.rs:370`: `Subsystem.comments` (not `#[ipc_type]`; optional/omitted-when-empty shape)
  - `crates/sqlite-db/src/types.rs:424`: `Discipline.skills` (not `#[ipc_type]`; optional/omitted-when-empty shape)
  - `crates/sqlite-db/src/types.rs:428`: `Discipline.mcp_servers` (not `#[ipc_type]`; optional/omitted-when-empty shape)
- Result: no `skip_serializing_if = "Vec::is_empty"` / `HashMap::is_empty` usages found in `#[ipc_type]` DTOs.

### 3.5 64-bit Integers On The Wire
- [x] Decision: standardize 64-bit integer representation (JSON number vs string) for all wire fields that can exceed 2^53.
  - Owner: `.docs/077_IPC_SWAP_READINESS_CHECKLIST.md`.
  - Acceptance: decision recorded + list of impacted fields.

#### Decision
- v1 policy: keep 64-bit integers serialized as JSON numbers, but treat values above `Number.MAX_SAFE_INTEGER` as contract violations.
- Callers must hard-fail when asked to send an out-of-range value; decoders must not silently truncate.
- This is a stopgap until we migrate specific fields to string encoding (no compat shims).

Impacted fields (current `u64` on Rust side, `bigint` in TS bindings):
- `RemoteWireFrame.id`
- Terminal stream sequencing:
  - `PtyOutputEvent.seq`
  - `TerminalBridgeReplayOutputArgs.afterSeq`
  - `TerminalBridgeReplayOutputChunk.seq`
  - `TerminalBridgeReplayOutputResult.truncatedUntilSeq`
- [ ] Implement chosen encoding for `RemoteWireFrame.id`.
  - Owner: `crates/ralph-contracts/src/transport.rs`.
  - Acceptance: rust tests + TS bindings match.
- [ ] Implement chosen encoding for terminal sequence fields (`seq`, `afterSeq`, `truncatedUntilSeq`).
  - Owner: `crates/ralph-backend/src/terminal/contract.rs`.
  - Acceptance: backend + frontend drift tests updated; no silent truncation.

## 4. Error Model (Must-Have)

### 4.1 Error Envelope Decision
- [x] Decision: keep IPC errors as `Result<T, String>` or migrate to a structured error payload.
  - Owner: `.docs/077_IPC_SWAP_READINESS_CHECKLIST.md`.
  - Acceptance: decision recorded + migration plan (no compat shims for structured errors).

#### Decision
- Keep IPC errors as `Result<T, String>` (coded `[R-XXXX] ...`) for v1 swap readiness.
- Defer structured errors until after `ralphd` parity is stable; migration will be all-or-nothing (no compat shims).

### 4.2 Minimum Standard (While Errors Are Strings)
- [x] Standard string error format is machine-parsable (stable error code + message).
  - Owner: `crates/ralph-errors` (`[R-XXXX] message` + `parse_ralph_error`).
  - Acceptance: `cargo test -p ralph-errors`.
- [x] Enforce coded errors in Tauri project commands (no raw `format!(...)` / `e.to_string()` on error paths).
  - Owner: `src-tauri/src/commands/project.rs`.
  - Acceptance: `rg \"Err\\(.*to_string\\(\\)\\)|map_err\\(\\|e\\| e\\.to_string\\(\\)\\)\" src-tauri/src/commands/project.rs` has no hits.
- [x] Enforce coded errors in Tauri tasks commands.
  - Owner: `src-tauri/src/commands/tasks.rs`.
  - Acceptance: `rg \"Err\\(.*to_string\\(\\)\\)|map_err\\(\\|e\\| e\\.to_string\\(\\)\\)\" src-tauri/src/commands/tasks.rs` has no hits.
- [x] Enforce coded errors in Tauri subsystems/disciplines commands.
  - Owner: `src-tauri/src/commands/subsystems.rs`.
  - Acceptance: `rg \"Err\\(.*to_string\\(\\)\\)|map_err\\(\\|e\\| e\\.to_string\\(\\)\\)\" src-tauri/src/commands/subsystems.rs` has no hits.
- [x] Enforce coded errors in Tauri prompt builder commands.
  - Owner: `src-tauri/src/commands/prompts.rs`.
  - Acceptance: `rg \"Err\\(.*to_string\\(\\)\\)|map_err\\(\\|e\\| e\\.to_string\\(\\)\\)\" src-tauri/src/commands/prompts.rs` has no hits.
- [x] Enforce coded errors in Tauri terminal bridge commands.
  - Owner: `src-tauri/src/commands/terminal_bridge.rs`.
  - Acceptance: `rg \"Err\\(.*to_string\\(\\)\\)|map_err\\(\\|e\\| e\\.to_string\\(\\)\\)\" src-tauri/src/commands/terminal_bridge.rs` has no hits.
- [x] Enforce coded errors in `ralphd` RPC server.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `rg \"RpcErr\" -n src-daemon/src/main.rs` shows coded `[R-XXXX]` strings only.

### 4.3 If Moving To Structured Errors (All-Or-Nothing)
- [ ] Define one canonical structured error DTO and export it to TS.
  - Owner: (proposed) `crates/ralph-contracts/src/error.rs`.
  - Acceptance: `just types` exports it and all call sites can consume it.
- [ ] Update remote wire `RpcErr` to carry the structured error payload (no string-only compat).
  - Owner: `crates/ralph-contracts/src/transport.rs`.
  - Acceptance: `cargo test -p ralph-contracts`.
- [ ] Update frontend invoke boundary to surface structured errors (no silent fallback to string).
  - Owner: `src/lib/tauri/invoke.ts`.
  - Acceptance: `bun test:run` (or a new unit test for error surface).

## 5. Single Transport Adapter Boundary in Frontend (Swap Enabler)
- [x] All frontend command calls go through one module boundary.
  - Owner: `src/lib/tauri/invoke.ts` (only `@tauri-apps/api/core` import in `src/**`).
  - Acceptance: `rg \"@tauri-apps/api/core\" src` only matches that file.
- [x] All frontend event subscriptions go through one module boundary.
  - Owner: `src/lib/tauri/events.ts` (only `@tauri-apps/api/event` import in `src/**`).
  - Acceptance: `rg \"@tauri-apps/api/event\" src` only matches that file.
- [x] Direct `@tauri-apps/api/*` usage is banned outside the boundary modules.
  - Owner: `src/lib/tauri/`.
  - Acceptance: `rg \"@tauri-apps/api/\" src | rg -v \"src/lib/tauri/(invoke|events|window)\\.ts\"` has no hits.
- [x] Add a CI/test gate that fails if forbidden `@tauri-apps/api/*` imports are introduced outside boundary modules.
  - Owner: (proposed) `src/lib/tauri/tauriImportBoundary.test.ts` (new).
  - Acceptance: `bun test:run src/lib/tauri/tauriImportBoundary.test.ts`.

## 6. Single “Backend Service” Boundary in Rust (Swap Enabler)

### 6.1 Transport-Agnostic Service Layer (Per Domain)
- [x] Project lock/session logic is backend-owned and reused by Tauri + `ralphd`.
  - Owner: `crates/ralph-backend/src/session.rs`.
  - Acceptance: `cargo test -p ralph-backend`.
- [x] Project validate/initialize logic is backend-owned and reused by Tauri + `ralphd`.
  - Owner: `crates/ralph-backend/src/project.rs`.
  - Acceptance: `cargo test -p ralph-backend`.
- [x] Project recent-list logic is backend-owned and reused by Tauri + `ralphd`.
  - Owner: `crates/ralph-backend/src/project_scan.rs`.
  - Acceptance: `cargo test -p ralph-backend`.
- [x] Project scanning logic is backend-owned and reused by Tauri + `ralphd`.
  - Owner: `crates/ralph-backend/src/project_scan.rs`.
  - Acceptance: `cargo test -p ralph-backend`.
- [x] Project info read logic is backend-owned and reused by Tauri + `ralphd`.
  - Owner: `crates/ralph-backend/src/project_scan.rs`.
  - Acceptance: `cargo test -p ralph-backend`.
- [x] Tasks domain logic is backend-owned and reused by Tauri + `ralphd`.
  - Owner: `crates/ralph-backend/src/tasks.rs`.
  - Acceptance: `cargo test -p ralph-backend`.
- [x] Prompt builder preview logic is backend-owned and reused by Tauri + `ralphd`.
  - Owner: `crates/ralph-backend/src/prompt_builder_preview.rs`.
  - Acceptance: `cargo test -p ralph-backend`.
- [x] Prompt builder config CRUD logic is backend-owned and reused by Tauri + `ralphd`.
  - Owner: `crates/ralph-backend/src/prompt_builder_configs_service.rs`.
  - Acceptance: `cargo test -p ralph-backend`.
- [x] Subsystems domain logic is backend-owned and reused by Tauri + `ralphd`.
  - Owner: `crates/ralph-backend/src/subsystems_service.rs`.
  - Acceptance: `cargo test -p ralph-backend`.
- [x] Disciplines domain logic is backend-owned and reused by Tauri + `ralphd`.
  - Owner: `crates/ralph-backend/src/disciplines_service.rs`.
  - Acceptance: `cargo test -p ralph-backend`.
- [x] Terminal manager is backend-owned (PTY state + buffering + replay semantics).
  - Owner: `crates/ralph-backend/src/terminal/`.
  - Acceptance: `cargo test -p ralph-backend`.
- [x] Terminal bridge adapter is backend-owned (invoke entrypoints to the terminal manager).
  - Owner: `crates/ralph-backend/src/terminal_bridge.rs`.
  - Acceptance: `cargo test -p ralph-backend`.
- [x] Agent sessions domain logic is backend-owned (service + DTOs) even if the UI-only commands remain local.
  - Owner: `crates/ralph-backend/src/agent_sessions_service.rs`.
  - Acceptance: `cargo test -p ralph-backend`.
- [x] API server logic is transport-agnostic and does not hard-depend on Tauri runtime types.
  - Owner: `src-tauri/src/api_server.rs`.
  - Acceptance: `cargo test --manifest-path src-tauri/Cargo.toml`.

### 6.2 Injected Event Sink Interface
- [x] Event sink trait exists.
  - Owner: `crates/ralph-contracts/src/transport.rs` (`EventSink`).
  - Acceptance: `cargo test -p ralph-contracts`.
- [x] Tauri `EventSink` implementation exists.
  - Owner: `src-tauri/src/event_sink.rs`.
  - Acceptance: `cargo test --manifest-path src-tauri/Cargo.toml`.
- [ ] `ralphd` `EventSink` implementation broadcasts events over WS as `RemoteWireFrame::Event`.
  - Owner: (proposed) `src-daemon/src/event_sink.rs` (new).
  - Acceptance: remote events arrive in Tauri remote-mode and re-emit locally.
- [x] Replace `api-server-error` direct `AppHandle.emit(...)` with the sink interface.
  - Owner: `src-tauri/src/api_server.rs`.
  - Acceptance: `rg \"api-server-error\" -n src-tauri/src/api_server.rs` has no hits.
- [x] Replace `signal-added` direct `AppHandle.emit(...)` with the sink interface.
  - Owner: `src-tauri/src/api_server.rs`.
  - Acceptance: `rg \"signal-added\" -n src-tauri/src/api_server.rs` has no hits.
- [x] Verify no direct `AppHandle.emit(...)` usage remains outside the sink implementation.
  - Owner: `src-tauri/src/event_sink.rs`.
  - Acceptance: `rg \"\\.emit\\(\" src-tauri/src | rg -v \"src-tauri/src/event_sink\\.rs\"` has no hits.

### 6.3 Injected RPC Client Interface (Remote Proxy)
- [x] Invoke-style RPC client trait exists (for proxying in remote mode).
  - Owner: `crates/ralph-contracts/src/transport.rs` (`RpcClient`).
  - Acceptance: `cargo test -p ralph-contracts`.
- [x] Tauri remote-mode adapter implements WS RPC + event pump.
  - Owner: `src-tauri/src/remote.rs` (`RemoteRpcClient` + `RemoteWireFrameConnection`).
  - Acceptance: `cargo test --manifest-path src-tauri/Cargo.toml`.

### 6.4 `ralphd` Headless `RemoteWireFrame` Server (Atomic Tasks)
- [x] Crate lives at `src-daemon/` (not `crates/ralphd`).
  - Owner: `src-daemon/Cargo.toml`.
  - Acceptance: `cargo metadata --no-deps | rg '\"name\":\"ralphd\"'` has a hit.
- [x] Accept WS connections and read/write text frames.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo build -p ralphd`.
- [x] Strictly decode `RemoteWireFrame` (unknown fields hard-fail).
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (protocol/strict decode tests).
- [x] Reject client-sent non-request frames (`Event`/`RpcOk`/`RpcErr`).
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (protocol tests).
- [x] Support async RPC handlers (needed for embedding + terminal).
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo build -p ralphd`.
- [x] RPC `protocol_version_get`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `project_validate_path`. (Nice-to-have for v1 parity.)
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `project_lock_set`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `project_lock_get`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `project_initialize`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `project_recent_list`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `project_scan`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `project_info_get`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `tasks_get`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `tasks_list_items`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `tasks_update`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `tasks_set_status`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `tasks_signal_add`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `tasks_signal_update`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `tasks_signal_delete`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `tasks_signal_summaries_get`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `tasks_comment_reply_add`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `tasks_create`. (Nice-to-have for v1 parity.)
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `tasks_delete`. (Nice-to-have for v1 parity.)
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `tasks_list`. (Nice-to-have for v1 parity.)
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `tasks_ask_answer`. (Nice-to-have for v1 parity.)
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `tasks_signal_comment_add`. (Nice-to-have for v1 parity.)
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `tasks_signal_comment_update`. (Nice-to-have for v1 parity.)
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `tasks_signal_comment_delete`. (Nice-to-have for v1 parity.)
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `tasks_signal_comments_list`. (Nice-to-have for v1 parity.)
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `subsystems_list`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `subsystems_create`. (Nice-to-have for v1 parity.)
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `subsystems_update`. (Nice-to-have for v1 parity.)
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `subsystems_delete`. (Nice-to-have for v1 parity.)
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `subsystems_comment_add`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `subsystems_comment_update`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `subsystems_comment_delete`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `disciplines_list`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `disciplines_create`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `disciplines_update`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `disciplines_delete`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `disciplines_cropped_image_get`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `disciplines_image_data_get`. (Nice-to-have for v1 parity.)
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [ ] RPC `prompt_builder_preview`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `prompt_builder_config_list`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `prompt_builder_config_get`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `prompt_builder_config_save`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [x] RPC `prompt_builder_config_delete`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [ ] RPC `terminal_start_session`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [ ] RPC `terminal_start_task_session`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [ ] RPC `terminal_start_human_session`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [ ] RPC `terminal_list_model_form_tree`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [ ] RPC `terminal_send_input`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [ ] RPC `terminal_resize`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [ ] RPC `terminal_set_stream_mode`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [ ] RPC `terminal_replay_output`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [ ] RPC `terminal_terminate`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (RPC smoke tests).
- [ ] Event stream: emit `backend-diagnostic`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (event smoke tests).
- [ ] Event stream: emit `terminal:output`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (event smoke tests).
- [ ] Event stream: emit `terminal:closed`.
  - Owner: `src-daemon/src/main.rs`.
  - Acceptance: `cargo test -p ralphd` (event smoke tests).
- [x] Add `src-daemon` integration test harness for WS `RemoteWireFrame` roundtrips.
  - Owner: (proposed) `src-daemon/tests/ws_roundtrip_test.rs` (new).
  - Acceptance: `cargo test -p ralphd --test ws_roundtrip_test`.
- [x] Add RPC smoke test: `protocol_version_get` yields `RpcOk`.
  - Owner: (proposed) `src-daemon/tests/ws_rpc_smoke_test.rs` (new).
  - Acceptance: `cargo test -p ralphd --test ws_rpc_smoke_test`.
- [x] Add RPC smoke test: unknown command yields `RpcErr`.
  - Owner: (proposed) `src-daemon/tests/ws_rpc_smoke_test.rs` (new).
  - Acceptance: `cargo test -p ralphd --test ws_rpc_smoke_test`.
- [ ] Add protocol smoke test: client-sent non-request frames hard-fail.
  - Owner: (proposed) `src-daemon/tests/ws_protocol_smoke_test.rs` (new).
  - Acceptance: `cargo test -p ralphd --test ws_protocol_smoke_test`.
- [ ] Add event smoke test: `backend-diagnostic` frame is delivered (once events are implemented).
  - Owner: (proposed) `src-daemon/tests/ws_event_smoke_test.rs` (new).
  - Acceptance: `cargo test -p ralphd --test ws_event_smoke_test`.

### 6.5 Tauri Commands Must Stay Thin (Adapter-Only)
- [x] Extract project scanning logic out of the Tauri command adapter.
  - Owner: `src-tauri/src/commands/project.rs`.
  - Acceptance: `rg \"fn scan_recursive\" -n src-tauri/src/commands/project.rs` has no hits.
- [x] Extract recent-projects persistence policy out of the Tauri command adapter (so `ralphd` can serve it too).
  - Owner: `src-tauri/src/commands/project.rs`.
  - Acceptance: `rg \"recent_projects\" -n src-tauri/src/commands/project.rs` has no hits.
- [x] Extract project info read/mapping out of the Tauri command adapter (so `ralphd` can serve it too).
  - Owner: `src-tauri/src/commands/project.rs`.
  - Acceptance: `rg \"db\\.get_project_info\\(\" -n src-tauri/src/commands/project.rs` has no hits.
- [x] Move agent-sessions domain logic out of the Tauri command adapter (so remote-mode does not depend on direct DB access).
  - Owner: `src-tauri/src/commands/agent_sessions.rs`.
  - Acceptance: `rg \"\\.db\\(\" -n src-tauri/src/commands/agent_sessions.rs` has no hits.
- [x] Remote connect/disconnect/status commands are control-plane only (no domain logic).
  - Owner: `src-tauri/src/commands/remote.rs`.
  - Acceptance: `rg \"SqliteDb|subsystems_service|disciplines_service|tasks_\" -n src-tauri/src/commands/remote.rs` has no hits.

## 7. Streaming/Terminal Contract Hardening (Must-Have For Headless Parity)

### 7.1 Backend Is Canonical
- [x] Keep the “withhold + replay” model authoritative in the backend.
  - Owner: `crates/ralph-backend/src/terminal/manager.rs`.
  - Acceptance: backend tests pass.

### 7.2 Contract Documentation (Not Just Implementation)
- [x] Document `sessionId` uniqueness rules.
  - Owner: `.docs/077_IPC_SWAP_READINESS_CHECKLIST.md`.
  - Acceptance: a dedicated “Terminal Contract” subsection exists and includes `sessionId` rules.
- [x] Document `seq` monotonicity rules.
  - Owner: `.docs/077_IPC_SWAP_READINESS_CHECKLIST.md`.
  - Acceptance: a dedicated “Terminal Contract” subsection exists and includes `seq` rules.
- [x] Document truncation signaling (`truncated`, `truncatedUntilSeq`).
  - Owner: `.docs/077_IPC_SWAP_READINESS_CHECKLIST.md`.
  - Acceptance: a dedicated “Terminal Contract” subsection exists and includes truncation signaling.
- [x] Document replay limits + ordering guarantees.
  - Owner: `.docs/077_IPC_SWAP_READINESS_CHECKLIST.md`.
  - Acceptance: a dedicated “Terminal Contract” subsection exists and includes replay limits/ordering.

#### Terminal Contract (Canonical)
- `sessionId` uniqueness:
  - The controller that starts a terminal session chooses `sessionId`.
  - `sessionId` must be unique among currently-live sessions; attempting to start a session with an already-live `sessionId` must hard-fail (no implicit takeover).
  - `sessionId` identity is stable for the lifetime of the session and is used as the join key for all output/closed events.
- `seq` monotonicity:
  - Output is a per-session ordered stream; each emitted chunk increments `seq` for that `sessionId`.
  - `seq` is strictly increasing for a given `sessionId` (no duplicates, no reordering).
  - `seq` is not required to be contiguous (gaps are allowed only if the backend explicitly signals truncation).
- Truncation signaling:
  - Replay results may set `truncated=true` when earlier output is no longer available.
  - When `truncated=true`, `truncatedUntilSeq` communicates the earliest still-available `seq` (any `seq < truncatedUntilSeq` is permanently unavailable for replay).
- Replay limits + ordering:
  - Replay returns chunks in ascending `seq` order.
  - Replay is best-effort bounded by `limit`; `hasMore=true` indicates there is more output after the last returned chunk.

### 7.3 Multi-Client Attach Policy
- [x] Decision: single-controller vs multi-attach policy (v1 likely: hard-fail extra controllers).
  - Owner: `.docs/077_IPC_SWAP_READINESS_CHECKLIST.md`.
  - Acceptance: decision recorded.

#### Decision
- v1: single-controller. Any attempt to attach a second controller to an active `sessionId` must hard-fail.
- [ ] Implement chosen attach policy (hard-fail path must be explicit).
  - Owner: `crates/ralph-backend/src/terminal/manager.rs`.
  - Acceptance: unit tests cover rejection behavior.

## 8. Domain Policy Ownership Audit (Swap Enabler)

Canonical checklist owner: this section (moved from `.docs/067_FRONTEND_LOGIC_BACKEND_AUDIT_CHECKLIST.md` to avoid duplication).

### 8.1 Prompt Builder Domain Ownership
- [x] Decision: canonical owner of prompt-builder section metadata is backend.
  - Owner: `.docs/077_IPC_SWAP_READINESS_CHECKLIST.md`.
  - Acceptance: decision recorded (including how frontend consumes it).
- [ ] Add `enrichment_instructions` section metadata entry to the frontend registry (stopgap until duplication is eliminated).
  - Owner: `src/lib/prompt-builder-registry.ts`.
  - Acceptance: `rg \"enrichment_instructions\" src/lib/prompt-builder-registry.ts` has hits in both metadata and instructions text.
- [ ] Fix backend section metadata category naming (`feature` vs `subsystem`) to match frontend taxonomy (or document canonical mapping).
  - Owner: `crates/prompt-builder/src/sections/metadata.rs`.
  - Acceptance: a unit test (or doc note) proves the mapping is stable and intentional.
- [x] Decision: canonical owner of prompt-builder recipe definitions is backend.
  - Owner: `.docs/077_IPC_SWAP_READINESS_CHECKLIST.md`.
  - Acceptance: decision recorded (including how frontend consumes it).
- [x] Decision: canonical owner of default instruction bodies is backend.
  - Owner: `.docs/077_IPC_SWAP_READINESS_CHECKLIST.md`.
  - Acceptance: decision recorded (including how frontend consumes overrides).

#### Decision Notes
- Canonical owner for prompt-builder section metadata/recipes/default instruction bodies is backend.
- Frontend consumes backend-owned registries by rendering and allowing explicit overrides only (no parallel source-of-truth).

### 8.2 Prompt Preview Assembly
- [ ] Backend preview is authoritative: it returns final `sections` + `fullPrompt` exactly as used (including user input insertion).
  - Owner: `crates/ralph-backend/src/prompt_builder_preview.rs`.
  - Acceptance: `cargo test -p ralph-backend` includes a unit test asserting `fullPrompt` matches `sections` order (including user input).
- [ ] Stop client-side section splicing for `user_input` in preview UI.
  - Owner: `src/hooks/prompt-builder/usePromptPreview.ts`.
  - Acceptance: `rg \"rebuildPreviewWithUserInput\" src/hooks/prompt-builder/usePromptPreview.ts` has no hits.

### 8.3 Naming and Canonicalization Invariants
- [ ] Define and enforce subsystem-name validation in backend (reject `/`, `:`, `\\`) for task create/update.
  - Owner: `crates/ralph-backend/src/tasks.rs`.
  - Acceptance: `cargo test -p ralph-backend` includes a unit test that invalid names hard-fail with a coded error.
- [ ] Define and enforce subsystem-name normalization in backend (mirror `normalizeFeatureName` semantics) for task create/update.
  - Owner: `crates/ralph-backend/src/tasks.rs`.
  - Acceptance: `cargo test -p ralph-backend` includes a unit test that normalization is applied.
- [ ] Keep frontend normalization as UX-only (it may still normalize, but correctness must not depend on it).
  - Owner: `src/lib/schemas/taskSchema.ts`.
  - Acceptance: doc note (or code comment) explicitly states backend is canonical for normalization/validation.

### 8.4 Session Launch Resolution Policy
- [x] Decision: canonical launch precedence policy (task -> discipline -> user default) is backend-owned.
  - Owner: `.docs/077_IPC_SWAP_READINESS_CHECKLIST.md`.
  - Acceptance: decision recorded.

#### Decision
- Canonical precedence: `task` overrides `discipline` overrides `user default`.
- If an override references an unknown agent/model/effort combination, backend must hard-fail (no coercion).
- [ ] Add backend DTO for resolved launch config + provenance metadata (what won, and why).
  - Owner: (proposed) `crates/ralph-backend/src/terminal/contract.rs`.
  - Acceptance: `just types-check` exports the DTO exactly once.
- [ ] Implement backend launch resolver (precedence + validation + explicit hard-fail paths).
  - Owner: (proposed) `crates/ralph-backend/src/terminal/session.rs`.
  - Acceptance: `cargo test -p ralph-backend` includes unit tests for precedence + rejection paths.
- [ ] Update terminal start commands to use backend resolver output (no frontend-owned fallback policy).
  - Owner: `src-tauri/src/commands/terminal_bridge.rs`.
  - Acceptance: `rg \"resolveLaunchConfig\" -n src-tauri/src/commands/terminal_bridge.rs` has no hits.
- [ ] Keep frontend as display-only for launch-source labels (render backend provenance, don’t re-resolve).
  - Owner: `src/components/workspace/task-detail/hooks/useResolvedTaskLaunch.ts`.
  - Acceptance: the hook no longer computes precedence; it renders backend-provided provenance.
- [ ] Remove fallback model/effort coercion policy from frontend (or make it an explicit UI-only suggestion).
  - Owner: `src/components/agent-session-launch/resolveLaunchConfig.ts`.
  - Acceptance: the function either disappears or is explicitly non-authoritative (no correctness coupling).

### 8.5 Stack Catalog Source Of Truth
- [ ] Remove hardcoded stack list from project selector.
  - Current frontend hardcoded list: `src/components/app-shell/ProjectSelector.tsx`
  - Backend provider: `stacks_metadata_list` in `src-tauri/src/commands/subsystems.rs`
  - Reason: stack definitions are backend/domain data and should not be duplicated.
  - Owner: `src/components/app-shell/ProjectSelector.tsx`.
  - Acceptance: `rg \"stack\" src/components/app-shell/ProjectSelector.tsx` shows it loads from IPC, not hardcoded literals.

### 8.6 Nice-to-Have Boundary Tightening
- [ ] Decision: backend-provided stats payloads for feature/discipline/project progress (only if these become shared product semantics).
  - Current frontend derived stats: `src/lib/stats.ts`, `src/hooks/features/useFeatureStats.ts`, `src/hooks/disciplines/useDisciplineStats.ts`
  - Existing backend types indicate domain intent: `crates/sqlite-db/src/types.rs` (`GroupStats`, `ProjectProgress`)
  - Owner: `.docs/077_IPC_SWAP_READINESS_CHECKLIST.md`.
  - Acceptance: decision recorded + scope of canonical stats (if any).

## 9. Parity/Drift Test Suite (Must-Have)

### 9.1 Non-GUI Contract Test Harness
- [ ] Add a contract test suite that runs without a GUI (Rust-only).
  - Owner: (proposed) `crates/ralph-contracts/tests/contract_suite_test.rs`.
  - Acceptance: `cargo test -p ralph-contracts` covers DTO serialization + strict decode.
- [ ] Add a contract test suite that runs without a GUI (frontend-only).
  - Owner: (proposed) `src/lib/tauri/contractSuite.test.ts`.
  - Acceptance: `bun test:run` covers event name drift + critical TS types.

### 9.2 Existing Drift Coverage
- [x] Rust terminal contract tests exist.
  - Owner: `crates/ralph-contracts/src/terminal.rs`.
  - Acceptance: `cargo test -p ralph-contracts`.
- [x] Frontend terminal drift tests exist.
  - Owner: `src/lib/terminal/terminalBridgeContract.test.ts`.
  - Acceptance: `bun test:run src/lib/terminal/terminalBridgeContract.test.ts`.
- [x] Rust backend-diagnostic contract tests exist.
  - Owner: `crates/ralph-contracts/src/events.rs`.
  - Acceptance: `cargo test -p ralph-contracts`.
- [x] Frontend backend-diagnostic drift tests exist.
  - Owner: `src/lib/tauri/eventsContract.test.ts`.
  - Acceptance: `bun test:run src/lib/tauri/eventsContract.test.ts`.

### 9.3 Extend Drift Coverage (Atomic Additions)
- [ ] When a new event constant is added, add it to `FRONTEND_EVENT_NAMES`.
  - Owner: `crates/ralph-contracts/src/frontend.rs`.
  - Acceptance: `cargo test -p ralph-contracts`.
- [ ] When a new frontend-listened event is added, update the TS drift test to match the Rust list.
  - Owner: `src/lib/tauri/eventsContract.test.ts`.
  - Acceptance: `bun test:run src/lib/tauri/eventsContract.test.ts`.
- [x] Add serialization-shape tests for high fan-out task DTOs returned by v1 parity RPC.
  - Owner: `crates/sqlite-db/src/types.rs`.
  - Acceptance: `cargo test -p sqlite-db`.
- [x] Add serialization-shape tests for high fan-out disciplines DTOs returned by v1 parity RPC.
  - Owner: `crates/ralph-backend/src/disciplines_contract.rs`.
  - Acceptance: `cargo test -p ralph-backend`.
- [x] Add serialization-shape tests for high fan-out subsystems DTOs returned by v1 parity RPC.
  - Owner: `crates/ralph-backend/src/subsystems_contract.rs`.
  - Acceptance: `cargo test -p ralph-backend`.

### 9.4 Single “Contract CI Gate”
- [x] Add `just verify-contract` that runs: `types-check`, Rust contract tests, and frontend drift tests.
  - Owner: `justfile`.
  - Acceptance: `just verify-contract`.
- [x] Add CI config that runs `just verify-contract` on every PR.
  - Owner: `.github/workflows/verify-contract.yml`.
  - Acceptance: CI fails on stale types/snapshots/drift tests.

## 10. “Ready For Swap” Definition Of Done

Turn the definition of done into executable gates + one manual smoke runbook.

### 10.1 Executable Gates
- [ ] Add `just verify-swap` that runs: `types-check`, contract tests, and `ralphd` WS smoke tests.
  - Owner: `justfile`.
  - Acceptance: `just verify-swap`.
- [ ] Add a `ralphd` parity smoke test that exercises the v1 MUST command subset over WS.
  - Owner: `src-daemon` tests (new).
  - Acceptance: `cargo test -p ralphd`.

### 10.2 Manual Smoke Runbook (Only For UX Checks)
- [ ] Write a swap smoke runbook: run Tauri in remote mode, connect to `ralphd`, and verify the core UI paths (project lock, tasks CRUD, terminal output).
  - Owner: `.docs/077_IPC_SWAP_READINESS_CHECKLIST.md`.
  - Acceptance: runbook section exists with exact commands and expected output.
