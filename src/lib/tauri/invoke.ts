import { invoke } from '@tauri-apps/api/core'

// Canonical IPC shape: every command that takes parameters expects a single top-level `args` key.
// This keeps the contract uniform and matches Rust signatures like `fn cmd(args: CmdArgs)`.
export function tauriInvoke<TResult>(command: string): Promise<TResult>
export function tauriInvoke<TResult>(command: string, args: Record<string, unknown>): Promise<TResult>
export function tauriInvoke<TResult>(command: string, args?: Record<string, unknown>): Promise<TResult> {
  if (args === undefined) {
    return invoke<TResult>(command)
  }
  return invoke<TResult>(command, { args })
}
