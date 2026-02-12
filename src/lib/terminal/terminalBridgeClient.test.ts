import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
  terminalBridgeEmitSystemMessage,
  terminalBridgeListenSessionClosed,
  terminalBridgeListenSessionOutput,
  terminalBridgeResize,
  terminalBridgeSendInput,
  terminalBridgeStartSession,
  terminalBridgeStartTaskSession,
  terminalBridgeTerminate
} from './terminalBridgeClient'

const mockInvoke = vi.fn()
const mockListen = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args)
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: (...args: unknown[]) => mockListen(...args)
}))

describe('terminalBridgeClient', () => {
  beforeEach(() => {
    mockInvoke.mockResolvedValue(undefined)
    mockListen.mockResolvedValue(vi.fn())
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  it('maps start session to terminal bridge command', async () => {
    await terminalBridgeStartSession({
      sessionId: 's1',
      mcpMode: 'interactive',
      model: 'sonnet',
      thinking: true
    })

    expect(mockInvoke).toHaveBeenCalledWith('terminal_bridge_start_session', {
      sessionId: 's1',
      mcpMode: 'interactive',
      model: 'sonnet',
      thinking: true
    })
  })

  it('maps task session start to terminal bridge command', async () => {
    await terminalBridgeStartTaskSession({
      sessionId: 's1',
      taskId: 7,
      model: undefined,
      thinking: undefined
    })

    expect(mockInvoke).toHaveBeenCalledWith('terminal_bridge_start_task_session', {
      sessionId: 's1',
      taskId: 7,
      model: undefined,
      thinking: undefined
    })
  })

  it('encodes input as bytes', async () => {
    await terminalBridgeSendInput('s1', 'ls\n')

    expect(mockInvoke).toHaveBeenCalledWith('terminal_bridge_send_input', {
      sessionId: 's1',
      data: [108, 115, 10]
    })
  })

  it('maps resize/terminate/system message commands', async () => {
    await terminalBridgeResize('s1', 120, 40)
    await terminalBridgeTerminate('s1')
    await terminalBridgeEmitSystemMessage('s1', '[session started]\r\n')

    expect(mockInvoke).toHaveBeenCalledWith('terminal_bridge_resize', {
      sessionId: 's1',
      cols: 120,
      rows: 40
    })
    expect(mockInvoke).toHaveBeenCalledWith('terminal_bridge_terminate', { sessionId: 's1' })
    expect(mockInvoke).toHaveBeenCalledWith('terminal_bridge_emit_system_message', {
      sessionId: 's1',
      text: '[session started]\r\n'
    })
  })

  it('filters output events by session id', async () => {
    let handler: ((event: { payload: { session_id: string; data: string } }) => void) | undefined
    mockListen.mockImplementation((eventName: string, cb: unknown) => {
      if (eventName === 'terminal_bridge:output') {
        handler = cb as typeof handler
      }
      return Promise.resolve(vi.fn())
    })

    const onOutput = vi.fn()
    await terminalBridgeListenSessionOutput('target', onOutput)

    handler?.({ payload: { session_id: 'other', data: 'x' } })
    handler?.({ payload: { session_id: 'target', data: 'y' } })

    expect(onOutput).toHaveBeenCalledTimes(1)
    expect(onOutput).toHaveBeenCalledWith({ session_id: 'target', data: 'y' })
  })

  it('filters closed events by session id', async () => {
    let handler: ((event: { payload: { session_id: string; exit_code: number } }) => void) | undefined
    mockListen.mockImplementation((eventName: string, cb: unknown) => {
      if (eventName === 'terminal_bridge:closed') {
        handler = cb as typeof handler
      }
      return Promise.resolve(vi.fn())
    })

    const onClosed = vi.fn()
    await terminalBridgeListenSessionClosed('target', onClosed)

    handler?.({ payload: { session_id: 'other', exit_code: 1 } })
    handler?.({ payload: { session_id: 'target', exit_code: 0 } })

    expect(onClosed).toHaveBeenCalledTimes(1)
    expect(onClosed).toHaveBeenCalledWith({ session_id: 'target', exit_code: 0 })
  })
})
