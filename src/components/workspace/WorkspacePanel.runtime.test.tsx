import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useAgentSessionLaunchPreferences } from '@/hooks/preferences'
import type { TerminalSessionConfig, TerminalSessionHandlers } from '@/lib/terminal'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'
import { WorkspacePanel } from './WorkspacePanel'

type AgentSessionLaunchConfig = {
  agent: 'claude' | 'codex'
  model: string
  effort: 'low' | 'medium' | 'high'
  thinking: boolean
  permissionLevel: 'safe' | 'balanced' | 'auto' | 'full_auto'
}

const { resolveLaunchConfigMock, terminalSessionMocks, xtermInstances, fitAddonInvocations } = vi.hoisted(() => ({
  resolveLaunchConfigMock: vi.fn(async (config: AgentSessionLaunchConfig): Promise<AgentSessionLaunchConfig> => config),
  terminalSessionMocks: [] as Array<{
    config: TerminalSessionConfig
    handlers: TerminalSessionHandlers
    session: {
      markReady: ReturnType<typeof vi.fn>
      sendInput: ReturnType<typeof vi.fn>
      resize: ReturnType<typeof vi.fn>
    }
  }>,
  xtermInstances: [] as Array<{
    open: ReturnType<typeof vi.fn>
    loadAddon: ReturnType<typeof vi.fn>
    onResize: ReturnType<typeof vi.fn>
    attachCustomKeyEventHandler: ReturnType<typeof vi.fn>
    dispose: ReturnType<typeof vi.fn>
    emitResize: (cols: number, rows: number) => void
  }>,
  fitAddonInvocations: [] as Array<{
    fit: ReturnType<typeof vi.fn>
  }>
}))

vi.mock('@/components/agent-session-launch/resolveLaunchConfig', () => ({
  resolveLaunchConfigAgainstCatalog: resolveLaunchConfigMock
}))

vi.mock('@/lib/terminal', async () => {
  const actual = await vi.importActual<typeof import('@/lib/terminal')>('@/lib/terminal')
  return {
    ...actual,
    useTerminalSession: (config: TerminalSessionConfig, handlers: TerminalSessionHandlers) => {
      const session = {
        markReady: vi.fn(),
        sendInput: vi.fn(),
        resize: vi.fn()
      }
      terminalSessionMocks.push({ config, handlers, session })
      return session
    }
  }
})

vi.mock('@xterm/addon-fit', () => ({
  FitAddon: class MockFitAddon {
    fit = vi.fn()

    constructor() {
      fitAddonInvocations.push({ fit: this.fit })
    }
  }
}))

vi.mock('@xterm/addon-web-links', () => ({
  WebLinksAddon: class MockWebLinksAddon {}
}))

vi.mock('@xterm/xterm', () => ({
  Terminal: vi.fn(function MockTerminal() {
    const resizeListeners: Array<(size: { cols: number; rows: number }) => void> = []
    const renderListeners: Array<() => void> = []

    const instance = {
      cols: 80,
      rows: 24,
      buffer: {
        active: {
          length: 0,
          getLine: vi.fn(() => null)
        }
      },
      open: vi.fn((container: HTMLElement) => {
        const node = document.createElement('div')
        node.setAttribute('data-testid', 'terminal-emulator')
        node.setAttribute('aria-label', 'terminal emulator')
        node.textContent = 'terminal emulator running'
        container.appendChild(node)
      }),
      loadAddon: vi.fn((_addon: unknown) => {}),
      onResize: vi.fn((callback: (size: { cols: number; rows: number }) => void) => {
        resizeListeners.push(callback)
      }),
      onData: vi.fn(),
      onRender: vi.fn((callback: () => void) => {
        renderListeners.push(callback)
        callback()
      }),
      write: vi.fn(),
      writeln: vi.fn(),
      attachCustomKeyEventHandler: vi.fn(),
      dispose: vi.fn(),
      emitResize: (cols: number, rows: number) => {
        instance.cols = cols
        instance.rows = rows
        for (const listener of resizeListeners) {
          listener({ cols, rows })
        }
      },
      emitRender: () => {
        for (const listener of renderListeners) {
          listener()
        }
      }
    }

    xtermInstances.push(instance)
    return instance
  })
}))

function resetTerminalRuntimeState() {
  useWorkspaceStore.setState({
    tabs: [],
    activeTabId: ''
  })

  localStorage.clear()

  resolveLaunchConfigMock.mockClear()
  terminalSessionMocks.length = 0
  xtermInstances.length = 0
  fitAddonInvocations.length = 0

  useAgentSessionLaunchPreferences.getState().setLaunchConfig({
    agent: 'codex',
    model: 'gpt-5-codex',
    effort: 'medium',
    thinking: true,
    permissionLevel: 'balanced'
  })
}

describe('WorkspacePanel terminal runtime', () => {
  beforeEach(() => {
    resetTerminalRuntimeState()

    class ResizeObserverMock {
      observe = vi.fn()
      unobserve = vi.fn()
      disconnect = vi.fn()
    }
    global.ResizeObserver = ResizeObserverMock as any

    global.requestAnimationFrame = vi.fn((callback: FrameRequestCallback) => {
      callback(0)
      return 0
    }) as any
  })

  it('creates a mounted terminal emulator surface after clicking +', async () => {
    const user = userEvent.setup()
    render(<WorkspacePanel />)

    await user.click(screen.getByRole('button', { name: /new terminal/i }))

    await waitFor(() => expect(resolveLaunchConfigMock).toHaveBeenCalledTimes(1))
    await waitFor(() => expect(terminalSessionMocks.length).toBeGreaterThan(0))
    await waitFor(() => expect(xtermInstances.length).toBeGreaterThan(0))

    expect(screen.getByRole('tab', { name: /Codex/i })).toBeInTheDocument()
    expect(screen.getByLabelText('terminal emulator')).toBeInTheDocument()
    expect(fitAddonInvocations.length).toBeGreaterThan(0)
    const latestFitAddon = fitAddonInvocations[fitAddonInvocations.length - 1]
    expect(latestFitAddon).toBeDefined()
    expect(latestFitAddon?.fit).toHaveBeenCalled()
    expect(terminalSessionMocks.some(({ session }) => session.markReady.mock.calls.length > 0)).toBe(true)
  })
})
