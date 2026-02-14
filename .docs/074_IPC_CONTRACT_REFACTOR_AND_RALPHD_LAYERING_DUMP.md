# IPC Contract Refactor (Breaking) + Ralphd Remote Layering Dump

Date: 2026-02-14

## 1. Context
1. Today is the best time to break the contract: we are mid-development and want extensibility.
2. The current transport (Tauri IPC) exists only because the UI WebView and Rust backend are the same desktop app process.
3. Headless + remote requires a transport-agnostic contract and a server process (`ralphd`) that can run without a GUI.

## 2. Goal
1. Improve the IPC contract now (breaking, no backwards compatibility).
2. Fix all callsites to match the new contract.
3. Make the contract easier to mirror 1:1 in `ralphd` later, with minimal translation.

## 3. Core Contract Decisions (Canonical)
1. JSON casing for JS-facing types: `camelCase`.
   1. Rust wire types use `#[serde(rename_all = "camelCase")]`.
   2. TS wire types are generated via `ts-rs` into `src/types/generated.ts` (`just types`).
2. Command naming: domain-prefixed, verb-suffixed.
   1. Example: `tasks_update`, `project_lock_get`, `subsystems_list`.
   2. Avoid ambiguous legacy names like `get_task`, `update_task`, etc.
3. Event naming: namespaced with `domain:topic`.
   1. Example: `terminal:output`, `terminal:closed`.
4. No compatibility shims.
   1. Old command names and old JSON field names are not supported.
   2. Broken assumptions fail loudly (repo fail-fast policy).

## 4. Canonical Source Of Truth
1. The command registry is `src-tauri/src/lib.rs` under `.invoke_handler(tauri::generate_handler![...])`.
2. The terminal event contract is defined in Rust (`src-tauri/src/terminal/events.rs`) and in TS (`src/lib/terminal/terminalBridgeContract.ts`).

## 5. Breaking Changes (Summary)
1. Terminal events
   1. Event names:
      1. `terminal:output`
      2. `terminal:closed`
   2. Payload keys:
      1. `sessionId` (was `session_id`)
      2. `exitCode` (was `exit_code`)
2. Terminal commands (frontend `invoke(...)`)
   1. `terminal_start_session`
   2. `terminal_start_task_session`
   3. `terminal_start_human_session`
   4. `terminal_send_input`
   5. `terminal_resize`
   6. `terminal_terminate`
   7. `terminal_set_stream_mode`
   8. `terminal_replay_output`
   9. `terminal_emit_system_message`
   10. `terminal_list_model_form_tree`
3. Tasks commands
   1. List/detail:
      1. `tasks_list`
      2. `tasks_get`
      3. `tasks_list_items`
   2. Mutations:
      1. `tasks_create`
      2. `tasks_update`
      3. `tasks_set_status`
      4. `tasks_delete`
   3. Signals:
      1. `tasks_signal_add`
      2. `tasks_signal_update`
      3. `tasks_signal_delete`
      4. `tasks_signal_summaries_get`
      5. `tasks_comment_reply_add`
      6. `tasks_ask_answer`
4. Subsystems and disciplines commands
   1. Subsystems:
      1. `subsystems_list`
      2. `subsystems_create`
      3. `subsystems_update`
      4. `subsystems_delete`
      5. `subsystems_comment_add`
      6. `subsystems_comment_update`
      7. `subsystems_comment_delete`
   2. Disciplines:
      1. `disciplines_list`
      2. `disciplines_create`
      3. `disciplines_update`
      4. `disciplines_delete`
      5. `stacks_metadata_list`
      6. `disciplines_image_data_get`
      7. `disciplines_cropped_image_get`
5. Project/window commands
   1. Project:
      1. `project_lock_get`
      2. `project_lock_set`
      3. `project_recent_list`
      4. `project_scan`
      5. `project_initialize`
      6. `project_validate_path`
      7. `project_info_get`
      8. `system_home_dir_get`
   2. Window:
      1. `window_splash_close`
      2. `window_open_new`
6. Prompt builder commands
   1. `prompt_builder_preview`
   2. `prompt_builder_config_list`
   3. `prompt_builder_config_get`
   4. `prompt_builder_config_save`
   5. `prompt_builder_config_delete`
7. TaskSignal JSON shape (camelCase)
   1. `signalVerb` (was `signal_verb`)
   2. `sessionId` (was `session_id`)
   3. `parentSignalId` (was `parent_signal_id`)

## 6. Migration Workflow Notes
1. Always regenerate wire types after touching Rust contracts:
   1. `just types`
2. Fix callsites by compiler, not by adding compat layers.
3. If you are unsure what the new command name is:
   1. Look at `src-tauri/src/lib.rs` handler list first.

## 7. “Why 3 Channels?” (SSH vs RPC vs WS) Clarification
1. Layering matters:
   1. SSH is the security envelope (auth + encryption + optional tunnel).
   2. Your app protocol rides inside it.
2. When people say “SSH + RPC + WS”, they often mean:
   1. SSH control plane: connect/auth + start server + port-forward.
   2. RPC data plane: request/response calls (invoke-like).
   3. Streaming data plane: push events (terminal output, logs).
3. You can consolidate the *app layer* to one channel:
   1. Single WebSocket that carries both:
      1. RPC frames (request/response with ids).
      2. Event frames (server push).
   2. This still typically runs over an SSH forward, so the security story remains SSH.
4. You can also consolidate even further (no remote port):
   1. Run `ralphd` over SSH stdio (`ssh host ralphd --stdio`) and speak a framed protocol on stdin/stdout.
   2. This is closest to “one channel” in the literal sense.
5. Recommendation (v1):
   1. SSH tunnel + single app-level WebSocket is the cleanest mental model:
      1. One connection to debug.
      2. Natural for streams and RPC multiplexing.
      3. Keeps `ralphd` bound to localhost on the remote host.

## 8. SSH Security Model (v1)
1. `ralphd` binds to `127.0.0.1` on the remote host.
2. It is only reachable through SSH port-forwarding (or stdio).
3. This is a standard best-practice posture for “developer daemons”:
   1. No public listener by default.
   2. SSH handles identity and encryption.

## 9. Ownership: Who Runs Agents + Who Buffers Streams
1. If `ralphd` owns the agent/PTY process:
   1. Reconnect is possible.
   2. `ralphd` can buffer output canonically and serve replay (`afterSeq`).
   3. The client stays simpler (no duplicate buffering system).
2. If the client backend owns the agent/PTY process:
   1. Disconnect is effectively fatal to the session (hard to “repair”).
   2. `ralphd` becomes thinner, but remote reliability gets worse.
3. Recommended for headless parity:
   1. `ralphd` owns process lifecycle + stream withholding/buffering + replay.

## 10. Complexity Score (Migration)
1. Complexity score (1-10): **8/10** for full parity remote headless `ralphd` (same as `.docs/073_REMOTE_HEADLESS_RALPH_VIA_SSH.md`).
2. Why it’s not “just transport replacement”:
   1. You need process supervision and reconnection semantics.
   2. You need a real server protocol boundary and versioning.
   3. Terminal streaming is stateful and bidirectional (resize/input/output/replay/truncation).
3. What the contract refactor buys:
   1. Much less translation when `ralphd` is introduced.
   2. Cleaner invariants and fewer “mystery mappings” between snake_case and camelCase.

## 11. Required Collections Must Serialize (No Empty Omission)
1. IPC types that are exported to TS (`#[ipc_type]`) must not omit required fields, even when empty.
2. Avoid `skip_serializing_if = "Vec::is_empty"` / `HashMap::is_empty` on required `Vec`/`HashMap` fields.
3. Why:
   1. `ts-rs` generates required TS fields for `Vec<T>` and `HashMap<K,V>`.
   2. If Rust omits empty collections, the frontend can observe `undefined` at runtime and accidentally send `undefined` back in mutation args.
   3. That turns into Tauri invoke payloads missing required keys, causing Rust deserialization failures like `tasks_update: missing field contextFiles`.
4. Guardrails:
   1. Add serialization-shape tests for key IPC structs (for example `Task` and `McpServerConfig`) to ensure required keys are always present.
