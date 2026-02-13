import { FitAddon } from '@xterm/addon-fit'
import { WebLinksAddon } from '@xterm/addon-web-links'
import { Terminal as XTerm } from '@xterm/xterm'
import { useEffect, useRef } from 'react'
import '@xterm/xterm/css/xterm.css'
import { TERMINAL_BG } from '@/constants/terminal'
import { WORKSPACE_SELECTORS } from '@/test/selectors'

interface TerminalProps {
  onReady?: (terminal: XTerm) => void
  onResize?: (dims: { cols: number; rows: number }) => void
}

export function Terminal({ onReady, onResize }: TerminalProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const terminalRef = useRef<XTerm | null>(null)
  const fitAddonRef = useRef<FitAddon | null>(null)
  const onReadyRef = useRef(onReady)
  const onResizeRef = useRef(onResize)
  onReadyRef.current = onReady
  onResizeRef.current = onResize

  useEffect(() => {
    if (!containerRef.current || terminalRef.current) return

    const terminal = new XTerm({
      cursorBlink: true,
      cursorStyle: 'block',
      fontSize: 13,
      fontFamily: "'GeistMono NF', monospace",
      lineHeight: 1.0,
      scrollback: 10000,
      convertEol: true,
      theme: {
        background: TERMINAL_BG,
        foreground: '#e0e0e0',
        cursor: '#e0e0e0',
        cursorAccent: TERMINAL_BG,
        selectionBackground: '#3a3a3a',
        selectionForeground: '#ffffff',
        black: '#1a1a1a',
        red: '#f87171',
        green: '#4ade80',
        yellow: '#facc15',
        blue: '#60a5fa',
        magenta: '#c084fc',
        cyan: '#22d3ee',
        white: '#e0e0e0',
        brightBlack: '#4a4a4a',
        brightRed: '#fca5a5',
        brightGreen: '#86efac',
        brightYellow: '#fde047',
        brightBlue: '#93c5fd',
        brightMagenta: '#d8b4fe',
        brightCyan: '#67e8f9',
        brightWhite: '#ffffff'
      }
    })

    const fitAddon = new FitAddon()
    terminal.loadAddon(fitAddon)
    terminal.loadAddon(new WebLinksAddon())

    terminal.open(containerRef.current)
    fitAddon.fit()

    terminalRef.current = terminal
    fitAddonRef.current = fitAddon

    onReadyRef.current?.(terminal)

    terminal.onResize(({ cols, rows }) => {
      onResizeRef.current?.({ cols, rows })
    })

    const resizeObserver = new ResizeObserver(() => {
      requestAnimationFrame(() => {
        fitAddonRef.current?.fit()
      })
    })
    resizeObserver.observe(containerRef.current)

    return () => {
      resizeObserver.disconnect()
      terminal.dispose()
      terminalRef.current = null
      fitAddonRef.current = null
    }
  }, [])

  return (
    <div
      ref={containerRef}
      data-testid={WORKSPACE_SELECTORS.terminalHost}
      className="h-full w-full overflow-hidden ml-[11px]"
      style={{ backgroundColor: TERMINAL_BG }}
    />
  )
}
