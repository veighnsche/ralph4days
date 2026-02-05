import { FitAddon } from "@xterm/addon-fit";
import { WebLinksAddon } from "@xterm/addon-web-links";
import { Terminal as XTerm } from "@xterm/xterm";
import { useCallback, useEffect, useRef } from "react";
import "@xterm/xterm/css/xterm.css";

interface TerminalProps {
  /** Called when terminal is ready to receive input */
  onReady?: (terminal: XTerm) => void;
  /** Font size in pixels */
  fontSize?: number;
  /** Font family */
  fontFamily?: string;
}

export function Terminal({
  onReady,
  fontSize = 13,
  fontFamily = "'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace",
}: TerminalProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const terminalRef = useRef<XTerm | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);

  useEffect(() => {
    if (!containerRef.current || terminalRef.current) return;

    // Create terminal instance
    const terminal = new XTerm({
      cursorBlink: false,
      cursorStyle: "block",
      disableStdin: true, // Read-only terminal
      fontSize,
      fontFamily,
      lineHeight: 1.2,
      scrollback: 10000,
      convertEol: true,
      theme: {
        background: "#0a0a0a",
        foreground: "#e0e0e0",
        cursor: "#e0e0e0",
        cursorAccent: "#0a0a0a",
        selectionBackground: "#3a3a3a",
        selectionForeground: "#ffffff",
        // Standard ANSI colors
        black: "#1a1a1a",
        red: "#f87171",
        green: "#4ade80",
        yellow: "#facc15",
        blue: "#60a5fa",
        magenta: "#c084fc",
        cyan: "#22d3ee",
        white: "#e0e0e0",
        // Bright variants
        brightBlack: "#4a4a4a",
        brightRed: "#fca5a5",
        brightGreen: "#86efac",
        brightYellow: "#fde047",
        brightBlue: "#93c5fd",
        brightMagenta: "#d8b4fe",
        brightCyan: "#67e8f9",
        brightWhite: "#ffffff",
      },
    });

    // Load addons
    const fitAddon = new FitAddon();
    const webLinksAddon = new WebLinksAddon();

    terminal.loadAddon(fitAddon);
    terminal.loadAddon(webLinksAddon);

    // Open terminal in container
    terminal.open(containerRef.current);

    // Fit to container
    fitAddon.fit();

    // Store refs
    terminalRef.current = terminal;
    fitAddonRef.current = fitAddon;

    // Notify parent
    onReady?.(terminal);

    // Handle resize
    const resizeObserver = new ResizeObserver(() => {
      // Use requestAnimationFrame to debounce resize events
      requestAnimationFrame(() => {
        if (fitAddonRef.current && containerRef.current) {
          fitAddonRef.current.fit();
        }
      });
    });
    resizeObserver.observe(containerRef.current);

    return () => {
      resizeObserver.disconnect();
      terminal.dispose();
      terminalRef.current = null;
      fitAddonRef.current = null;
    };
  }, [fontSize, fontFamily, onReady]);

  return (
    <div
      ref={containerRef}
      className="h-full w-full overflow-hidden border border-[hsl(var(--border))]"
      style={{ backgroundColor: "#0a0a0a" }}
    />
  );
}

/**
 * Hook to manage terminal output
 */
export function useTerminal() {
  const terminalRef = useRef<XTerm | null>(null);

  const setTerminal = useCallback((terminal: XTerm) => {
    terminalRef.current = terminal;
  }, []);

  const write = useCallback((text: string) => {
    terminalRef.current?.write(text);
  }, []);

  const writeln = useCallback((text: string) => {
    terminalRef.current?.writeln(text);
  }, []);

  const clear = useCallback(() => {
    terminalRef.current?.clear();
  }, []);

  const scrollToBottom = useCallback(() => {
    terminalRef.current?.scrollToBottom();
  }, []);

  return {
    setTerminal,
    write,
    writeln,
    clear,
    scrollToBottom,
    terminal: terminalRef,
  };
}
