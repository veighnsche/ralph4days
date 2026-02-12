import type { Terminal as XTerm } from '@xterm/xterm'
import { TerminalSquare } from 'lucide-react'
import { useRef, useState } from 'react'
import { InlineError } from '@/components/shared'
import type { Agent, AgentSessionLaunchConfig, Effort, PermissionLevel } from '@/hooks/preferences'
import { useTabMeta } from '@/hooks/workspace'
import { Terminal, useTerminalSession } from '@/lib/terminal'
import { NOOP_TAB_LIFECYCLE, type WorkspaceTab } from '@/stores/useWorkspaceStore'

// WHY: Claude Code welcome screen is left-aligned in PTY (upstream issue #5430)
// See: https://github.com/anthropics/claude-code/issues/5430

export function TerminalTabContent({ tab }: { tab: WorkspaceTab }) {
  useTabMeta(tab.id, tab.title, TerminalSquare)
  const terminalRef = useRef<XTerm | null>(null)
  const [sessionError, setSessionError] = useState<string | null>(null)
  const params = parseTerminalTabParams(tab.params)
  const kind = params.taskId != null ? 'task_execution' : 'manual'
  const agent = params.agent ?? 'claude'
  const model = params.model
  const effort = params.effort
  const permissionLevel = params.permissionLevel ?? 'balanced'
  const launchCommand = buildLaunchCommand(agent, model, effort, permissionLevel)

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
        launchCommand,
        postStartPreamble: undefined,
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

function agentLabel(agent: string | undefined): string {
  return agent === 'codex' ? 'Codex' : 'Claude'
}

export type TerminalTabParams = Partial<AgentSessionLaunchConfig> & {
  title?: string
  taskId?: number
  initPrompt?: string
}

function parseTerminalTabParams(params: unknown): TerminalTabParams {
  if (params == null) return {}
  if (typeof params !== 'object' || Array.isArray(params)) {
    throw new Error('Invalid terminal tab params: expected object')
  }
  const candidate = params as Record<string, unknown>
  const parsed: TerminalTabParams = {}
  if (candidate.agent !== undefined) {
    if (candidate.agent !== 'claude' && candidate.agent !== 'codex') {
      throw new Error('Invalid terminal tab params.agent')
    }
    parsed.agent = candidate.agent as Agent
  }
  if (candidate.model !== undefined) {
    if (typeof candidate.model !== 'string' || candidate.model.trim() === '') {
      throw new Error('Invalid terminal tab params.model')
    }
    parsed.model = candidate.model
  }
  if (candidate.effort !== undefined) {
    if (candidate.effort !== 'low' && candidate.effort !== 'medium' && candidate.effort !== 'high') {
      throw new Error('Invalid terminal tab params.effort')
    }
    parsed.effort = candidate.effort as Effort
  }
  if (candidate.thinking !== undefined) {
    if (typeof candidate.thinking !== 'boolean') {
      throw new Error('Invalid terminal tab params.thinking')
    }
    parsed.thinking = candidate.thinking
  }
  if (candidate.permissionLevel !== undefined) {
    if (
      candidate.permissionLevel !== 'safe' &&
      candidate.permissionLevel !== 'balanced' &&
      candidate.permissionLevel !== 'auto' &&
      candidate.permissionLevel !== 'full_auto'
    ) {
      throw new Error('Invalid terminal tab params.permissionLevel')
    }
    parsed.permissionLevel = candidate.permissionLevel as PermissionLevel
  }
  if (candidate.taskId !== undefined) {
    if (!Number.isInteger(candidate.taskId) || Number(candidate.taskId) <= 0) {
      throw new Error('Invalid terminal tab params.taskId')
    }
    parsed.taskId = candidate.taskId as number
  }
  if (candidate.initPrompt !== undefined) {
    if (typeof candidate.initPrompt !== 'string') {
      throw new Error('Invalid terminal tab params.initPrompt')
    }
    parsed.initPrompt = candidate.initPrompt
  }
  if (candidate.title !== undefined) {
    if (typeof candidate.title !== 'string' || candidate.title.trim() === '') {
      throw new Error('Invalid terminal tab params.title')
    }
    parsed.title = candidate.title
  }
  return parsed
}

export function createTerminalTab(input: TerminalTabParams = {}): Omit<WorkspaceTab, 'id'> {
  const title = input.title ?? `${agentLabel(input.agent)} (${input.model ?? 'default'})`
  return {
    type: 'terminal',
    component: TerminalTabContent,
    title,
    closeable: true,
    lifecycle: NOOP_TAB_LIFECYCLE,
    params: {
      agent: input.agent,
      model: input.model,
      effort: input.effort,
      thinking: input.thinking,
      permissionLevel: input.permissionLevel,
      taskId: input.taskId,
      initPrompt: input.initPrompt
    }
  }
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
