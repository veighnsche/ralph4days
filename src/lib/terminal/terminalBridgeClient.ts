import { listen } from '@tauri-apps/api/event'
import { tauriInvoke } from '@/lib/tauri/invoke'
import type {
  PtyClosedEvent,
  PtyOutputEvent,
  TerminalBridgeEmitSystemMessageArgs,
  TerminalBridgeListModelFormTreeResult,
  TerminalBridgeReplayOutputResult,
  TerminalBridgeResizeArgs,
  TerminalBridgeSendInputArgs,
  TerminalBridgeSetStreamModeArgs,
  TerminalBridgeStartSessionArgs,
  TerminalBridgeStartTaskSessionArgs,
  TerminalBridgeTerminateArgs
} from '@/types/generated'
import { TERMINAL_BRIDGE_COMMANDS, TERMINAL_BRIDGE_EVENTS } from './terminalBridgeContract'

// Terminal bridge wire types are generated from Rust contracts in src/types/generated.ts.
const TERMINAL_BRIDGE_DEBUG_PREFIX = '[terminal_bridge]'

function terminalBridgeDebugLog(event: string, payload: unknown) {
  // Intentionally verbose for transport debugging.
  console.debug(`${TERMINAL_BRIDGE_DEBUG_PREFIX} ${event}`, payload)
}

function decodeUtf8Bytes(data: number[]): string {
  return new TextDecoder().decode(new Uint8Array(data))
}

function decodeBase64Payload(data: string): string {
  try {
    const binary = atob(data)
    const bytes = new Uint8Array(binary.length)
    for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i)
    return new TextDecoder().decode(bytes)
  } catch {
    return data
  }
}

function previewText(text: string, maxChars = 220): string {
  if (text.length <= maxChars) return text
  return `${text.slice(0, maxChars)}…`
}

function toReplayAfterSeq(afterSeq: bigint): number {
  if (afterSeq > BigInt(Number.MAX_SAFE_INTEGER)) {
    throw new Error(`[terminal_bridge] afterSeq out of JSON-safe range: ${afterSeq}`)
  }
  return Number(afterSeq)
}

function toReplayAfterSeqRequest(afterSeq: bigint | number | string): number {
  if (typeof afterSeq === 'bigint') {
    if (afterSeq < 0n) {
      throw new Error(`[terminal_bridge] Invalid afterSeq value: ${afterSeq}`)
    }
    return toReplayAfterSeq(afterSeq)
  }

  if (typeof afterSeq === 'string') {
    try {
      const parsed = BigInt(afterSeq)
      if (parsed < 0n) {
        throw new Error()
      }
      return toReplayAfterSeq(parsed)
    } catch {
      throw new Error(`[terminal_bridge] Invalid afterSeq value: ${afterSeq}`)
    }
  }

  if (!Number.isSafeInteger(afterSeq) || afterSeq < 0) {
    throw new Error(`[terminal_bridge] Invalid afterSeq value: ${afterSeq}`)
  }

  return afterSeq
}

export type TerminalBridgeStartHumanSessionArgs = {
  terminalSessionId: string
  kind: string
  taskId?: number
  agent?: string
  model?: string
  effort?: 'low' | 'medium' | 'high'
  permissionLevel?: 'safe' | 'balanced' | 'auto' | 'full_auto'
  postStartPreamble?: string
  initPrompt?: string
  mcpMode?: string
  thinking?: boolean
}

export type TerminalBridgeStartHumanSessionResult = {
  agentSessionId: string
  agentSessionNumber: number
}

export async function terminalBridgeStartSession(params: TerminalBridgeStartSessionArgs) {
  terminalBridgeDebugLog('tx.startSession', params)
  await tauriInvoke(TERMINAL_BRIDGE_COMMANDS.startSession, params)
}

export async function terminalBridgeStartTaskSession(params: TerminalBridgeStartTaskSessionArgs) {
  terminalBridgeDebugLog('tx.startTaskSession', params)
  await tauriInvoke(TERMINAL_BRIDGE_COMMANDS.startTaskSession, params)
}

export async function terminalBridgeListModelFormTree(): Promise<TerminalBridgeListModelFormTreeResult> {
  terminalBridgeDebugLog('tx.listModelFormTree', {})
  return tauriInvoke<TerminalBridgeListModelFormTreeResult>(TERMINAL_BRIDGE_COMMANDS.listModelFormTree)
}

export async function terminalBridgeStartHumanSession(
  params: TerminalBridgeStartHumanSessionArgs
): Promise<TerminalBridgeStartHumanSessionResult> {
  terminalBridgeDebugLog('tx.startHumanSession', params)
  const result = await tauriInvoke<TerminalBridgeStartHumanSessionResult>(TERMINAL_BRIDGE_COMMANDS.startHumanSession, {
    terminalSessionId: params.terminalSessionId,
    kind: params.kind,
    taskId: params.taskId ?? null,
    agent: params.agent ?? null,
    model: params.model ?? null,
    effort: params.effort ?? null,
    permissionLevel: params.permissionLevel ?? null,
    postStartPreamble: params.postStartPreamble ?? null,
    initPrompt: params.initPrompt ?? null,
    mcpMode: params.mcpMode ?? null,
    thinking: params.thinking ?? null
  })
  terminalBridgeDebugLog('tx.startHumanSession.result', result)
  return result
}

export async function terminalBridgeSendInput(sessionId: string, data: string) {
  const params: TerminalBridgeSendInputArgs = {
    sessionId,
    data: Array.from(new TextEncoder().encode(data))
  }
  terminalBridgeDebugLog('tx.sendInput', {
    sessionId,
    byteCount: params.data.length,
    preview: previewText(decodeUtf8Bytes(params.data), 140)
  })
  await tauriInvoke(TERMINAL_BRIDGE_COMMANDS.sendInput, params)
}

export async function terminalBridgeResize(sessionId: string, cols: number, rows: number) {
  const params: TerminalBridgeResizeArgs = { sessionId, cols, rows }
  terminalBridgeDebugLog('tx.resize', params)
  await tauriInvoke(TERMINAL_BRIDGE_COMMANDS.resize, params)
}

export async function terminalBridgeTerminate(sessionId: string) {
  const params: TerminalBridgeTerminateArgs = { sessionId }
  terminalBridgeDebugLog('tx.terminate', params)
  await tauriInvoke(TERMINAL_BRIDGE_COMMANDS.terminate, params)
}

export async function terminalBridgeSetStreamMode(sessionId: string, mode: 'live' | 'buffered') {
  const params: TerminalBridgeSetStreamModeArgs = { sessionId, mode }
  terminalBridgeDebugLog('tx.setStreamMode', params)
  await tauriInvoke(TERMINAL_BRIDGE_COMMANDS.setStreamMode, params)
}

export async function terminalBridgeReplayOutput(
  sessionId: string,
  afterSeq: bigint | number | string,
  limit = 256
): Promise<TerminalBridgeReplayOutputResult> {
  const normalizedAfterSeq = toReplayAfterSeqRequest(afterSeq)
  const params: { sessionId: string; afterSeq: number; limit: number } = {
    sessionId,
    afterSeq: normalizedAfterSeq,
    limit
  }
  terminalBridgeDebugLog('tx.replayOutput', params)
  return tauriInvoke<TerminalBridgeReplayOutputResult>(TERMINAL_BRIDGE_COMMANDS.replayOutput, params)
}

export async function terminalBridgeEmitSystemMessage(sessionId: string, text: string) {
  const params: TerminalBridgeEmitSystemMessageArgs = { sessionId, text }
  terminalBridgeDebugLog('tx.emitSystemMessage', params)
  await tauriInvoke(TERMINAL_BRIDGE_COMMANDS.emitSystemMessage, params)
}

export async function terminalBridgeListenSessionOutput(
  sessionId: string,
  onOutput: (payload: PtyOutputEvent) => void
) {
  terminalBridgeDebugLog('rx.output.subscribe', { sessionId })
  return listen<PtyOutputEvent>(TERMINAL_BRIDGE_EVENTS.output, event => {
    if (event.payload.sessionId !== sessionId) return
    const decoded = decodeBase64Payload(event.payload.data)
    terminalBridgeDebugLog('rx.output', {
      sessionId: event.payload.sessionId,
      seq: String(event.payload.seq),
      byteCount: decoded.length,
      preview: previewText(decoded)
    })
    onOutput(event.payload)
  })
}

export async function terminalBridgeListenSessionClosed(
  sessionId: string,
  onClosed: (payload: PtyClosedEvent) => void
) {
  terminalBridgeDebugLog('rx.closed.subscribe', { sessionId })
  return listen<PtyClosedEvent>(TERMINAL_BRIDGE_EVENTS.closed, event => {
    if (event.payload.sessionId !== sessionId) return
    terminalBridgeDebugLog('rx.closed', event.payload)
    onClosed(event.payload)
  })
}
