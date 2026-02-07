import { renderHook, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { TerminalSessionConfig, TerminalSessionHandlers } from './session'
import { useTerminalSession } from './session'

// Mock Tauri API
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
    mockInvoke.mockResolvedValue(undefined)
    mockListen.mockResolvedValue(mockUnlisten)
    mockUnlisten.mockResolvedValue(undefined)
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  it('creates PTY session on mount with correct params', async () => {
    renderHook(() => useTerminalSession(defaultConfig, defaultHandlers))

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('create_pty_session', {
        sessionId: 'test-session',
        mcpMode: 'interactive',
        model: 'haiku',
        thinking: true
      })
    })
  })

  it('terminates PTY session on unmount', async () => {
    const { unmount } = renderHook(() => useTerminalSession(defaultConfig, defaultHandlers))

    unmount()

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('terminate_pty_session', {
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
      expect(mockListen).toHaveBeenCalledWith('ralph://pty_output', expect.any(Function))
    })
  })

  it('sets up PTY closed listener', async () => {
    renderHook(() => useTerminalSession(defaultConfig, defaultHandlers))

    await waitFor(() => {
      expect(mockListen).toHaveBeenCalledWith('ralph://pty_closed', expect.any(Function))
    })
  })

  it('buffers output when not ready', async () => {
    const handlers = {
      ...defaultHandlers,
      onOutput: vi.fn()
    }

    let outputCallback: ((event: { payload: { session_id: string; data: string } }) => void) | undefined

    mockListen.mockImplementation((eventName: string, callback: unknown) => {
      if (eventName === 'ralph://pty_output') {
        outputCallback = callback as typeof outputCallback
      }
      return Promise.resolve(mockUnlisten)
    })

    renderHook(() => useTerminalSession(defaultConfig, handlers))

    await waitFor(() => expect(outputCallback).toBeDefined())

    // Simulate output arriving before terminal is ready ("Hello" as base64)
    outputCallback?.({
      payload: { session_id: 'test-session', data: btoa('Hello') }
    })

    // Should NOT call onOutput yet (buffered)
    expect(handlers.onOutput).not.toHaveBeenCalled()
  })

  it('flushes buffered output when markReady is called', async () => {
    const handlers = {
      ...defaultHandlers,
      onOutput: vi.fn()
    }

    let outputCallback: ((event: { payload: { session_id: string; data: string } }) => void) | undefined

    mockListen.mockImplementation((eventName: string, callback: unknown) => {
      if (eventName === 'ralph://pty_output') {
        outputCallback = callback as typeof outputCallback
      }
      return Promise.resolve(mockUnlisten)
    })

    const { result } = renderHook(() => useTerminalSession(defaultConfig, handlers))

    await waitFor(() => expect(outputCallback).toBeDefined())

    // Buffer output before ready ("Hello" as base64)
    const bufferedData = new Uint8Array([72, 101, 108, 108, 111]) // "Hello"
    outputCallback?.({
      payload: { session_id: 'test-session', data: btoa('Hello') }
    })

    expect(handlers.onOutput).not.toHaveBeenCalled()

    // Mark ready - should flush buffer
    result.current.markReady()

    await waitFor(() => {
      expect(handlers.onOutput).toHaveBeenCalledWith(bufferedData)
    })
  })

  it('sends input to PTY via IPC', async () => {
    const { result } = renderHook(() => useTerminalSession(defaultConfig, defaultHandlers))

    await waitFor(() => expect(result.current.sendInput).toBeDefined())

    result.current.sendInput('ls -la\n')

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('send_terminal_input', {
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
      expect(mockInvoke).toHaveBeenCalledWith('resize_pty', {
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

    let outputCallback: ((event: { payload: { session_id: string; data: string } }) => void) | undefined

    mockListen.mockImplementation((eventName: string, callback: unknown) => {
      if (eventName === 'ralph://pty_output') {
        outputCallback = callback as typeof outputCallback
      }
      return Promise.resolve(mockUnlisten)
    })

    const { result } = renderHook(() => useTerminalSession(defaultConfig, handlers))

    await waitFor(() => expect(outputCallback).toBeDefined())

    result.current.markReady()

    // Output from different session
    outputCallback?.({
      payload: { session_id: 'different-session', data: btoa('He') }
    })

    expect(handlers.onOutput).not.toHaveBeenCalled()
  })

  it('calls onClosed when session ends', async () => {
    const handlers = {
      ...defaultHandlers,
      onClosed: vi.fn()
    }

    let closedCallback: ((event: { payload: { session_id: string; exit_code: number } }) => void) | undefined
    let outputCallback: ((event: { payload: { session_id: string; data: string } }) => void) | undefined

    mockListen.mockImplementation((eventName: string, callback: unknown) => {
      if (eventName === 'ralph://pty_closed') {
        closedCallback = callback as typeof closedCallback
      }
      if (eventName === 'ralph://pty_output') {
        outputCallback = callback as typeof outputCallback
      }
      return Promise.resolve(mockUnlisten)
    })

    renderHook(() => useTerminalSession(defaultConfig, handlers))

    await waitFor(() => expect(closedCallback).toBeDefined())
    await waitFor(() => expect(outputCallback).toBeDefined())

    // Simulate output first to mark session as started
    outputCallback?.({
      payload: { session_id: 'test-session', data: btoa('He') }
    })

    // Now simulate session close
    closedCallback?.({
      payload: { session_id: 'test-session', exit_code: 0 }
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
      expect(mockInvoke).toHaveBeenCalledWith('create_pty_session', {
        sessionId: 'test',
        mcpMode: 'interactive',
        model: null,
        thinking: null
      })
    })
  })
})
