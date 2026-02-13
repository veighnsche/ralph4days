import { act, render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useAgentSessionLaunchPreferences } from '@/hooks/preferences'
import type { AgentSessionLaunchConfig } from '@/lib/agent-session-launch-config'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'
import { WorkspacePanel } from './WorkspacePanel'

const { resolveLaunchConfigMock, useTerminalSessionMock, sessionCalls } = vi.hoisted(() => ({
  resolveLaunchConfigMock: vi.fn(async (config: AgentSessionLaunchConfig): Promise<AgentSessionLaunchConfig> => config),
  useTerminalSessionMock: vi.fn(),
  sessionCalls: [] as Array<{
    config: unknown
    handlers: {
      onError?: (error: string) => void
    }
  }>
}))

vi.mock('@/components/agent-session-launch/resolveLaunchConfig', () => ({
  resolveLaunchConfigAgainstCatalog: resolveLaunchConfigMock
}))

vi.mock('@/lib/terminal', () => ({
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
  useTerminalSession: (config: unknown, handlers: { onError?: (error: string) => void }) => {
    useTerminalSessionMock(config)
    sessionCalls.push({ config, handlers })
    return {
      markReady: vi.fn(),
      sendInput: vi.fn(),
      resize: vi.fn()
    }
  }
}))

const CODEX_LAUNCH_CONFIG: AgentSessionLaunchConfig = {
  agent: 'codex',
  model: 'gpt-5-codex',
  effort: 'medium',
  thinking: true,
  permissionLevel: 'balanced'
}

function resetWorkspaceState() {
  useWorkspaceStore.setState({
    tabs: [],
    activeTabId: ''
  })

  sessionCalls.length = 0
  resolveLaunchConfigMock.mockClear()
  useTerminalSessionMock.mockClear()

  useAgentSessionLaunchPreferences.getState().setLaunchConfig(CODEX_LAUNCH_CONFIG)
}

describe('WorkspacePanel', () => {
  beforeEach(() => {
    localStorage.clear()
    resetWorkspaceState()
  })

  it('opens a Codex terminal tab when plus button is clicked', async () => {
    const user = userEvent.setup()
    render(<WorkspacePanel />)

    expect(screen.getByText('No workspace tabs open')).toBeInTheDocument()

    const newTabButton = screen.getByRole('button', { name: /new terminal/i })
    await user.click(newTabButton)

    await waitFor(() => {
      expect(resolveLaunchConfigMock).toHaveBeenCalledTimes(1)
    })
    expect(resolveLaunchConfigMock).toHaveBeenCalledWith(CODEX_LAUNCH_CONFIG)

    await waitFor(() => {
      expect(sessionCalls).toHaveLength(1)
    })

    const tab = await screen.findByRole('tab', { name: /Codex/i })
    expect(tab).toBeTruthy()
    expect(screen.getByTestId('terminal')).toBeVisible()
    expect(screen.queryByText('No workspace tabs open')).not.toBeInTheDocument()

    const config = sessionCalls[0]!.config as {
      agent?: string
      humanSession?: {
        kind: string
        agent?: string
      }
      sessionId: string
    }

    expect(config.agent).toBe('codex')
    expect(config.humanSession?.kind).toBe('manual')
    expect(config.humanSession?.agent).toBe('codex')
    expect(config.sessionId.startsWith('terminal-')).toBe(true)
  })

  it('surfaces terminal session errors in the UI', async () => {
    const user = userEvent.setup()
    render(<WorkspacePanel />)

    await user.click(screen.getByRole('button', { name: /new terminal/i }))

    await waitFor(() => {
      expect(sessionCalls).toHaveLength(1)
    })

    const handlers = sessionCalls[0]!.handlers
    const errorMessage = 'TypeError: JSON.stringify cannot serialize BigInt'

    act(() => {
      handlers.onError?.(errorMessage)
    })

    const alert = await screen.findByRole('alert')
    expect(alert).toHaveTextContent('JSON.stringify cannot serialize BigInt')
  })
})
