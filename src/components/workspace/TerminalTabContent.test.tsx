import { act, render, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import { TerminalTabContent } from './TerminalTabContent'

const { invokeMock, terminalBridgeEmitSystemMessageMock, useTerminalSessionMock, useTerminalSessionState } = vi.hoisted(
  () => ({
    invokeMock: vi.fn(),
    terminalBridgeEmitSystemMessageMock: vi.fn().mockResolvedValue(undefined),
    useTerminalSessionMock: vi.fn(),
    useTerminalSessionState: { lastHandlers: null as { onStarted?: () => void } | null }
  })
)

vi.mock('@/lib/terminal', () => ({
  terminalBridgeEmitSystemMessage: terminalBridgeEmitSystemMessageMock,
  Terminal: ({ onReady }: { onReady?: (terminal: unknown) => void }) => {
    onReady?.({
      cols: 80,
      rows: 24,
      write: vi.fn(),
      writeln: vi.fn(),
      onData: vi.fn(),
      attachCustomKeyEventHandler: vi.fn()
    })
    return <div data-testid="terminal">Terminal</div>
  },
  useTerminalSession: (config: unknown, handlers: { onStarted?: () => void }) => {
    useTerminalSessionMock(config)
    useTerminalSessionState.lastHandlers = handlers
    return {
      markReady: vi.fn(),
      sendInput: vi.fn(),
      resize: vi.fn()
    }
  }
}))

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args)
}))

vi.mock('@/hooks/workspace/useTabMeta', () => ({
  useTabMeta: vi.fn()
}))

vi.mock('lucide-react', async importOriginal => ({
  ...(await importOriginal<typeof import('lucide-react')>()),
  TerminalSquare: () => <svg data-testid="terminal-icon" />
}))

describe('TerminalTabContent', () => {
  const mockTab: WorkspaceTab = {
    id: 'test-terminal-1',
    type: 'terminal',
    component: TerminalTabContent,
    title: 'Terminal 1',
    closeable: true,
    data: {
      model: 'haiku',
      thinking: true
    }
  }

  beforeEach(() => {
    vi.clearAllMocks()
    useTerminalSessionState.lastHandlers = null
    // Keep async session-persist state transitions out of tests that do not await them.
    invokeMock.mockImplementation(() => new Promise(() => {}))
  })

  it('renders Terminal component', () => {
    const { getByTestId } = render(<TerminalTabContent tab={mockTab} />)
    expect(getByTestId('terminal')).toBeTruthy()
  })

  it('sets tab metadata', async () => {
    const { useTabMeta } = await import('@/hooks/workspace/useTabMeta')
    render(<TerminalTabContent tab={mockTab} />)

    await waitFor(() => {
      expect(useTabMeta).toHaveBeenCalledWith('test-terminal-1', 'Terminal 1', expect.any(Function))
    })
  })

  it('handles tab with minimal config', () => {
    const minimalTab: WorkspaceTab = {
      id: 'test-terminal-2',
      type: 'terminal',
      component: TerminalTabContent,
      title: 'Terminal 2',
      closeable: true
    }

    const { getByTestId } = render(<TerminalTabContent tab={minimalTab} />)
    expect(getByTestId('terminal')).toBeTruthy()
  })

  it('renders Terminal inside flex layout wrapper', () => {
    const { container } = render(<TerminalTabContent tab={mockTab} />)
    const wrapper = container.firstElementChild
    expect(wrapper?.classList.contains('flex')).toBe(true)
    expect(wrapper?.querySelector('[data-testid="terminal"]')).toBeTruthy()
  })

  it('emits startup message only after session persisted and bridge started', async () => {
    vi.useFakeTimers()
    invokeMock.mockResolvedValueOnce(undefined)
    render(<TerminalTabContent tab={mockTab} />)

    await act(async () => {
      await Promise.resolve()
    })
    expect(useTerminalSessionMock).toHaveBeenCalled()

    const firstConfig = useTerminalSessionMock.mock.calls[0][0] as { enabled?: boolean }
    expect(firstConfig.enabled).toBe(false)

    await act(async () => {
      await Promise.resolve()
    })
    const hasEnabledTrue = useTerminalSessionMock.mock.calls.some(
      call => ((call[0] as { enabled?: boolean }).enabled ?? false) === true
    )
    expect(hasEnabledTrue).toBe(true)

    expect(terminalBridgeEmitSystemMessageMock).not.toHaveBeenCalled()

    act(() => {
      useTerminalSessionState.lastHandlers?.onStarted?.()
    })

    act(() => {
      vi.advanceTimersByTime(1500)
    })

    await act(async () => {
      await Promise.resolve()
    })
    expect(terminalBridgeEmitSystemMessageMock).toHaveBeenCalledTimes(1)

    vi.useRealTimers()
  })
})
