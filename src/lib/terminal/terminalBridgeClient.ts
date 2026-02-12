import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { TERMINAL_BRIDGE_COMMANDS, TERMINAL_BRIDGE_EVENTS } from './terminalBridgeContract'

type OutputEventPayload = {
  session_id: string
  data: string
}

type ClosedEventPayload = {
  session_id: string
  exit_code: number
}

export async function terminalBridgeStartSession(params: {
  sessionId: string
  mcpMode: string
  model: string | null
  thinking: boolean | null
}) {
  await invoke(TERMINAL_BRIDGE_COMMANDS.startSession, params)
}

export async function terminalBridgeStartTaskSession(params: {
  sessionId: string
  taskId: number
  model: string | null
  thinking: boolean | null
}) {
  await invoke(TERMINAL_BRIDGE_COMMANDS.startTaskSession, params)
}

export async function terminalBridgeSendInput(sessionId: string, data: string) {
  const bytes = Array.from(new TextEncoder().encode(data))
  await invoke(TERMINAL_BRIDGE_COMMANDS.sendInput, { sessionId, data: bytes })
}

export async function terminalBridgeResize(sessionId: string, cols: number, rows: number) {
  await invoke(TERMINAL_BRIDGE_COMMANDS.resize, { sessionId, cols, rows })
}

export async function terminalBridgeTerminate(sessionId: string) {
  await invoke(TERMINAL_BRIDGE_COMMANDS.terminate, { sessionId })
}

export async function terminalBridgeEmitSystemMessage(sessionId: string, text: string) {
  await invoke(TERMINAL_BRIDGE_COMMANDS.emitSystemMessage, { sessionId, text })
}

export async function terminalBridgeListenSessionOutput(
  sessionId: string,
  onOutput: (payload: OutputEventPayload) => void
) {
  return listen<OutputEventPayload>(TERMINAL_BRIDGE_EVENTS.output, event => {
    if (event.payload.session_id !== sessionId) return
    onOutput(event.payload)
  })
}

export async function terminalBridgeListenSessionClosed(
  sessionId: string,
  onClosed: (payload: ClosedEventPayload) => void
) {
  return listen<ClosedEventPayload>(TERMINAL_BRIDGE_EVENTS.closed, event => {
    if (event.payload.session_id !== sessionId) return
    onClosed(event.payload)
  })
}
