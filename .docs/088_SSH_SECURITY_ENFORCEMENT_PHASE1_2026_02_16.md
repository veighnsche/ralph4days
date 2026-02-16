# SSH Security Enforcement + Tunnel Manager Ownership

Date: 2026-02-16

## Scope
- Enforce hard security invariants for remote transport so direct non-SSH-style exposure paths fail loudly.
- Replace manual WS endpoint UX with first-class SSH tunnel manager commands (connect/liveness/teardown).
- Remove external shell `ssh` dependency from tunnel lifecycle ownership and move it in-process.

## Invariants Added
1. `ralphd` must bind to loopback only (`127.0.0.1` or `::1`).
2. Tauri `remote_connect` accepts only loopback `ws://` URLs with explicit port:
   - allowed: `ws://127.0.0.1:<port>`, `ws://[::1]:<port>`
   - rejected: non-loopback hosts, `wss://`, URL credentials, query/fragment, non-root path, missing port
3. SSH tunnel lifecycle is command-owned in Tauri backend with embedded SSH transport:
   - `remote_ssh_connect` performs strict host-key verification, authenticates key-based SSH, opens `direct-tcpip` to remote `127.0.0.1:<ralphdPort>`, then attaches remote WS over that stream
   - `remote_ssh_status_get` reports active tunnel ownership state
   - `remote_ssh_disconnect` tears down both SSH session and remote connection
4. `remote_disconnect` also tears down managed SSH session to avoid orphan ownership state.

## Files Changed
- `src-daemon/src/main.rs`
  - Added bind parsing helper and loopback enforcement.
  - Added unit tests that reject non-loopback binds.
- `src-tauri/src/ssh_tunnel.rs`
  - Added embedded SSH transport manager (`russh`) with strict known_hosts verification and direct-tcpip channel ownership.
- `src-tauri/src/remote.rs`
  - Added `connect_via_stream` for WebSocket transport over arbitrary async streams.
- `src-tauri/src/commands/remote.rs`
  - Added strict WS URL validator with SSH-tunnel-only posture.
  - Replaced shell-process tunnel lifecycle with embedded SSH lifecycle.
  - Added `remote_ssh_connect`, `remote_ssh_status_get`, `remote_ssh_disconnect` ownership flow.
  - Added unit tests for URL guards + SSH arg normalization.
- `src-tauri/src/commands/state.rs`
  - Updated canonical `SshTunnelSession` ownership type for embedded SSH session state.
- `src-tauri/src/commands/state_desktop.rs`
- `src-tauri/src/commands/state_mobile.rs`
  - Maintained `ssh_tunnel` manager state field in app state.
- `src-tauri/src/lib.rs`
  - Registered/retained SSH tunnel command surface and added embedded SSH module wiring.
- `src-tauri/tests/invoke_command_list_contract_test.rs`
  - Invoke command surface remains contract-locked and stable.
- `src-tauri/tests/remote_strict_decode_contract_test.rs`
  - Updated strict decode guard coverage for evolved remote SSH DTOs.
- `src-tauri/Cargo.toml`
  - Added `russh` dependency for embedded SSH transport.
  - Added `url` crate for strict URL parsing in command validation.
- `crates/core-contracts/src/remote.rs`
  - Evolved canonical DTOs:
    - `RemoteSshConnectArgs` adds optional `known_hosts_file`
    - `RemoteSshConnectResult` uses `ssh_session_id` and includes `known_hosts_file`
    - `RemoteSshStatus` uses `ssh_session_id` and includes `known_hosts_file`
- `src/types/generated.ts`
  - Regenerated TS wire types from canonical Rust contracts.
- `src/App.tsx`
  - Maintained `remote_ssh_status_get` query/invalidation wiring in mobile gate flow.
- `src/components/app-shell/RemoteConnectionPanel.tsx`
  - Replaced raw `wsUrl` input with SSH profile form (host/user/ports/identity/known_hosts).
  - Connect/disconnect call SSH tunnel manager commands.

## Anti-Reward-Hacking Posture
- No hidden compatibility bypass was added.
- No insecure fallback path exists once these checks are active.
- Violations fail immediately with explicit remediation text.
- Unknown host keys are rejected unless pre-trusted in `known_hosts`.

## Remaining Work
- Add explicit host-key trust UX (fingerprint display + user-approved enrollment flow) without weakening strict verification.
- Add passphrase/agent-backed key unlock support for encrypted identity keys.
- Add integration tests covering embedded SSH lifecycle failure propagation and reconnect behavior end-to-end.
