import type { Terminal as XTerm } from '@xterm/xterm'
import { TerminalSquare } from 'lucide-react'
import { useRef } from 'react'
import { useTabMeta } from '@/hooks/useTabMeta'
import { Terminal, useTerminalSession } from '@/lib/terminal'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'

// NOTE TO FUTURE DEVELOPERS (and Vince):
// Claude Code's welcome screen renders LEFT-ALIGNED in PTY environments. This is NOT a Ralph bug.
// Claude Code uses React + Ink (Yoga layout) which defensively falls back to left-alignment when
// terminal width detection is uncertain — standard behavior in embedded PTY terminals.
// See: https://github.com/anthropics/claude-code/issues/5430
// DO NOT waste time trying to "fix" centering. It's upstream. Move on to features that matter.

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

  // Terminal uses refs for these callbacks (called once at mount, never re-evaluated),
  // so memoization is unnecessary.
  const handleReady = (terminal: XTerm) => {
    terminalRef.current = terminal
    requestAnimationFrame(() => session.resize(terminal.cols, terminal.rows))
    session.markReady()
    terminal.onData(data => session.sendInput(data))

    // xterm.js doesn't implement the kitty keyboard protocol, so Shift+Enter
    // sends \r (same as Enter). Real terminals (Ghostty, Kitty, Konsole) send
    // the CSI u sequence \x1b[13;2u which Claude Code needs for multi-line input.
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
