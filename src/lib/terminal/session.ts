import { useCallback, useEffect, useRef, useState } from 'react'
import { recordTerminalBridgeOutput, recordTerminalReady } from './diagnostics'
import {
  terminalBridgeListenSessionClosed,
  terminalBridgeListenSessionOutput,
  terminalBridgeReplayOutput,
  terminalBridgeResize,
  terminalBridgeSendInput,
  terminalBridgeSetStreamMode,
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
  isActive?: boolean
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

function encodeSystemMessage(text: string): Uint8Array {
  return new TextEncoder().encode(text)
}

function normalizeSeq(rawSeq: unknown): bigint {
  if (typeof rawSeq === 'bigint') {
    return rawSeq
  }

  if (typeof rawSeq === 'number') {
    if (!Number.isSafeInteger(rawSeq) || rawSeq < 0) {
      throw new Error(`[terminal_session] Invalid seq value: ${rawSeq}`)
    }
    return BigInt(rawSeq)
  }

  if (typeof rawSeq === 'string') {
    try {
      const parsed = BigInt(rawSeq)
      if (parsed < 0n) {
        throw new Error()
      }
      return parsed
    } catch {
      throw new Error(`[terminal_session] Invalid seq value: ${rawSeq}`)
    }
  }

  throw new Error(`[terminal_session] Invalid seq type: ${typeof rawSeq}`)
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
  const lastDeliveredSeqRef = useRef<bigint>(0n)
  const deliveredSystemSeqZeroRef = useRef(false)
  const isResumingRef = useRef(false)
  const queuedLiveOutputRef = useRef<Array<{ seq: bigint; data: string }>>([])
  handlersRef.current = handlers

  const isEnabled = config.enabled ?? true
  const isActive = config.isActive ?? true
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
  const [streamReady, setStreamReady] = useState(false)

  const emitOutputBytes = useCallback(
    (bytes: Uint8Array) => {
      recordTerminalBridgeOutput(sessionId, bytes)
      if (isReadyRef.current) {
        handlersRef.current.onOutput?.(bytes)
      } else {
        outputBufferRef.current.push(bytes)
      }
    },
    [sessionId]
  )

  const emitSystemMessage = useCallback(
    (text: string) => {
      emitOutputBytes(encodeSystemMessage(text))
    },
    [emitOutputBytes]
  )

  const deliverOutputChunk = useCallback(
    (seq: bigint, base64Data: string) => {
      if (seq === 0n) {
        if (deliveredSystemSeqZeroRef.current) return
        deliveredSystemSeqZeroRef.current = true
      } else if (seq <= lastDeliveredSeqRef.current) {
        return
      }
      const bytes = decodeOutputBytes(base64Data)
      if (seq !== 0n) {
        lastDeliveredSeqRef.current = seq
      }
      emitOutputBytes(bytes)
    },
    [emitOutputBytes]
  )

  const markSessionStarted = useCallback(() => {
    if (!sessionStartedRef.current) {
      sessionStartedRef.current = true
      setStreamReady(true)
    }
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

  const replayBufferedOutput = useCallback(
    async (afterSeq: bigint, limit = 256) => {
      let cursor = afterSeq
      let replayLoops = 0
      let truncationNotified = false

      while (replayLoops < 128) {
        replayLoops += 1
        const replayResult = await terminalBridgeReplayOutput(sessionId, cursor, limit)

        if (replayResult.truncated && !truncationNotified) {
          truncationNotified = true
          emitSystemMessage('\r\n\x1b[2m[output replay truncated while inactive]\x1b[0m\r\n')
        }

        for (const chunk of replayResult.chunks) {
          deliverOutputChunk(normalizeSeq(chunk.seq), chunk.data)
          cursor = normalizeSeq(chunk.seq)
        }

        if (!replayResult.hasMore) {
          break
        }
      }

      return cursor
    },
    [deliverOutputChunk, emitSystemMessage, sessionId]
  )

  const startHumanSession = useCallback(() => {
    if (!humanKind) return null
    return terminalBridgeStartHumanSession({
      terminalSessionId: sessionId,
      kind: humanKind,
      taskId,
      agent: humanAgent ?? 'codex',
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
      agent: agent ?? 'codex',
      model: model ?? undefined,
      effort: effort ?? undefined,
      ...(permissionLevel != null ? { permissionLevel } : {}),
      thinking: thinking ?? undefined
    })
  }, [agent, effort, model, permissionLevel, sessionId, taskId, thinking])

  const startInteractiveSession = useCallback(() => {
    return terminalBridgeStartSession({
      sessionId,
      agent: agent ?? 'codex',
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
    isReadyRef.current = false
    outputBufferRef.current = []
    pendingInputRef.current = []
    pendingResizeRef.current = null
    lastDeliveredSeqRef.current = 0n
    deliveredSystemSeqZeroRef.current = false
    isResumingRef.current = false
    queuedLiveOutputRef.current = []

    let active = true
    let unlistenOutput: (() => void) | null = null
    let unlistenClosed: (() => void) | null = null

    setListenersReady(false)
    setStreamReady(false)

    Promise.all([
      terminalBridgeListenSessionOutput(sessionId, payload => {
        let seq: bigint
        try {
          seq = normalizeSeq(payload.seq)
        } catch (error) {
          handlersRef.current.onError?.(String(error))
          return
        }

        try {
          decodeOutputBytes(payload.data)
        } catch (error) {
          handlersRef.current.onError?.(String(error))
          return
        }

        markSessionStarted()
        flushPendingInput()

        if (isResumingRef.current) {
          queuedLiveOutputRef.current.push({ seq, data: payload.data })
          return
        }

        try {
          deliverOutputChunk(seq, payload.data)
        } catch (error) {
          handlersRef.current.onError?.(String(error))
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
      setStreamReady(false)
      unlistenOutput?.()
      unlistenClosed?.()
    }
  }, [deliverOutputChunk, flushPendingInput, isEnabled, markSessionStarted, sessionId])

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
    if (!(isEnabled && listenersReady && streamReady)) return

    let cancelled = false

    const clearResumeState = () => {
      if (cancelled) return
      isResumingRef.current = false
      queuedLiveOutputRef.current = []
    }

    const flushQueuedLiveOutput = () => {
      const queued = [...queuedLiveOutputRef.current].sort((left, right) => {
        if (left.seq < right.seq) return -1
        if (left.seq > right.seq) return 1
        return 0
      })
      queuedLiveOutputRef.current = []
      for (const chunk of queued) {
        deliverOutputChunk(chunk.seq, chunk.data)
      }
    }

    const syncInactiveRuntimeMode = async () => {
      await terminalBridgeSetStreamMode(sessionId, 'buffered')
    }

    const syncActiveRuntimeMode = async () => {
      isResumingRef.current = true
      queuedLiveOutputRef.current = []

      await terminalBridgeSetStreamMode(sessionId, 'buffered')
      const cursor = await replayBufferedOutput(lastDeliveredSeqRef.current)
      await terminalBridgeSetStreamMode(sessionId, 'live')
      await replayBufferedOutput(cursor, 1024)
      flushQueuedLiveOutput()
    }

    const restoreLiveModeOnSyncFailure = async () => {
      if (cancelled || !isActive) return
      try {
        await terminalBridgeSetStreamMode(sessionId, 'live')
      } catch (restoreError) {
        handlersRef.current.onError?.(
          `[terminal_session] Failed to restore stream mode to live: ${String(restoreError)}`
        )
      }
    }

    const syncRuntimeMode = async () => {
      try {
        if (isActive) {
          await syncActiveRuntimeMode()
        } else {
          await syncInactiveRuntimeMode()
        }
      } catch (error) {
        await restoreLiveModeOnSyncFailure()
        throw error
      } finally {
        clearResumeState()
      }
    }

    syncRuntimeMode().catch(err => {
      if (cancelled) return
      handlersRef.current.onError?.(String(err))
    })

    return () => {
      cancelled = true
      isResumingRef.current = false
      queuedLiveOutputRef.current = []
    }
  }, [deliverOutputChunk, isActive, isEnabled, listenersReady, replayBufferedOutput, sessionId, streamReady])

  useEffect(() => {
    if (!isEnabled) return

    return () => {
      terminalBridgeTerminate(sessionId).catch(() => {})
    }
  }, [isEnabled, sessionId])

  const markReady = () => {
    recordTerminalReady(sessionId)
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
