import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useEffect, useRef } from 'react'

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
  const sessionStartedRef = useRef(false)
  const handlersRef = useRef(handlers)
  handlersRef.current = handlers

  useEffect(() => {
    if (config.taskId !== undefined) {
      invoke('create_pty_session_for_task', {
        sessionId: config.sessionId,
        taskId: config.taskId,
        model: config.model || null,
        thinking: config.thinking ?? null
      }).catch(err => handlersRef.current.onError?.(String(err)))
    } else {
      invoke('create_pty_session', {
        sessionId: config.sessionId,
        mcpMode: config.mcpMode || 'interactive',
        model: config.model || null,
        thinking: config.thinking ?? null
      }).catch(err => handlersRef.current.onError?.(String(err)))
    }

    return () => {
      invoke('terminate_pty_session', { sessionId: config.sessionId }).catch(() => {})
    }
  }, [config.sessionId, config.mcpMode, config.taskId, config.model, config.thinking])

  useEffect(() => {
    const unlisten = listen<{ session_id: string; data: string }>('ralph://pty_output', event => {
      if (event.payload.session_id !== config.sessionId) return
      sessionStartedRef.current = true

      const binary = atob(event.payload.data)
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
    const unlisten = listen<{ session_id: string; exit_code: number }>('ralph://pty_closed', event => {
      if (event.payload.session_id === config.sessionId && sessionStartedRef.current) {
        handlersRef.current.onClosed?.(event.payload.exit_code)
      }
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
    const bytes = Array.from(new TextEncoder().encode(data))
    invoke('send_terminal_input', {
      sessionId: config.sessionId,
      data: bytes
    }).catch(err => handlersRef.current.onError?.(String(err)))
  }

  const resize = (cols: number, rows: number) => {
    invoke('resize_pty', {
      sessionId: config.sessionId,
      cols,
      rows
    }).catch(() => {})
  }

  return { markReady, sendInput, resize }
}
