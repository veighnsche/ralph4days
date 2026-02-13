import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
  terminalBridgeEmitSystemMessage,
  terminalBridgeListenSessionClosed,
  terminalBridgeListenSessionOutput,
  terminalBridgeListModelFormTree,
  terminalBridgeReplayOutput,
  terminalBridgeResize,
  terminalBridgeSendInput,
  terminalBridgeSetStreamMode,
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
    await terminalBridgeSetStreamMode('s1', 'buffered')
    await terminalBridgeReplayOutput('s1', 42n, 64)
    await terminalBridgeEmitSystemMessage('s1', '[session started]\r\n')

    expect(mockInvoke).toHaveBeenCalledWith('terminal_bridge_resize', {
      sessionId: 's1',
      cols: 120,
      rows: 40
    })
    expect(mockInvoke).toHaveBeenCalledWith('terminal_bridge_terminate', { sessionId: 's1' })
    expect(mockInvoke).toHaveBeenCalledWith('terminal_bridge_set_stream_mode', {
      sessionId: 's1',
      mode: 'buffered'
    })
    expect(mockInvoke).toHaveBeenCalledWith('terminal_bridge_replay_output', {
      sessionId: 's1',
      afterSeq: 42n,
      limit: 64
    })
    expect(mockInvoke).toHaveBeenCalledWith('terminal_bridge_emit_system_message', {
      sessionId: 's1',
      text: '[session started]\r\n'
    })
  })

  it('maps model-form-tree command', async () => {
    mockInvoke.mockResolvedValue({
      providers: [
        { agent: 'claude', models: [] },
        { agent: 'codex', models: [] }
      ]
    })

    const result = await terminalBridgeListModelFormTree()

    expect(mockInvoke).toHaveBeenCalledWith('terminal_bridge_list_model_form_tree')
    expect(result).toEqual({
      providers: [
        { agent: 'claude', models: [] },
        { agent: 'codex', models: [] }
      ]
    })
  })

  it('filters output events by session id', async () => {
    let handler: ((event: { payload: { session_id: string; seq: bigint; data: string } }) => void) | undefined
    mockListen.mockImplementation((eventName: string, cb: unknown) => {
      if (eventName === 'terminal_bridge:output') {
        handler = cb as typeof handler
      }
      return Promise.resolve(vi.fn())
    })

    const onOutput = vi.fn()
    await terminalBridgeListenSessionOutput('target', onOutput)

    handler?.({ payload: { session_id: 'other', seq: 1n, data: 'x' } })
    handler?.({ payload: { session_id: 'target', seq: 2n, data: 'y' } })

    expect(onOutput).toHaveBeenCalledTimes(1)
    expect(onOutput).toHaveBeenCalledWith({ session_id: 'target', seq: 2n, data: 'y' })
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
