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

    expect(mockInvoke).toHaveBeenCalledWith('terminal_start_session', {
      args: {
        sessionId: 's1',
        mcpMode: 'interactive',
        model: 'sonnet',
        thinking: true
      }
    })
  })

  it('maps task session start to terminal bridge command', async () => {
    await terminalBridgeStartTaskSession({
      sessionId: 's1',
      taskId: 7,
      model: undefined,
      thinking: undefined
    })

    expect(mockInvoke).toHaveBeenCalledWith('terminal_start_task_session', {
      args: {
        sessionId: 's1',
        taskId: 7,
        model: undefined,
        thinking: undefined
      }
    })
  })

  it('encodes input as bytes', async () => {
    await terminalBridgeSendInput('s1', 'ls\n')

    expect(mockInvoke).toHaveBeenCalledWith('terminal_send_input', {
      args: {
        sessionId: 's1',
        data: [108, 115, 10]
      }
    })
  })

  it('maps resize/terminate/system message commands', async () => {
    await terminalBridgeResize('s1', 120, 40)
    await terminalBridgeTerminate('s1')
    await terminalBridgeSetStreamMode('s1', 'buffered')
    await terminalBridgeReplayOutput('s1', 42n, 64)
    await terminalBridgeEmitSystemMessage('s1', '[session started]\r\n')

    expect(mockInvoke).toHaveBeenCalledWith('terminal_resize', {
      args: {
        sessionId: 's1',
        cols: 120,
        rows: 40
      }
    })
    expect(mockInvoke).toHaveBeenCalledWith('terminal_terminate', { args: { sessionId: 's1' } })
    expect(mockInvoke).toHaveBeenCalledWith('terminal_set_stream_mode', {
      args: {
        sessionId: 's1',
        mode: 'buffered'
      }
    })
    expect(mockInvoke).toHaveBeenCalledWith('terminal_replay_output', {
      args: {
        sessionId: 's1',
        afterSeq: 42,
        limit: 64
      }
    })
    expect(mockInvoke).toHaveBeenCalledWith('terminal_emit_system_message', {
      args: {
        sessionId: 's1',
        text: '[session started]\r\n'
      }
    })
  })

  it('accepts numeric afterSeq payloads in replay requests', async () => {
    await terminalBridgeReplayOutput('s1', 123, 16)

    expect(mockInvoke).toHaveBeenCalledWith('terminal_replay_output', {
      args: {
        sessionId: 's1',
        afterSeq: 123,
        limit: 16
      }
    })
  })

  it('accepts string afterSeq payloads in replay requests', async () => {
    await terminalBridgeReplayOutput('s1', '123')

    expect(mockInvoke).toHaveBeenCalledWith('terminal_replay_output', {
      args: {
        sessionId: 's1',
        afterSeq: 123,
        limit: 256
      }
    })
  })

  it('rejects malformed string afterSeq payloads in replay requests', async () => {
    await expect(terminalBridgeReplayOutput('s1', 'bad')).rejects.toThrow(
      '[terminal_bridge] Invalid afterSeq value: bad'
    )

    expect(mockInvoke).not.toHaveBeenCalled()
  })

  it('maps model-form-tree command', async () => {
    mockInvoke.mockResolvedValue({
      providers: [
        { agent: 'claude', models: [] },
        { agent: 'codex', models: [] }
      ]
    })

    const result = await terminalBridgeListModelFormTree()

    expect(mockInvoke).toHaveBeenCalledWith('terminal_list_model_form_tree')
    expect(result).toEqual({
      providers: [
        { agent: 'claude', models: [] },
        { agent: 'codex', models: [] }
      ]
    })
  })

  it('filters output events by session id', async () => {
    let handler: ((event: { payload: { sessionId: string; seq: bigint; data: string } }) => void) | undefined
    mockListen.mockImplementation((eventName: string, cb: unknown) => {
      if (eventName === 'terminal:output') {
        handler = cb as typeof handler
      }
      return Promise.resolve(vi.fn())
    })

    const onOutput = vi.fn()
    await terminalBridgeListenSessionOutput('target', onOutput)

    handler?.({ payload: { sessionId: 'other', seq: 1n, data: 'x' } })
    handler?.({ payload: { sessionId: 'target', seq: 2n, data: 'y' } })

    expect(onOutput).toHaveBeenCalledTimes(1)
    expect(onOutput).toHaveBeenCalledWith({ sessionId: 'target', seq: 2n, data: 'y' })
  })

  it('filters closed events by session id', async () => {
    let handler: ((event: { payload: { sessionId: string; exitCode: number } }) => void) | undefined
    mockListen.mockImplementation((eventName: string, cb: unknown) => {
      if (eventName === 'terminal:closed') {
        handler = cb as typeof handler
      }
      return Promise.resolve(vi.fn())
    })

    const onClosed = vi.fn()
    await terminalBridgeListenSessionClosed('target', onClosed)

    handler?.({ payload: { sessionId: 'other', exitCode: 1 } })
    handler?.({ payload: { sessionId: 'target', exitCode: 0 } })

    expect(onClosed).toHaveBeenCalledTimes(1)
    expect(onClosed).toHaveBeenCalledWith({ sessionId: 'target', exitCode: 0 })
  })
})
