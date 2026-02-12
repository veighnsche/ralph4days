import { useCallback, useEffect, useRef, useState } from 'react'
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

function decodeOutputBytes(base64Data: string): Uint8Array {
  const binary = atob(base64Data)
  const bytes = new Uint8Array(binary.length)
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i)
  return bytes
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
  const sessionId = config.sessionId
  const mcpMode = config.mcpMode
  const taskId = config.taskId
  const agent = config.agent
  const model = config.model
  const effort = config.effort
  const thinking = config.thinking
  const permissionLevel = config.permissionLevel
  const humanSession = config.humanSession
  const humanKind = humanSession?.kind
  const humanAgent = humanSession?.agent
  const humanPostStartPreamble = humanSession?.postStartPreamble
  const humanInitPrompt = humanSession?.initPrompt
  const [listenersReady, setListenersReady] = useState(false)

  const markSessionStarted = useCallback(() => {
    sessionStartedRef.current = true
    if (pendingResizeRef.current) {
      const { cols, rows } = pendingResizeRef.current
      pendingResizeRef.current = null
      terminalBridgeResize(sessionId, cols, rows).catch(() => {})
    }
    if (startedNotifiedRef.current) return
    startedNotifiedRef.current = true
    handlersRef.current.onStarted?.()
  }, [sessionId])

  const flushPendingInput = useCallback(() => {
    if (!sessionStartedRef.current || pendingInputRef.current.length === 0) return

    const pending = [...pendingInputRef.current]
    pendingInputRef.current = []

    for (const data of pending) {
      terminalBridgeSendInput(sessionId, data).catch(err => handlersRef.current.onError?.(String(err)))
    }
  }, [sessionId])

  const notifyStartSuccess = useCallback(() => {
    markSessionStarted()
    flushPendingInput()
  }, [markSessionStarted, flushPendingInput])

  const startHumanSession = useCallback(() => {
    if (!humanKind) return null
    return terminalBridgeStartHumanSession({
      terminalSessionId: sessionId,
      kind: humanKind,
      taskId,
      agent: humanAgent ?? 'claude',
      model: model ?? undefined,
      effort: effort ?? undefined,
      ...(permissionLevel != null ? { permissionLevel } : {}),
      postStartPreamble: humanPostStartPreamble ?? undefined,
      initPrompt: humanInitPrompt ?? undefined,
      mcpMode: taskId !== undefined ? undefined : mcpMode || 'interactive',
      thinking: thinking ?? undefined
    })
  }, [
    effort,
    humanAgent,
    humanInitPrompt,
    humanKind,
    humanPostStartPreamble,
    mcpMode,
    model,
    permissionLevel,
    sessionId,
    taskId,
    thinking
  ])

  const startTaskSession = useCallback(() => {
    if (taskId === undefined) return null
    return terminalBridgeStartTaskSession({
      sessionId,
      taskId,
      agent: agent ?? 'claude',
      model: model ?? undefined,
      effort: effort ?? undefined,
      ...(permissionLevel != null ? { permissionLevel } : {}),
      thinking: thinking ?? undefined
    })
  }, [agent, effort, model, permissionLevel, sessionId, taskId, thinking])

  const startInteractiveSession = useCallback(() => {
    return terminalBridgeStartSession({
      sessionId,
      agent: agent ?? 'claude',
      mcpMode: mcpMode || 'interactive',
      model: model ?? undefined,
      effort: effort ?? undefined,
      ...(permissionLevel != null ? { permissionLevel } : {}),
      thinking: thinking ?? undefined
    })
  }, [agent, effort, mcpMode, model, permissionLevel, sessionId, thinking])

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
      terminalBridgeListenSessionOutput(sessionId, payload => {
        markSessionStarted()
        flushPendingInput()
        const bytes = decodeOutputBytes(payload.data)
        if (isReadyRef.current) {
          handlersRef.current.onOutput?.(bytes)
        } else {
          outputBufferRef.current.push(bytes)
        }
      }),
      terminalBridgeListenSessionClosed(sessionId, payload => {
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
  }, [flushPendingInput, isEnabled, markSessionStarted, sessionId])

  useEffect(() => {
    if (!(isEnabled && listenersReady) || startRequestedRef.current) return

    startRequestedRef.current = true

    const onStartFailed = (err: unknown) => {
      startRequestedRef.current = false
      handlersRef.current.onError?.(String(err))
    }
    const startPromise = startHumanSession() ?? startTaskSession() ?? startInteractiveSession()
    startPromise.then(notifyStartSuccess).catch(onStartFailed)
  }, [isEnabled, listenersReady, notifyStartSuccess, startHumanSession, startInteractiveSession, startTaskSession])

  useEffect(() => {
    if (!isEnabled) return

    return () => {
      terminalBridgeTerminate(sessionId).catch(() => {})
    }
  }, [isEnabled, sessionId])

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

    terminalBridgeSendInput(sessionId, data).catch(err => handlersRef.current.onError?.(String(err)))
  }

  const resize = (cols: number, rows: number) => {
    if (!sessionStartedRef.current) {
      pendingResizeRef.current = { cols, rows }
      return
    }
    terminalBridgeResize(sessionId, cols, rows).catch(() => {})
  }

  return { markReady, sendInput, resize }
}
