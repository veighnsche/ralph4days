import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Terminal as XTerm } from "@xterm/xterm";
import { TerminalSquare } from "lucide-react";
import { useCallback, useEffect, useRef } from "react";
import { Terminal } from "@/components/ui/terminal";
import { useTabMeta } from "@/hooks/useTabMeta";

// NOTE TO FUTURE DEVELOPERS (and Vince):
// Claude Code's welcome screen renders LEFT-ALIGNED in PTY environments. This is NOT a Ralph bug.
// Claude Code uses React + Ink (Yoga layout) which defensively falls back to left-alignment when
// terminal width detection is uncertain — standard behavior in embedded PTY terminals.
// See: https://github.com/anthropics/claude-code/issues/5430
// DO NOT waste time trying to "fix" centering. It's upstream. Move on to features that matter.

import type { WorkspaceTab } from "@/stores/useWorkspaceStore";

export function TerminalTabContent({ tab }: { tab: WorkspaceTab }) {
  const sessionId = tab.id;
  const model = tab.data?.model;
  const thinking = tab.data?.thinking;

  useTabMeta(tab.id, tab.title, TerminalSquare);
  const terminalRef = useRef<XTerm | null>(null);
  // Buffer output that arrives before xterm is ready
  const pendingOutputRef = useRef<Uint8Array[]>([]);
  const terminalReadyRef = useRef(false);
  const sessionStartedRef = useRef(false);

  // Create PTY session on mount, terminate on unmount
  useEffect(() => {
    invoke("create_pty_session", {
      sessionId,
      mcpMode: "interactive",
      model: model || null,
      thinking: thinking ?? null,
    }).catch((err) => {
      console.error("Failed to create PTY session:", err);
    });

    return () => {
      invoke("terminate_pty_session", { sessionId }).catch(() => {});
    };
  }, [sessionId, model, thinking]);

  // Listen for PTY output
  useEffect(() => {
    const unlisten = listen<{ session_id: string; data: number[] }>("ralph://pty_output", (event) => {
      if (event.payload.session_id !== sessionId) return;

      // Mark session as started when we receive first output
      sessionStartedRef.current = true;

      const bytes = new Uint8Array(event.payload.data);
      if (terminalReadyRef.current && terminalRef.current) {
        terminalRef.current.write(bytes);
      } else {
        // Buffer until terminal is ready
        pendingOutputRef.current.push(bytes);
      }
    });

    return () => {
      unlisten.then((unsub) => unsub());
    };
  }, [sessionId]);

  // Listen for PTY close
  useEffect(() => {
    const unlisten = listen<{ session_id: string; exit_code: number }>("ralph://pty_closed", (event) => {
      if (
        event.payload.session_id === sessionId &&
        terminalRef.current &&
        sessionStartedRef.current // Only show if session actually started
      ) {
        terminalRef.current.writeln("\r\n\x1b[2m[Session ended]\x1b[0m");
      }
    });

    return () => {
      unlisten.then((unsub) => unsub());
    };
  }, [sessionId]);

  const handleReady = useCallback(
    (terminal: XTerm) => {
      terminalRef.current = terminal;
      terminalReadyRef.current = true;

      // Use requestAnimationFrame to ensure FitAddon has fully completed
      requestAnimationFrame(() => {
        // Resize PTY to match actual terminal dimensions
        invoke("resize_pty", {
          sessionId,
          cols: terminal.cols,
          rows: terminal.rows,
        }).catch(() => {});
      });

      // Flush any buffered output
      for (const chunk of pendingOutputRef.current) {
        terminal.write(chunk);
      }
      pendingOutputRef.current = [];

      // Forward keyboard input to PTY
      terminal.onData((data) => {
        const bytes = Array.from(new TextEncoder().encode(data));
        invoke("send_terminal_input", { sessionId, data: bytes }).catch(() => {});
      });
    },
    [sessionId]
  );

  const handleResize = useCallback(
    ({ cols, rows }: { cols: number; rows: number }) => {
      invoke("resize_pty", { sessionId, cols, rows }).catch(() => {});
    },
    [sessionId]
  );

  return (
    <div className="flex-1 flex flex-col min-h-0 h-full">
      <div className="flex-1 min-h-0">
        <Terminal onReady={handleReady} interactive={true} onResize={handleResize} />
      </div>
    </div>
  );
}
