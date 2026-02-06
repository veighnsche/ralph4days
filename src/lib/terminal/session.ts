import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import type { TerminalSessionConfig, TerminalSessionHandlers } from "./types";

/**
 * Hook to manage a PTY terminal session with Claude Code.
 * Handles session lifecycle, output buffering, and input forwarding.
 */
export function useTerminalSession(config: TerminalSessionConfig, handlers: TerminalSessionHandlers) {
  const [isReady, setIsReady] = useState(false);
  const [isConnected, setIsConnected] = useState(false);
  const outputBufferRef = useRef<Uint8Array[]>([]);
  const sessionStartedRef = useRef(false);

  // Create PTY session on mount, terminate on unmount
  useEffect(() => {
    invoke("create_pty_session", {
      sessionId: config.sessionId,
      mcpMode: config.mcpMode || "interactive",
      model: config.model || null,
      thinking: config.thinking ?? null,
    })
      .then(() => setIsConnected(true))
      .catch((err) => handlers.onError?.(String(err)));

    return () => {
      invoke("terminate_pty_session", { sessionId: config.sessionId }).catch(() => {});
    };
  }, [config.sessionId, config.mcpMode, config.model, config.thinking, handlers]);

  // Listen for PTY output
  useEffect(() => {
    const unlisten = listen<{ session_id: string; data: number[] }>("ralph://pty_output", (event) => {
      if (event.payload.session_id !== config.sessionId) return;
      sessionStartedRef.current = true;

      const bytes = new Uint8Array(event.payload.data);
      if (isReady) {
        handlers.onOutput?.(bytes);
      } else {
        outputBufferRef.current.push(bytes);
      }
    });

    return () => {
      unlisten.then((unsub) => unsub());
    };
  }, [config.sessionId, isReady, handlers]);

  // Listen for PTY close
  useEffect(() => {
    const unlisten = listen<{ session_id: string; exit_code: number }>("ralph://pty_closed", (event) => {
      if (event.payload.session_id === config.sessionId && sessionStartedRef.current) {
        handlers.onClosed?.(event.payload.exit_code);
      }
    });

    return () => {
      unlisten.then((unsub) => unsub());
    };
  }, [config.sessionId, handlers]);

  // Mark terminal ready and flush buffered output
  const markReady = useCallback(() => {
    setIsReady(true);
    for (const chunk of outputBufferRef.current) {
      handlers.onOutput?.(chunk);
    }
    outputBufferRef.current = [];
  }, [handlers]);

  // Send input to PTY
  const sendInput = useCallback(
    (data: string) => {
      const bytes = Array.from(new TextEncoder().encode(data));
      invoke("send_terminal_input", {
        sessionId: config.sessionId,
        data: bytes,
      }).catch((err) => handlers.onError?.(String(err)));
    },
    [config.sessionId, handlers]
  );

  // Resize PTY
  const resize = useCallback(
    (cols: number, rows: number) => {
      invoke("resize_pty", {
        sessionId: config.sessionId,
        cols,
        rows,
      }).catch(() => {});
    },
    [config.sessionId]
  );

  return {
    isConnected,
    isReady,
    markReady,
    sendInput,
    resize,
  };
}
