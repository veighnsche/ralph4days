import { useEffect, useRef } from 'react'
import {
  terminalBridgeListenSessionClosed,
  terminalBridgeListenSessionOutput,
  terminalBridgeResize,
  terminalBridgeSendInput,
  terminalBridgeStartSession,
  terminalBridgeStartTaskSession,
  terminalBridgeTerminate
} from './terminalBridgeClient'

export interface TerminalSessionConfig {
  sessionId: string
  mcpMode?: string
  taskId?: number
  model?: string | null
  thinking?: boolean | null
}

export interface TerminalSessionHandlers {
  onOutput?: (data: Uint8Array) => void
  onClosed?: (exitCode: number) => void
  onError?: (error: string) => void
}

export function useTerminalSession(config: TerminalSessionConfig, handlers: TerminalSessionHandlers) {
  const isReadyRef = useRef(false)
  const outputBufferRef = useRef<Uint8Array[]>([])
  const pendingInputRef = useRef<string[]>([])
  const sessionStartedRef = useRef(false)
  const handlersRef = useRef(handlers)
  handlersRef.current = handlers

  const flushPendingInput = () => {
    if (!sessionStartedRef.current || pendingInputRef.current.length === 0) return

    const pending = [...pendingInputRef.current]
    pendingInputRef.current = []

    for (const data of pending) {
      terminalBridgeSendInput(config.sessionId, data).catch(err => handlersRef.current.onError?.(String(err)))
    }
  }

  useEffect(() => {
    if (config.taskId !== undefined) {
      terminalBridgeStartTaskSession({
        sessionId: config.sessionId,
        taskId: config.taskId,
        model: config.model || null,
        thinking: config.thinking ?? null
      })
        .then(() => {
          sessionStartedRef.current = true
          flushPendingInput()
        })
        .catch(err => handlersRef.current.onError?.(String(err)))
    } else {
      terminalBridgeStartSession({
        sessionId: config.sessionId,
        mcpMode: config.mcpMode || 'interactive',
        model: config.model || null,
        thinking: config.thinking ?? null
      })
        .then(() => {
          sessionStartedRef.current = true
          flushPendingInput()
        })
        .catch(err => handlersRef.current.onError?.(String(err)))
    }

    return () => {
      terminalBridgeTerminate(config.sessionId).catch(() => {})
    }
  }, [config.sessionId, config.mcpMode, config.taskId, config.model, config.thinking])

  useEffect(() => {
    const unlisten = terminalBridgeListenSessionOutput(config.sessionId, payload => {
      sessionStartedRef.current = true
      flushPendingInput()

      const binary = atob(payload.data)
      const bytes = new Uint8Array(binary.length)
      for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i)
      if (isReadyRef.current) {
        handlersRef.current.onOutput?.(bytes)
      } else {
        outputBufferRef.current.push(bytes)
      }
    })

    return () => {
      unlisten.then(unsub => unsub())
    }
  }, [config.sessionId])

  useEffect(() => {
    const unlisten = terminalBridgeListenSessionClosed(config.sessionId, payload => {
      if (!sessionStartedRef.current) return
      handlersRef.current.onClosed?.(payload.exit_code)
    })

    return () => {
      unlisten.then(unsub => unsub())
    }
  }, [config.sessionId])

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
    terminalBridgeResize(config.sessionId, cols, rows).catch(() => {})
  }

  return { markReady, sendInput, resize }
}
