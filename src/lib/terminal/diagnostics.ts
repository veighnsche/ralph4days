import type { Terminal as XTerm } from '@xterm/xterm'

const PREVIEW_MAX = 180
const decoder = new TextDecoder()
const ESC = 0x1b
const BEL = 0x07
const BACKSLASH = 0x5c

interface TerminalSessionDiagnostics {
  bridgeChunks: number
  bridgeBytes: number
  bridgePrintableBytes: number
  writeCalls: number
  writeBytes: number
  renderEvents: number
  nonEmptyRenderEvents: number
  maxVisibleNonEmptyLines: number
  readyMarks: number
  lastPreview: string
  lastPlainPreview: string
  lastRenderPreview: string
}

interface TerminalDiagnosticsState {
  sessions: Record<string, TerminalSessionDiagnostics>
}

declare global {
  interface Window {
    __RALPH_TERMINAL_DIAG__?: TerminalDiagnosticsState
  }
}

function getDiagnosticsState(): TerminalDiagnosticsState | null {
  if (typeof window === 'undefined') return null
  if (!window.__RALPH_TERMINAL_DIAG__) {
    window.__RALPH_TERMINAL_DIAG__ = { sessions: {} }
  }
  return window.__RALPH_TERMINAL_DIAG__
}

function getSessionDiagnostics(sessionId: string): TerminalSessionDiagnostics | null {
  const state = getDiagnosticsState()
  if (!state) return null

  const existing = state.sessions[sessionId]
  if (existing) return existing

  const created: TerminalSessionDiagnostics = {
    bridgeChunks: 0,
    bridgeBytes: 0,
    bridgePrintableBytes: 0,
    writeCalls: 0,
    writeBytes: 0,
    renderEvents: 0,
    nonEmptyRenderEvents: 0,
    maxVisibleNonEmptyLines: 0,
    readyMarks: 0,
    lastPreview: '',
    lastPlainPreview: '',
    lastRenderPreview: ''
  }
  state.sessions[sessionId] = created
  return created
}

function countPrintableBytes(bytes: Uint8Array): number {
  let printable = 0
  for (const value of bytes) {
    if (value === 9 || value === 10 || value === 13 || (value >= 32 && value <= 126)) {
      printable += 1
    }
  }
  return printable
}

function stripAnsiForPreview(text: string): string {
  let output = ''
  let index = 0

  while (index < text.length) {
    const code = text.charCodeAt(index)
    if (code !== ESC) {
      output += text[index]
      index += 1
      continue
    }

    const next = text.charCodeAt(index + 1)

    // Skip CSI: ESC [ ... <final-byte>
    if (next === 0x5b) {
      index += 2
      while (index < text.length) {
        const finalByte = text.charCodeAt(index)
        index += 1
        if (finalByte >= 0x40 && finalByte <= 0x7e) {
          break
        }
      }
      continue
    }

    // Skip OSC: ESC ] ... BEL or ESC \
    if (next === 0x5d) {
      index += 2
      while (index < text.length) {
        const oscByte = text.charCodeAt(index)
        if (oscByte === BEL) {
          index += 1
          break
        }
        if (oscByte === ESC && text.charCodeAt(index + 1) === BACKSLASH) {
          index += 2
          break
        }
        index += 1
      }
      continue
    }

    // Skip bare escape and one following byte.
    index += index + 1 < text.length ? 2 : 1
  }

  return output
}

function updatePreview(diag: TerminalSessionDiagnostics, bytes: Uint8Array) {
  const decoded = decoder.decode(bytes)
  const normalized = decoded.replace(/\s+/g, ' ').trim()
  if (normalized.length === 0) return
  diag.lastPreview = normalized.slice(0, PREVIEW_MAX)
  const plainNormalized = stripAnsiForPreview(decoded).replace(/\s+/g, ' ').trim()
  if (plainNormalized.length > 0) {
    diag.lastPlainPreview = plainNormalized.slice(0, PREVIEW_MAX)
  }
}

export function recordTerminalBridgeOutput(sessionId: string, bytes: Uint8Array) {
  const diag = getSessionDiagnostics(sessionId)
  if (!diag) return
  diag.bridgeChunks += 1
  diag.bridgeBytes += bytes.byteLength
  diag.bridgePrintableBytes += countPrintableBytes(bytes)
  updatePreview(diag, bytes)
}

export function recordTerminalWrite(sessionId: string, bytes: Uint8Array) {
  const diag = getSessionDiagnostics(sessionId)
  if (!diag) return
  diag.writeCalls += 1
  diag.writeBytes += bytes.byteLength
  updatePreview(diag, bytes)
}

export function recordTerminalReady(sessionId: string) {
  const diag = getSessionDiagnostics(sessionId)
  if (!diag) return
  diag.readyMarks += 1
}

export function recordTerminalRender(sessionId: string, terminal: XTerm) {
  const diag = getSessionDiagnostics(sessionId)
  if (!diag) return

  diag.renderEvents += 1

  const activeBuffer = terminal.buffer.active
  const visibleLines = Math.min(terminal.rows, activeBuffer.length)
  let nonEmptyLines = 0
  const previewLines: string[] = []

  for (let lineIndex = 0; lineIndex < visibleLines; lineIndex += 1) {
    const line = activeBuffer.getLine(lineIndex)
    if (!line) continue
    const text = line.translateToString(true).trim()
    if (text.length > 0) {
      nonEmptyLines += 1
      if (previewLines.length < 6) {
        previewLines.push(text)
      }
    }
  }

  if (nonEmptyLines > 0) {
    diag.nonEmptyRenderEvents += 1
    if (nonEmptyLines > diag.maxVisibleNonEmptyLines) {
      diag.maxVisibleNonEmptyLines = nonEmptyLines
    }
    if (previewLines.length > 0) {
      diag.lastRenderPreview = previewLines.join(' | ').slice(0, PREVIEW_MAX)
    }
  }
}
