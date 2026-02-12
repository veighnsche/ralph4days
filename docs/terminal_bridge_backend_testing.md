# Terminal Bridge Backend Testing

This test project scopes backend coverage for the `terminal_bridge` subsystem in `src-tauri`.

## Current scope

- Contract stability:
  - `terminal_bridge:output`
  - `terminal_bridge:closed`
- Payload shape:
  - `PtyOutputEvent`
  - `PtyClosedEvent`
- Command helpers:
  - project lock path behavior
  - system message payload encoding
  - ANSI/newline preservation in emitted system messages
- PTY manager negative paths:
  - missing session input/resize error
  - terminate missing session is a no-op
- Runtime-backed emission check:
  - `terminal_bridge_emit_system_message` delivers `terminal_bridge:output` payload via app event bus
- Command-path lifecycle checks (deterministic subset):
  - `send_input` missing-session error path
  - `resize` missing-session error path
  - `terminate` missing-session no-op path

## Run only this suite

```bash
just test-terminal-bridge-backend
```

## Next phase (recommended)

- Add lifecycle integration coverage for:
  - start session
  - input
  - resize
  - terminate
- Validate event emission ordering for startup preamble delivery.
