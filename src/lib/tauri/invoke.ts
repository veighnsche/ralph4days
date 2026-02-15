import { invoke } from '@tauri-apps/api/core'
import type { RalphError } from '@/types/generated'

const INTERNAL_ERROR_CODE = 8100

function isRalphErrorLike(value: unknown): value is RalphError {
  if (value == null || typeof value !== 'object') return false
  const obj = value as { code?: unknown; message?: unknown }
  return typeof obj.code === 'number' && Number.isInteger(obj.code) && typeof obj.message === 'string'
}

function parseRalphErrorString(value: string): RalphError | null {
  const match = value.match(/^\[R-(\d{4})\](?:\s(.*))?$/s)
  if (!match) return null
  const code = Number(match[1])
  if (!Number.isInteger(code)) return null
  return { code, message: match[2] ?? '' }
}

function coerceInvokeError(command: string, err: unknown): RalphError {
  if (isRalphErrorLike(err)) return err

  if (typeof err === 'string') {
    return (
      parseRalphErrorString(err) ?? {
        code: INTERNAL_ERROR_CODE,
        message: `ralph invariant violated: uncoded IPC error for '${command}': ${err}`
      }
    )
  }

  if (err instanceof Error) {
    return (
      parseRalphErrorString(err.message) ?? {
        code: INTERNAL_ERROR_CODE,
        message: `ralph invariant violated: uncoded IPC error for '${command}': ${err.message}`
      }
    )
  }

  return {
    code: INTERNAL_ERROR_CODE,
    message: `ralph invariant violated: uncoded IPC error for '${command}': ${String(err)}`
  }
}

export class RalphIpcError extends Error {
  readonly code: number
  readonly ralph: RalphError
  readonly command: string
  readonly raw: unknown

  constructor(command: string, err: RalphError, raw: unknown) {
    super(`[R-${String(err.code).padStart(4, '0')}] ${err.message}`)
    this.name = 'RalphIpcError'
    this.code = err.code
    this.ralph = err
    this.command = command
    this.raw = raw
  }

  override toString() {
    return this.message
  }
}

// Canonical IPC shape: every command that takes parameters expects a single top-level `args` key.
// This keeps the contract uniform and matches Rust signatures like `fn cmd(args: CmdArgs)`.
export function tauriInvoke<TResult>(command: string): Promise<TResult>
export function tauriInvoke<TResult>(command: string, args: Record<string, unknown>): Promise<TResult>
export async function tauriInvoke<TResult>(command: string, args?: Record<string, unknown>): Promise<TResult> {
  try {
    if (args === undefined) {
      return await invoke<TResult>(command)
    }
    return await invoke<TResult>(command, { args })
  } catch (err) {
    throw new RalphIpcError(command, coerceInvokeError(command, err), err)
  }
}
