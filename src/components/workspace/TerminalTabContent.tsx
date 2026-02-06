import type { Terminal as XTerm } from "@xterm/xterm";
import { TerminalSquare } from "lucide-react";
import { useCallback, useRef } from "react";
import { useTabMeta } from "@/hooks/useTabMeta";
import { Terminal, useTerminalSession } from "@/lib/terminal";
import type { WorkspaceTab } from "@/stores/useWorkspaceStore";

// NOTE TO FUTURE DEVELOPERS (and Vince):
// Claude Code's welcome screen renders LEFT-ALIGNED in PTY environments. This is NOT a Ralph bug.
// Claude Code uses React + Ink (Yoga layout) which defensively falls back to left-alignment when
// terminal width detection is uncertain — standard behavior in embedded PTY terminals.
// See: https://github.com/anthropics/claude-code/issues/5430
// DO NOT waste time trying to "fix" centering. It's upstream. Move on to features that matter.

export function TerminalTabContent({ tab }: { tab: WorkspaceTab }) {
  useTabMeta(tab.id, tab.title, TerminalSquare);
  const terminalRef = useRef<XTerm | null>(null);

  const session = useTerminalSession(
    {
      sessionId: tab.id,
      mcpMode: "interactive",
      model: tab.data?.model,
      thinking: tab.data?.thinking,
    },
    {
      onOutput: (data) => terminalRef.current?.write(data),
      onClosed: () => {
        terminalRef.current?.writeln("\r\n\x1b[2m[Session ended]\x1b[0m");
      },
      onError: (err) => console.error("Terminal error:", err),
    }
  );

  const handleReady = useCallback(
    (terminal: XTerm) => {
      terminalRef.current = terminal;

      // Resize PTY to match terminal dimensions after fit
      requestAnimationFrame(() => {
        session.resize(terminal.cols, terminal.rows);
      });

      // Mark session ready (flushes buffered output)
      session.markReady();

      // Forward keyboard input to PTY
      terminal.onData((data) => session.sendInput(data));
    },
    [session]
  );

  const handleResize = useCallback(
    ({ cols, rows }: { cols: number; rows: number }) => {
      session.resize(cols, rows);
    },
    [session]
  );

  return (
    <div className="flex-1 flex flex-col min-h-0 h-full">
      <div className="flex-1 min-h-0">
        <Terminal onReady={handleReady} interactive={true} onResize={handleResize} />
      </div>
    </div>
  );
}
