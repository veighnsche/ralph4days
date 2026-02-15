import { invoke } from '@tauri-apps/api/core'
import type { RalphError, RalphErrorContextItem, RalphErrorLocation } from '@/types/generated'

const INTERNAL_ERROR_CODE = 8100

function isRalphErrorLocationLike(value: unknown): value is RalphErrorLocation {
  if (value == null || typeof value !== 'object') return false
  const obj = value as { file?: unknown; line?: unknown; column?: unknown }
  return (
    typeof obj.file === 'string' &&
    typeof obj.line === 'number' &&
    Number.isInteger(obj.line) &&
    typeof obj.column === 'number' &&
    Number.isInteger(obj.column)
  )
}

function isRalphErrorContextLike(value: unknown): value is RalphErrorContextItem[] {
  if (!Array.isArray(value)) return false
  for (const item of value) {
    if (item == null || typeof item !== 'object') return false
    const obj = item as { key?: unknown; value?: unknown }
    if (typeof obj.key !== 'string') return false
  }
  return true
}

function isRalphErrorLike(value: unknown): value is RalphError {
  if (value == null || typeof value !== 'object') return false
  const obj = value as { code?: unknown; message?: unknown; location?: unknown; context?: unknown; hint?: unknown }
  if (typeof obj.code !== 'number' || !Number.isInteger(obj.code)) return false
  if (typeof obj.message !== 'string') return false
  if (!isRalphErrorLocationLike(obj.location)) return false
  if (!isRalphErrorContextLike(obj.context)) return false
  if (obj.hint !== undefined && typeof obj.hint !== 'string') return false
  return true
}

function frontendLocation(): RalphErrorLocation {
  return { file: '<frontend>', line: 0, column: 0 }
}

function attachInvokeContext(command: string, args: Record<string, unknown> | undefined, err: RalphError): RalphError {
  const extra: RalphErrorContextItem[] = [{ key: 'command', value: command }]
  if (args !== undefined) extra.push({ key: 'args', value: args })
  return { ...err, context: [...err.context, ...extra] }
}

function internalInvokeError(command: string, args: Record<string, unknown> | undefined, raw: unknown): RalphError {
  return {
    code: INTERNAL_ERROR_CODE,
    message: `ralph invariant violated: uncoded IPC error for '${command}': ${String(raw)}`,
    location: frontendLocation(),
    context: [
      { key: 'command', value: command },
      ...(args === undefined ? [] : [{ key: 'args', value: args }]),
      { key: 'raw', value: String(raw) }
    ]
  }
}

function coerceInvokeError(command: string, args: Record<string, unknown> | undefined, err: unknown): RalphError {
  if (isRalphErrorLike(err)) return attachInvokeContext(command, args, err)

  if (typeof err === 'string') {
    return internalInvokeError(command, args, err)
  }

  if (err instanceof Error) {
    const base = internalInvokeError(command, args, err.message)
    return {
      ...base,
      context: [...base.context, ...(err.stack == null ? [] : [{ key: 'stack', value: err.stack }])]
    }
  }

  return internalInvokeError(command, args, err)
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
    throw new RalphIpcError(command, coerceInvokeError(command, args, err), err)
  }
}
