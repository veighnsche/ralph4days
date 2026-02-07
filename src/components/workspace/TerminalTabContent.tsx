import type { Terminal as XTerm } from '@xterm/xterm'
import { TerminalSquare } from 'lucide-react'
import { useRef } from 'react'
import { useTabMeta } from '@/hooks/useTabMeta'
import { Terminal, useTerminalSession } from '@/lib/terminal'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'

// WHY: Claude Code welcome screen is left-aligned in PTY (upstream issue #5430)
// See: https://github.com/anthropics/claude-code/issues/5430

export function TerminalTabContent({ tab }: { tab: WorkspaceTab }) {
  useTabMeta(tab.id, tab.title, TerminalSquare)
  const terminalRef = useRef<XTerm | null>(null)

  const session = useTerminalSession(
    {
      sessionId: tab.id,
      mcpMode: 'interactive',
      model: tab.data?.model,
      thinking: tab.data?.thinking
    },
    {
      onOutput: data => terminalRef.current?.write(data),
      onClosed: () => {
        terminalRef.current?.writeln('\r\n\x1b[2m[Session ended]\x1b[0m')
      },
      onError: err => console.error('Terminal error:', err)
    }
  )

  // WHY: Callbacks use refs (invoked once at mount), so memoization unnecessary
  const handleReady = (terminal: XTerm) => {
    terminalRef.current = terminal
    requestAnimationFrame(() => session.resize(terminal.cols, terminal.rows))
    session.markReady()
    terminal.onData(data => session.sendInput(data))

    // WHY: xterm.js doesn't implement kitty protocol for Shift+Enter; need CSI u seq \x1b[13;2u
    terminal.attachCustomKeyEventHandler(event => {
      if (
        event.type === 'keydown' &&
        event.key === 'Enter' &&
        event.shiftKey &&
        !event.ctrlKey &&
        !event.altKey &&
        !event.metaKey
      ) {
        session.sendInput('\x1b[13;2u')
        return false
      }
      return true
    })
  }

  const handleResize = ({ cols, rows }: { cols: number; rows: number }) => {
    session.resize(cols, rows)
  }

  return <Terminal onReady={handleReady} onResize={handleResize} />
}
