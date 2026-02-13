# Workspace Isolation Restart Dump

Date: 2026-02-13
Context owner: workspace isolation hardening follow-up

## Why this exists
This is a cold-restart handoff so you can reboot the dev box and resume immediately.

## Current situation
The workspace isolation implementation landed, then code review flagged **4 functional regressions** that must be fixed before considering the patch stable.

## Review findings to resolve

### 1) P1 - New terminal IPC commands invoked but not registered
- Symptom: frontend invokes `terminal_bridge_set_stream_mode` / `terminal_bridge_replay_output`, runtime throws unknown command.
- Primary references:
  - `src/lib/terminal/session.ts`
  - `src-tauri/src/lib.rs`
- Required fix:
  - Ensure both new commands are in `tauri::generate_handler!`.

### 2) P1 - PTY startup output can be dropped
- Symptom: reader thread emits/buffers only when session map lookup succeeds; thread starts before insert; early bytes can vanish.
- Primary reference:
  - `src-tauri/src/terminal/manager.rs`
- Required fix:
  - Insert session state before reader can enqueue output, or otherwise guarantee startup output is buffered even before map visibility.

### 3) P1 - React-query invalidation keys no longer align
- Symptom: query keys are now domain-prefixed (`['app'|'workspace', ...]`), but mutation invalidations still use legacy keys (`['get_tasks']` style), causing stale UI.
- Primary references:
  - `src/hooks/api/useInvoke.ts`
  - `src/hooks/api/useInvokeMutation.ts`
  - places passing `invalidateKeys`
- Required fix:
  - Canonicalize invalidate keys through the same key-builder used by `useInvoke`.

### 4) P2 - seq=0 system messages are dropped by dedupe
- Symptom: session output dedupe ignores `seq <= lastDeliveredSeq`; with `lastDeliveredSeq = 0`, backend system messages emitted as `seq: 0` never render.
- Primary references:
  - `src/lib/terminal/session.ts`
  - `src-tauri/src/commands/terminal_bridge.rs` (system message event creation)
- Required fix:
  - Either allow seq-0 messages through dedupe, or assign system events a normal monotonic seq path.

## Suggested patch order (safe path)
1. Fix Tauri command registration first (unblocks runtime).
2. Fix PTY startup buffering ordering (prevents data loss).
3. Fix query invalidation canonicalization (prevents stale state).
4. Fix seq-0 behavior (restores visible system status lines).
5. Re-run targeted tests + build.

## Validation checklist
- Frontend:
  - `bun test:run src/lib/terminal/session.test.ts`
  - `bun test:run src/lib/terminal/terminalBridgeClient.test.ts src/components/workspace/tabs/terminal/content.test.tsx`
  - `bun test:run src/stores/useWorkspaceStore.test.ts src/components/workspace/kernel.test.ts`
- Rust:
  - `cargo test --manifest-path src-tauri/Cargo.toml terminal_bridge --quiet`
  - `cargo test --manifest-path src-tauri/Cargo.toml terminal::manager --quiet`
- Build:
  - `bun run build`

## Expected done condition
- No unknown terminal bridge command errors in runtime logs.
- No dropped startup terminal banner/prompt output.
- Mutation-driven views refresh correctly in both app/workspace query domains.
- Human-session "connected" (or equivalent system) message renders in terminal.
- Targeted tests + build all pass.
