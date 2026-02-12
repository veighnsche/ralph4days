import { invoke } from '@tauri-apps/api/core'
import type { Terminal as XTerm } from '@xterm/xterm'
import { TerminalSquare } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { InlineError } from '@/components/shared'
import { useTabMeta } from '@/hooks/workspace'
import { Terminal, terminalBridgeEmitSystemMessage, useTerminalSession } from '@/lib/terminal'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'

// WHY: Claude Code welcome screen is left-aligned in PTY (upstream issue #5430)
// See: https://github.com/anthropics/claude-code/issues/5430

export function TerminalTabContent({ tab }: { tab: WorkspaceTab }) {
  useTabMeta(tab.id, tab.title, TerminalSquare)
  const terminalRef = useRef<XTerm | null>(null)
  const [sessionError, setSessionError] = useState<string | null>(null)
  const [agentSessionPersisted, setAgentSessionPersisted] = useState(false)
  const [bridgeStarted, setBridgeStarted] = useState(false)
  const agentSessionCreatedRef = useRef(false)
  const terminalReadyRef = useRef(false)
  const startMessageEmittedRef = useRef(false)

  const tryEmitSessionStartMessage = () => {
    if (startMessageEmittedRef.current) return
    if (!(agentSessionPersisted && bridgeStarted && terminalReadyRef.current && terminalRef.current)) return
    startMessageEmittedRef.current = true

    // Claude startup repaints the screen; delayed write keeps this visible.
    setTimeout(() => {
      terminalBridgeEmitSystemMessage(tab.id, `\x1b[2m[session ${tab.id} started]\x1b[0m\r\n`).catch(err => {
        setSessionError(String(err))
      })
    }, 1200)
  }

  useEffect(() => {
    if (agentSessionCreatedRef.current) return
    agentSessionCreatedRef.current = true

    const kind = tab.data?.taskId != null ? 'task_execution' : 'manual'
    const model = tab.data?.model
    const launchCommand = model != null ? `claude --model ${model}${tab.data?.thinking ? ' --thinking' : ''}` : 'claude'

    invoke('create_human_agent_session', {
      params: {
        id: tab.id,
        kind,
        taskId: tab.data?.taskId,
        agent: 'claude',
        model,
        launchCommand,
        postStartPreamble: null,
        initPrompt: tab.data?.initPrompt ?? null
      }
    })
      .then(() => {
        setAgentSessionPersisted(true)
      })
      .catch(err => {
        // Keep terminal usable even if session bookkeeping fails.
        console.warn('Failed to create human agent session:', err)
      })
  }, [tab.id, tab.data?.taskId, tab.data?.model, tab.data?.thinking, tab.data?.initPrompt])

  useEffect(() => {
    tryEmitSessionStartMessage()
  }, [agentSessionPersisted, bridgeStarted])

  const session = useTerminalSession(
    {
      sessionId: tab.id,
      mcpMode: tab.data?.taskId !== undefined ? undefined : 'interactive',
      taskId: tab.data?.taskId,
      model: tab.data?.model,
      thinking: tab.data?.thinking,
      enabled: agentSessionPersisted
    },
    {
      onStarted: () => setBridgeStarted(true),
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
    terminalReadyRef.current = true
    tryEmitSessionStartMessage()
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
