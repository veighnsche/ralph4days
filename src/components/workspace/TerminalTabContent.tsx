import type { Terminal as XTerm } from '@xterm/xterm'
import { TerminalSquare } from 'lucide-react'
import { useRef, useState } from 'react'
import { InlineError } from '@/components/shared'
import type { PermissionLevel } from '@/hooks/preferences'
import { useTabMeta } from '@/hooks/workspace'
import { Terminal, useTerminalSession } from '@/lib/terminal'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'

// WHY: Claude Code welcome screen is left-aligned in PTY (upstream issue #5430)
// See: https://github.com/anthropics/claude-code/issues/5430

export function TerminalTabContent({ tab }: { tab: WorkspaceTab }) {
  useTabMeta(tab.id, tab.title, TerminalSquare)
  const terminalRef = useRef<XTerm | null>(null)
  const [sessionError, setSessionError] = useState<string | null>(null)
  const kind = tab.data?.taskId != null ? 'task_execution' : 'manual'
  const agent = tab.data?.agent ?? 'claude'
  const model = tab.data?.model
  const effort = tab.data?.effort
  const permissionLevel = (tab.data?.permissionLevel as PermissionLevel | undefined) ?? 'balanced'
  const launchCommand = buildLaunchCommand(agent, model, effort, permissionLevel)

  const session = useTerminalSession(
    {
      sessionId: tab.id,
      agent,
      mcpMode: tab.data?.taskId !== undefined ? undefined : 'interactive',
      taskId: tab.data?.taskId,
      model: tab.data?.model,
      effort: tab.data?.effort,
      thinking: tab.data?.thinking,
      permissionLevel,
      humanSession: {
        kind,
        agent,
        launchCommand,
        postStartPreamble: undefined,
        initPrompt: tab.data?.initPrompt ?? undefined
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

function buildLaunchCommand(
  agent: string,
  model?: string,
  effort?: 'low' | 'medium' | 'high',
  permissionLevel: PermissionLevel = 'balanced'
) {
  if (agent === 'codex') {
    const modelArg = model != null ? ` --model ${model}` : ''
    const permissionArg =
      permissionLevel === 'safe'
        ? ' --sandbox workspace-write --ask-for-approval untrusted'
        : permissionLevel === 'balanced'
          ? ' --sandbox workspace-write --ask-for-approval on-request'
          : permissionLevel === 'auto'
            ? ' --full-auto'
            : ' --dangerously-bypass-approvals-and-sandbox'
    return `codex${modelArg}${permissionArg}`
  }
  const modelArg = model != null ? ` --model ${model}` : ''
  const effortArg = effort != null ? ` --effort ${effort}` : ''
  const permissionArg =
    permissionLevel === 'safe'
      ? ' --permission-mode default'
      : permissionLevel === 'balanced'
        ? ' --permission-mode delegate'
        : permissionLevel === 'auto'
          ? ' --permission-mode dontAsk'
          : ' --permission-mode bypassPermissions'
  return `claude${modelArg}${effortArg}${permissionArg}`
}
