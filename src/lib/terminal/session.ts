import { useEffect, useRef, useState } from 'react'
import {
  terminalBridgeListenSessionClosed,
  terminalBridgeListenSessionOutput,
  terminalBridgeResize,
  terminalBridgeSendInput,
  terminalBridgeStartHumanSession,
  terminalBridgeStartSession,
  terminalBridgeStartTaskSession,
  terminalBridgeTerminate
} from './terminalBridgeClient'

export interface TerminalSessionConfig {
  sessionId: string
  agent?: string
  mcpMode?: string
  taskId?: number
  model?: string | null
  effort?: 'low' | 'medium' | 'high' | null
  thinking?: boolean | null
  permissionLevel?: 'safe' | 'balanced' | 'auto' | 'full_auto' | null
  enabled?: boolean
  humanSession?: {
    kind: string
    agent?: string
    launchCommand?: string
    postStartPreamble?: string
    initPrompt?: string
  }
}

export interface TerminalSessionHandlers {
  onStarted?: () => void
  onOutput?: (data: Uint8Array) => void
  onClosed?: (exitCode: number) => void
  onError?: (error: string) => void
}

export function useTerminalSession(config: TerminalSessionConfig, handlers: TerminalSessionHandlers) {
  const isReadyRef = useRef(false)
  const outputBufferRef = useRef<Uint8Array[]>([])
  const pendingInputRef = useRef<string[]>([])
  const pendingResizeRef = useRef<{ cols: number; rows: number } | null>(null)
  const sessionStartedRef = useRef(false)
  const startedNotifiedRef = useRef(false)
  const startRequestedRef = useRef(false)
  const handlersRef = useRef(handlers)
  handlersRef.current = handlers
  const isEnabled = config.enabled ?? true
  const [listenersReady, setListenersReady] = useState(false)

  const markSessionStarted = () => {
    sessionStartedRef.current = true
    if (pendingResizeRef.current) {
      const { cols, rows } = pendingResizeRef.current
      pendingResizeRef.current = null
      terminalBridgeResize(config.sessionId, cols, rows).catch(() => {})
    }
    if (startedNotifiedRef.current) return
    startedNotifiedRef.current = true
    handlersRef.current.onStarted?.()
  }

  const flushPendingInput = () => {
    if (!sessionStartedRef.current || pendingInputRef.current.length === 0) return

    const pending = [...pendingInputRef.current]
    pendingInputRef.current = []

    for (const data of pending) {
      terminalBridgeSendInput(config.sessionId, data).catch(err => handlersRef.current.onError?.(String(err)))
    }
  }

  useEffect(() => {
    if (!isEnabled) return

    startRequestedRef.current = false
    sessionStartedRef.current = false
    startedNotifiedRef.current = false
    outputBufferRef.current = []
    pendingInputRef.current = []
    pendingResizeRef.current = null

    let active = true
    let unlistenOutput: (() => void) | null = null
    let unlistenClosed: (() => void) | null = null

    setListenersReady(false)

    Promise.all([
      terminalBridgeListenSessionOutput(config.sessionId, payload => {
        markSessionStarted()
        flushPendingInput()

        const binary = atob(payload.data)
        const bytes = new Uint8Array(binary.length)
        for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i)
        if (isReadyRef.current) {
          handlersRef.current.onOutput?.(bytes)
        } else {
          outputBufferRef.current.push(bytes)
        }
      }),
      terminalBridgeListenSessionClosed(config.sessionId, payload => {
        if (!sessionStartedRef.current) return
        handlersRef.current.onClosed?.(payload.exit_code)
      })
    ])
      .then(([outUnsub, closedUnsub]) => {
        unlistenOutput = outUnsub
        unlistenClosed = closedUnsub
        if (active) setListenersReady(true)
      })
      .catch(err => handlersRef.current.onError?.(String(err)))

    return () => {
      active = false
      setListenersReady(false)
      unlistenOutput?.()
      unlistenClosed?.()
    }
  }, [config.sessionId, isEnabled])

  useEffect(() => {
    if (!(isEnabled && listenersReady) || startRequestedRef.current) return

    startRequestedRef.current = true

    const onStartFailed = (err: unknown) => {
      startRequestedRef.current = false
      handlersRef.current.onError?.(String(err))
    }

    if (config.humanSession) {
      terminalBridgeStartHumanSession({
        terminalSessionId: config.sessionId,
        kind: config.humanSession.kind,
        taskId: config.taskId,
        agent: config.humanSession.agent ?? 'claude',
        model: config.model ?? undefined,
        effort: config.effort ?? undefined,
        ...(config.permissionLevel != null ? { permissionLevel: config.permissionLevel } : {}),
        launchCommand: config.humanSession.launchCommand ?? undefined,
        postStartPreamble: config.humanSession.postStartPreamble ?? undefined,
        initPrompt: config.humanSession.initPrompt ?? undefined,
        mcpMode: config.taskId !== undefined ? undefined : config.mcpMode || 'interactive',
        thinking: config.thinking ?? undefined
      })
        .then(() => {
          markSessionStarted()
          flushPendingInput()
        })
        .catch(onStartFailed)
    } else if (config.taskId !== undefined) {
      terminalBridgeStartTaskSession({
        sessionId: config.sessionId,
        taskId: config.taskId,
        agent: config.agent ?? 'claude',
        model: config.model ?? undefined,
        effort: config.effort ?? undefined,
        ...(config.permissionLevel != null ? { permissionLevel: config.permissionLevel } : {}),
        thinking: config.thinking ?? undefined
      })
        .then(() => {
          markSessionStarted()
          flushPendingInput()
        })
        .catch(onStartFailed)
    } else {
      terminalBridgeStartSession({
        sessionId: config.sessionId,
        agent: config.agent ?? 'claude',
        mcpMode: config.mcpMode || 'interactive',
        model: config.model ?? undefined,
        effort: config.effort ?? undefined,
        ...(config.permissionLevel != null ? { permissionLevel: config.permissionLevel } : {}),
        thinking: config.thinking ?? undefined
      })
        .then(() => {
          markSessionStarted()
          flushPendingInput()
        })
        .catch(onStartFailed)
    }
  }, [
    config.sessionId,
    config.mcpMode,
    config.taskId,
    config.agent,
    config.model,
    config.effort,
    config.thinking,
    config.permissionLevel,
    config.humanSession?.kind,
    config.humanSession?.agent,
    config.humanSession?.launchCommand,
    config.humanSession?.postStartPreamble,
    config.humanSession?.initPrompt,
    isEnabled,
    listenersReady
  ])

  useEffect(() => {
    if (!isEnabled) return

    return () => {
      terminalBridgeTerminate(config.sessionId).catch(() => {})
    }
  }, [config.sessionId, isEnabled])

  const markReady = () => {
    isReadyRef.current = true
    for (const chunk of outputBufferRef.current) {
      handlersRef.current.onOutput?.(chunk)
    }
    outputBufferRef.current = []
  }

  const sendInput = (data: string) => {
    if (!sessionStartedRef.current) {
      pendingInputRef.current.push(data)
      return
    }

    terminalBridgeSendInput(config.sessionId, data).catch(err => handlersRef.current.onError?.(String(err)))
  }

  const resize = (cols: number, rows: number) => {
    if (!sessionStartedRef.current) {
      pendingResizeRef.current = { cols, rows }
      return
    }
    terminalBridgeResize(config.sessionId, cols, rows).catch(() => {})
  }

  return { markReady, sendInput, resize }
}
