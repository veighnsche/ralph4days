import { render, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import { TerminalTabContent } from './TerminalTabContent'

vi.mock('@/lib/terminal', () => ({
  terminalBridgeEmitSystemMessage: vi.fn().mockResolvedValue(undefined),
  Terminal: ({ onReady }: { onReady?: (terminal: unknown) => void }) => {
    if (onReady) {
      setTimeout(() => {
        onReady({
          cols: 80,
          rows: 24,
          write: vi.fn(),
          writeln: vi.fn(),
          onData: vi.fn(),
          attachCustomKeyEventHandler: vi.fn()
        })
      }, 0)
    }
    return <div data-testid="terminal">Terminal</div>
  },
  useTerminalSession: () => ({
    markReady: vi.fn(),
    sendInput: vi.fn(),
    resize: vi.fn()
  })
}))

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined)
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
})
