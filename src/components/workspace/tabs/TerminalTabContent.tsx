import type { Terminal as XTerm } from '@xterm/xterm'
import { TerminalSquare } from 'lucide-react'
import { useRef, useState } from 'react'
import { InlineError } from '@/components/shared'
import type { Agent, AgentSessionLaunchConfig, Effort, PermissionLevel } from '@/hooks/preferences'
import { useTabMeta } from '@/hooks/workspace'
import { Terminal, useTerminalSession } from '@/lib/terminal'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'

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

function agentLabel(agent: string | undefined): string {
  return agent === 'codex' ? 'Codex' : 'Claude'
}

export type TerminalTabParams = Partial<AgentSessionLaunchConfig> & {
  title?: string
  taskId?: number
  initPrompt?: string
}

function parseAgentField(value: unknown): Agent {
  if (value !== 'claude' && value !== 'codex') {
    throw new Error('Invalid terminal tab params.agent')
  }
  return value
}

function parseModelField(value: unknown): string {
  if (typeof value !== 'string' || value.trim() === '') {
    throw new Error('Invalid terminal tab params.model')
  }
  return value
}

function parseEffortField(value: unknown): Effort {
  if (value !== 'low' && value !== 'medium' && value !== 'high') {
    throw new Error('Invalid terminal tab params.effort')
  }
  return value
}

function parseBooleanField(value: unknown, key: 'thinking'): boolean {
  if (typeof value !== 'boolean') {
    throw new Error(`Invalid terminal tab params.${key}`)
  }
  return value
}

function parsePermissionLevelField(value: unknown): PermissionLevel {
  if (value !== 'safe' && value !== 'balanced' && value !== 'auto' && value !== 'full_auto') {
    throw new Error('Invalid terminal tab params.permissionLevel')
  }
  return value
}

function parseTaskIdField(value: unknown): number {
  if (!Number.isInteger(value) || Number(value) <= 0) {
    throw new Error('Invalid terminal tab params.taskId')
  }
  return value as number
}

function parseStringField(value: unknown, key: 'initPrompt' | 'title'): string {
  if (typeof value !== 'string' || (key === 'title' && value.trim() === '')) {
    throw new Error(`Invalid terminal tab params.${key}`)
  }
  return value
}

function parseTerminalTabParams(params: unknown): TerminalTabParams {
  if (params == null) return {}
  if (typeof params !== 'object' || Array.isArray(params)) {
    throw new Error('Invalid terminal tab params: expected object')
  }
  const candidate = params as Record<string, unknown>
  const parsed: TerminalTabParams = {}
  if (candidate.agent !== undefined) parsed.agent = parseAgentField(candidate.agent)
  if (candidate.model !== undefined) parsed.model = parseModelField(candidate.model)
  if (candidate.effort !== undefined) parsed.effort = parseEffortField(candidate.effort)
  if (candidate.thinking !== undefined) parsed.thinking = parseBooleanField(candidate.thinking, 'thinking')
  if (candidate.permissionLevel !== undefined) {
    parsed.permissionLevel = parsePermissionLevelField(candidate.permissionLevel)
  }
  if (candidate.taskId !== undefined) parsed.taskId = parseTaskIdField(candidate.taskId)
  if (candidate.initPrompt !== undefined) parsed.initPrompt = parseStringField(candidate.initPrompt, 'initPrompt')
  if (candidate.title !== undefined) parsed.title = parseStringField(candidate.title, 'title')
  return parsed
}

export function createTerminalTab(input: TerminalTabParams = {}): Omit<WorkspaceTab, 'id'> {
  const title = input.title ?? `${agentLabel(input.agent)} (${input.model ?? 'default'})`
  return {
    type: 'terminal',
    title,
    closeable: true,
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
