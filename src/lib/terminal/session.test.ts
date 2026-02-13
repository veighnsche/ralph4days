import { act, renderHook, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { TerminalSessionConfig, TerminalSessionHandlers } from './session'
import { useTerminalSession } from './session'

const mockInvoke = vi.fn()
const mockListen = vi.fn()
const mockUnlisten = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args)
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: (...args: unknown[]) => mockListen(...args)
}))

describe('useTerminalSession', () => {
  const defaultConfig: TerminalSessionConfig = {
    sessionId: 'test-session',
    mcpMode: 'interactive',
    model: 'haiku',
    thinking: true
  }

  const defaultHandlers: TerminalSessionHandlers = {
    onOutput: vi.fn(),
    onClosed: vi.fn(),
    onError: vi.fn()
  }

  beforeEach(() => {
    mockInvoke.mockImplementation((command: string) => {
      if (command === 'terminal_bridge_replay_output') {
        return Promise.resolve({
          chunks: [],
          hasMore: false,
          truncated: false,
          truncatedUntilSeq: null
        })
      }
      return Promise.resolve(undefined)
    })
    mockListen.mockResolvedValue(mockUnlisten)
    mockUnlisten.mockResolvedValue(undefined)
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  it('creates PTY session on mount with correct params', async () => {
    renderHook(() => useTerminalSession(defaultConfig, defaultHandlers))

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('terminal_bridge_start_session', {
        sessionId: 'test-session',
        agent: 'claude',
        mcpMode: 'interactive',
        model: 'haiku',
        thinking: true
      })
    })
  })

  it('does not start terminal bridge session when disabled', async () => {
    const config: TerminalSessionConfig = {
      ...defaultConfig,
      enabled: false
    }

    renderHook(() => useTerminalSession(config, defaultHandlers))

    await waitFor(() => {
      expect(mockInvoke).not.toHaveBeenCalledWith('terminal_bridge_start_session', expect.anything())
      expect(mockInvoke).not.toHaveBeenCalledWith('terminal_bridge_start_task_session', expect.anything())
    })
  })

  it('calls onStarted once when bridge session starts', async () => {
    const handlers = {
      ...defaultHandlers,
      onStarted: vi.fn()
    }

    renderHook(() => useTerminalSession(defaultConfig, handlers))

    await waitFor(() => {
      expect(handlers.onStarted).toHaveBeenCalledTimes(1)
    })
  })

  it('terminates PTY session on unmount', async () => {
    const { unmount } = renderHook(() => useTerminalSession(defaultConfig, defaultHandlers))

    unmount()

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('terminal_bridge_terminate', {
        sessionId: 'test-session'
      })
    })
  })

  it('calls onError when session creation fails', async () => {
    const error = 'Failed to spawn claude'
    mockInvoke.mockRejectedValueOnce(error)

    const handlers = {
      ...defaultHandlers,
      onError: vi.fn()
    }

    renderHook(() => useTerminalSession(defaultConfig, handlers))

    await waitFor(() => {
      expect(handlers.onError).toHaveBeenCalledWith(error)
    })
  })

  it('sets up PTY output listener', async () => {
    renderHook(() => useTerminalSession(defaultConfig, defaultHandlers))

    await waitFor(() => {
      expect(mockListen).toHaveBeenCalledWith('terminal_bridge:output', expect.any(Function))
    })
  })

  it('sets up PTY closed listener', async () => {
    renderHook(() => useTerminalSession(defaultConfig, defaultHandlers))

    await waitFor(() => {
      expect(mockListen).toHaveBeenCalledWith('terminal_bridge:closed', expect.any(Function))
    })
  })

  it('buffers output when not ready', async () => {
    const handlers = {
      ...defaultHandlers,
      onOutput: vi.fn()
    }

    let outputCallback: ((event: { payload: { session_id: string; seq: bigint; data: string } }) => void) | undefined

    mockListen.mockImplementation((eventName: string, callback: unknown) => {
      if (eventName === 'terminal_bridge:output') {
        outputCallback = callback as typeof outputCallback
      }
      return Promise.resolve(mockUnlisten)
    })

    renderHook(() => useTerminalSession(defaultConfig, handlers))

    await waitFor(() => expect(outputCallback).toBeDefined())

    act(() => {
      outputCallback?.({
        payload: { session_id: 'test-session', seq: 1n, data: btoa('Hello') }
      })
    })

    expect(handlers.onOutput).not.toHaveBeenCalled()
  })

  it('accepts numeric seq values from event payloads', async () => {
    const handlers = {
      ...defaultHandlers,
      onOutput: vi.fn()
    }

    let outputCallback: ((event: { payload: { session_id: string; seq: unknown; data: string } }) => void) | undefined

    mockListen.mockImplementation((eventName: string, callback: unknown) => {
      if (eventName === 'terminal_bridge:output') {
        outputCallback = callback as typeof outputCallback
      }
      return Promise.resolve(mockUnlisten)
    })

    const { result } = renderHook(() => useTerminalSession(defaultConfig, handlers))

    await waitFor(() => expect(outputCallback).toBeDefined())

    result.current.markReady()
    act(() => {
      outputCallback?.({
        payload: {
          session_id: 'test-session',
          seq: 7,
          data: btoa('hello')
        }
      })
    })

    await waitFor(() => {
      expect(handlers.onOutput).toHaveBeenCalledWith(new TextEncoder().encode('hello'))
    })
  })

  it('reports invalid seq payload values instead of dropping output', async () => {
    const handlers = {
      ...defaultHandlers,
      onOutput: vi.fn(),
      onError: vi.fn()
    }

    let outputCallback: ((event: { payload: { session_id: string; seq: unknown; data: string } }) => void) | undefined

    mockListen.mockImplementation((eventName: string, callback: unknown) => {
      if (eventName === 'terminal_bridge:output') {
        outputCallback = callback as typeof outputCallback
      }
      return Promise.resolve(mockUnlisten)
    })

    const { result } = renderHook(() => useTerminalSession(defaultConfig, handlers))

    await waitFor(() => expect(outputCallback).toBeDefined())

    result.current.markReady()
    act(() => {
      outputCallback?.({
        payload: {
          session_id: 'test-session',
          seq: 'not-a-number',
          data: btoa('oops')
        }
      })
    })

    await waitFor(() => {
      expect(handlers.onError).toHaveBeenCalledTimes(1)
      expect(handlers.onOutput).not.toHaveBeenCalled()
    })
  })

  it('flushes buffered output when markReady is called', async () => {
    const handlers = {
      ...defaultHandlers,
      onOutput: vi.fn()
    }

    let outputCallback: ((event: { payload: { session_id: string; seq: bigint; data: string } }) => void) | undefined

    mockListen.mockImplementation((eventName: string, callback: unknown) => {
      if (eventName === 'terminal_bridge:output') {
        outputCallback = callback as typeof outputCallback
      }
      return Promise.resolve(mockUnlisten)
    })

    const { result } = renderHook(() => useTerminalSession(defaultConfig, handlers))

    await waitFor(() => expect(outputCallback).toBeDefined())

    const bufferedData = new Uint8Array([72, 101, 108, 108, 111]) // "Hello"
    act(() => {
      outputCallback?.({
        payload: { session_id: 'test-session', seq: 1n, data: btoa('Hello') }
      })
    })

    expect(handlers.onOutput).not.toHaveBeenCalled()

    result.current.markReady()

    await waitFor(() => expect(handlers.onOutput).toHaveBeenCalledWith(bufferedData))
  })

  it('sends input to PTY via IPC', async () => {
    const { result } = renderHook(() => useTerminalSession(defaultConfig, defaultHandlers))

    await waitFor(() => expect(result.current.sendInput).toBeDefined())

    result.current.sendInput('ls -la\n')

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('terminal_bridge_send_input', {
        sessionId: 'test-session',
        data: [108, 115, 32, 45, 108, 97, 10] // "ls -la\n" as bytes
      })
    })
  })

  it('resizes PTY via IPC', async () => {
    const { result } = renderHook(() => useTerminalSession(defaultConfig, defaultHandlers))

    await waitFor(() => expect(result.current.resize).toBeDefined())

    result.current.resize(120, 40)

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('terminal_bridge_resize', {
        sessionId: 'test-session',
        cols: 120,
        rows: 40
      })
    })
  })

  it('ignores output from other sessions', async () => {
    const handlers = {
      ...defaultHandlers,
      onOutput: vi.fn()
    }

    let outputCallback: ((event: { payload: { session_id: string; seq: bigint; data: string } }) => void) | undefined

    mockListen.mockImplementation((eventName: string, callback: unknown) => {
      if (eventName === 'terminal_bridge:output') {
        outputCallback = callback as typeof outputCallback
      }
      return Promise.resolve(mockUnlisten)
    })

    const { result } = renderHook(() => useTerminalSession(defaultConfig, handlers))

    await waitFor(() => expect(outputCallback).toBeDefined())

    result.current.markReady()

    act(() => {
      outputCallback?.({
        payload: { session_id: 'different-session', seq: 1n, data: btoa('He') }
      })
    })

    expect(handlers.onOutput).not.toHaveBeenCalled()
  })

  it('calls onClosed when session ends', async () => {
    const handlers = {
      ...defaultHandlers,
      onClosed: vi.fn()
    }

    let closedCallback: ((event: { payload: { session_id: string; exit_code: number } }) => void) | undefined
    let outputCallback: ((event: { payload: { session_id: string; seq: bigint; data: string } }) => void) | undefined

    mockListen.mockImplementation((eventName: string, callback: unknown) => {
      if (eventName === 'terminal_bridge:closed') {
        closedCallback = callback as typeof closedCallback
      }
      if (eventName === 'terminal_bridge:output') {
        outputCallback = callback as typeof outputCallback
      }
      return Promise.resolve(mockUnlisten)
    })

    renderHook(() => useTerminalSession(defaultConfig, handlers))

    await waitFor(() => expect(closedCallback).toBeDefined())
    await waitFor(() => expect(outputCallback).toBeDefined())

    act(() => {
      outputCallback?.({
        payload: { session_id: 'test-session', seq: 1n, data: btoa('He') }
      })
    })

    act(() => {
      closedCallback?.({
        payload: { session_id: 'test-session', exit_code: 0 }
      })
    })

    await waitFor(() => {
      expect(handlers.onClosed).toHaveBeenCalledWith(0)
    })
  })

  it('uses default mcpMode when not provided', async () => {
    const config = {
      sessionId: 'test',
      model: null,
      thinking: null
    }

    renderHook(() => useTerminalSession(config, defaultHandlers))

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('terminal_bridge_start_session', {
        sessionId: 'test',
        agent: 'claude',
        mcpMode: 'interactive',
        model: undefined,
        thinking: undefined
      })
    })
  })

  it('switches to buffered mode while inactive', async () => {
    const config: TerminalSessionConfig = {
      ...defaultConfig,
      isActive: false
    }

    renderHook(() => useTerminalSession(config, defaultHandlers))

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('terminal_bridge_set_stream_mode', {
        sessionId: 'test-session',
        mode: 'buffered'
      })
    })
  })

  it('replays output and returns to live mode on reactivation', async () => {
    mockInvoke.mockImplementation((command: string) => {
      if (command === 'terminal_bridge_replay_output') {
        return Promise.resolve({
          chunks: [{ seq: 2n, data: btoa('Hi') }],
          hasMore: false,
          truncated: false,
          truncatedUntilSeq: null
        })
      }
      return Promise.resolve(undefined)
    })

    const { result, rerender } = renderHook(
      ({ isActive }: { isActive: boolean }) =>
        useTerminalSession(
          {
            ...defaultConfig,
            isActive
          },
          defaultHandlers
        ),
      {
        initialProps: { isActive: false }
      }
    )

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('terminal_bridge_set_stream_mode', {
        sessionId: 'test-session',
        mode: 'buffered'
      })
    })

    result.current.markReady()
    rerender({ isActive: true })

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('terminal_bridge_set_stream_mode', {
        sessionId: 'test-session',
        mode: 'live'
      })
    })
  })

  it('normalizes numeric seq values in replayed chunks', async () => {
    mockInvoke.mockImplementation((command: string) => {
      if (command === 'terminal_bridge_replay_output') {
        return Promise.resolve({
          chunks: [{ seq: 5 as unknown as bigint, data: btoa('Hi') }],
          hasMore: false,
          truncated: false,
          truncatedUntilSeq: null
        })
      }
      return Promise.resolve(undefined)
    })

    const handlers = {
      ...defaultHandlers,
      onOutput: vi.fn()
    }

    const { result } = renderHook(() => useTerminalSession(defaultConfig, handlers))

    result.current.markReady()
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('terminal_bridge_set_stream_mode', {
        sessionId: 'test-session',
        mode: 'buffered'
      })
    })

    await waitFor(() => {
      expect(handlers.onOutput).toHaveBeenCalledWith(new TextEncoder().encode('Hi'))
    })
  })

  it('restores live stream mode when replay fails', async () => {
    mockInvoke.mockImplementation((command: string) => {
      if (command === 'terminal_bridge_replay_output') {
        return Promise.reject(new Error('[terminal_bridge] failed to replay output'))
      }
      return Promise.resolve(undefined)
    })

    const handlers = {
      ...defaultHandlers,
      onError: vi.fn()
    }

    const { rerender } = renderHook(
      ({ isActive }: { isActive: boolean }) =>
        useTerminalSession(
          {
            ...defaultConfig,
            isActive
          },
          handlers
        ),
      {
        initialProps: { isActive: false }
      }
    )

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('terminal_bridge_set_stream_mode', {
        sessionId: 'test-session',
        mode: 'buffered'
      })
    })

    rerender({ isActive: true })

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('terminal_bridge_set_stream_mode', {
        sessionId: 'test-session',
        mode: 'live'
      })
    })

    await waitFor(() => {
      expect(handlers.onError).toHaveBeenCalledWith(
        expect.stringContaining('[terminal_bridge] failed to replay output')
      )
    })
  })

  it('delivers seq-0 system output despite initial dedupe cursor', async () => {
    const handlers = {
      ...defaultHandlers,
      onOutput: vi.fn()
    }

    let outputCallback: ((event: { payload: { session_id: string; seq: bigint; data: string } }) => void) | undefined

    mockListen.mockImplementation((eventName: string, callback: unknown) => {
      if (eventName === 'terminal_bridge:output') {
        outputCallback = callback as typeof outputCallback
      }
      return Promise.resolve(mockUnlisten)
    })

    const { result } = renderHook(() => useTerminalSession(defaultConfig, handlers))

    await waitFor(() => expect(outputCallback).toBeDefined())

    result.current.markReady()
    act(() => {
      outputCallback?.({
        payload: { session_id: 'test-session', seq: 0n, data: btoa('connected') }
      })
    })

    await waitFor(() => {
      expect(handlers.onOutput).toHaveBeenCalledWith(new Uint8Array([99, 111, 110, 110, 101, 99, 116, 101, 100]))
    })
  })
})
