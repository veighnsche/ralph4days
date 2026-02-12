import type { Terminal as XTerm } from '@xterm/xterm'
import { TerminalSquare } from 'lucide-react'
import { useRef, useState } from 'react'
import { InlineError } from '@/components/shared'
import { useTabMeta } from '@/hooks/workspace'
import { Terminal, useTerminalSession } from '@/lib/terminal'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import { parseTerminalTabParams } from './schema'

// WHY: Claude Code welcome screen is left-aligned in PTY (upstream issue #5430)
// See: https://github.com/anthropics/claude-code/issues/5430

export function TerminalTabContent({ tab }: { tab: WorkspaceTab }) {
  useTabMeta(tab.id, tab.title, TerminalSquare)
  const terminalRef = useRef<XTerm | null>(null)
  const [sessionError, setSessionError] = useState<string | null>(null)
  const params = parseTerminalTabParams(tab.params)
  const kind = params.taskId != null ? 'task_execution' : 'manual'
  const agent = params.agent ?? 'claude'
  const permissionLevel = params.permissionLevel ?? 'balanced'

  const session = useTerminalSession(
    {
      sessionId: tab.id,
      agent,
      mcpMode: params.taskId !== undefined ? undefined : 'interactive',
      taskId: params.taskId,
      model: params.model,
      effort: params.effort,
      thinking: params.thinking,
      permissionLevel,
      humanSession: {
        kind,
        agent,
        initPrompt: params.initPrompt ?? undefined
      }
    },
    {
      onOutput: data => terminalRef.current?.write(data),
      onClosed: () => {
        terminalRef.current?.writeln('\r\n\x1b[2m[Session ended]\x1b[0m')
      },
      onError: err => setSessionError(err)
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

  return (
    <div className="h-full flex flex-col">
      <InlineError error={sessionError} onDismiss={() => setSessionError(null)} />
      <div className="flex-1 min-h-0">
        <Terminal onReady={handleReady} onResize={handleResize} />
      </div>
    </div>
  )
}
