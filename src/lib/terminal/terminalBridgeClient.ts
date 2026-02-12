import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type {
  PtyClosedEvent,
  PtyOutputEvent,
  TerminalBridgeEmitSystemMessageArgs,
  TerminalBridgeResizeArgs,
  TerminalBridgeSendInputArgs,
  TerminalBridgeStartSessionArgs,
  TerminalBridgeStartTaskSessionArgs,
  TerminalBridgeTerminateArgs
} from '@/types/generated'
import { TERMINAL_BRIDGE_COMMANDS, TERMINAL_BRIDGE_EVENTS } from './terminalBridgeContract'

// Terminal bridge wire types are generated from Rust contracts in src/types/generated.ts.

export async function terminalBridgeStartSession(params: TerminalBridgeStartSessionArgs) {
  await invoke(TERMINAL_BRIDGE_COMMANDS.startSession, params)
}

export async function terminalBridgeStartTaskSession(params: TerminalBridgeStartTaskSessionArgs) {
  await invoke(TERMINAL_BRIDGE_COMMANDS.startTaskSession, params)
}

export async function terminalBridgeSendInput(sessionId: string, data: string) {
  const params: TerminalBridgeSendInputArgs = {
    sessionId,
    data: Array.from(new TextEncoder().encode(data))
  }
  await invoke(TERMINAL_BRIDGE_COMMANDS.sendInput, params)
}

export async function terminalBridgeResize(sessionId: string, cols: number, rows: number) {
  const params: TerminalBridgeResizeArgs = { sessionId, cols, rows }
  await invoke(TERMINAL_BRIDGE_COMMANDS.resize, params)
}

export async function terminalBridgeTerminate(sessionId: string) {
  const params: TerminalBridgeTerminateArgs = { sessionId }
  await invoke(TERMINAL_BRIDGE_COMMANDS.terminate, params)
}

export async function terminalBridgeEmitSystemMessage(sessionId: string, text: string) {
  const params: TerminalBridgeEmitSystemMessageArgs = { sessionId, text }
  await invoke(TERMINAL_BRIDGE_COMMANDS.emitSystemMessage, params)
}

export async function terminalBridgeListenSessionOutput(
  sessionId: string,
  onOutput: (payload: PtyOutputEvent) => void
) {
  return listen<PtyOutputEvent>(TERMINAL_BRIDGE_EVENTS.output, event => {
    if (event.payload.session_id !== sessionId) return
    onOutput(event.payload)
  })
}

export async function terminalBridgeListenSessionClosed(
  sessionId: string,
  onClosed: (payload: PtyClosedEvent) => void
) {
  return listen<PtyClosedEvent>(TERMINAL_BRIDGE_EVENTS.closed, event => {
    if (event.payload.session_id !== sessionId) return
    onClosed(event.payload)
  })
}
