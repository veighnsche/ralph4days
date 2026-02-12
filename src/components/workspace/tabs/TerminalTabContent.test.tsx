import { render, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import { TerminalTabContent } from './TerminalTabContent'

const { useTerminalSessionMock } = vi.hoisted(() => ({
  useTerminalSessionMock: vi.fn()
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
  useTerminalSession: (config: unknown) => {
    useTerminalSessionMock(config)
    return {
      markReady: vi.fn(),
      sendInput: vi.fn(),
      resize: vi.fn()
    }
  }
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
    title: 'Terminal 1',
    closeable: true,
    params: {
      model: 'haiku',
      thinking: true
    }
  }

  beforeEach(() => {
    vi.clearAllMocks()
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

  it('starts through backend human session path', async () => {
    render(<TerminalTabContent tab={mockTab} />)
    await waitFor(() => expect(useTerminalSessionMock).toHaveBeenCalled())

    const config = useTerminalSessionMock.mock.calls[0][0] as {
      sessionId: string
      humanSession?: { kind: string; agent?: string }
    }
    expect(config.sessionId).toBe('test-terminal-1')
    expect(config.humanSession?.kind).toBe('manual')
    expect(config.humanSession?.agent).toBe('claude')
  })

  it('supports codex agent launch metadata', async () => {
    const codexTab: WorkspaceTab = {
      ...mockTab,
      id: 'test-terminal-codex',
      params: {
        agent: 'codex',
        model: 'gpt-5-codex',
        thinking: true
      }
    }

    render(<TerminalTabContent tab={codexTab} />)
    await waitFor(() => expect(useTerminalSessionMock).toHaveBeenCalled())

    const config = useTerminalSessionMock.mock.calls[0][0] as {
      agent?: string
      humanSession?: { agent?: string }
    }
    expect(config.agent).toBe('codex')
    expect(config.humanSession?.agent).toBe('codex')
  })
})
