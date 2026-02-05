import { useEffect, useRef } from "react";
import { Terminal, useTerminal } from "./Terminal";
import { useLoopStore } from "@/stores/useLoopStore";
import type { Terminal as XTerm } from "@xterm/xterm";

export function OutputPanel() {
  const { output, rateLimitInfo } = useLoopStore();
  const { setTerminal, writeln, clear } = useTerminal();
  const lastOutputLengthRef = useRef(0);
  const terminalReadyRef = useRef(false);

  // Handle terminal ready
  const handleTerminalReady = (terminal: XTerm) => {
    setTerminal(terminal);
    terminalReadyRef.current = true;

    // Write any existing output that accumulated before terminal was ready
    if (output.length > 0) {
      output.forEach((line) => {
        writeLineToTerminal(terminal, line);
      });
      lastOutputLengthRef.current = output.length;
    }
  };

  // Write a line with appropriate ANSI coloring
  const writeLineToTerminal = (
    terminal: XTerm,
    line: { text: string; timestamp: Date; type: "output" | "error" | "info" | "success" }
  ) => {
    const time = formatTime(line.timestamp);
    const colorCode = getColorCode(line.type);
    const resetCode = "\x1b[0m";
    const dimCode = "\x1b[2m";

    terminal.writeln(`${dimCode}[${time}]${resetCode} ${colorCode}${line.text}${resetCode}`);
  };

  // Write new output lines to terminal
  useEffect(() => {
    if (!terminalReadyRef.current) return;

    // Only write new lines
    const newLines = output.slice(lastOutputLengthRef.current);
    newLines.forEach((line) => {
      const time = formatTime(line.timestamp);
      const colorCode = getColorCode(line.type);
      const resetCode = "\x1b[0m";
      const dimCode = "\x1b[2m";

      writeln(`${dimCode}[${time}]${resetCode} ${colorCode}${line.text}${resetCode}`);
    });

    lastOutputLengthRef.current = output.length;
  }, [output, writeln]);

  // Clear terminal when output is cleared
  useEffect(() => {
    if (output.length === 0 && lastOutputLengthRef.current > 0) {
      clear();
      lastOutputLengthRef.current = 0;
    }
  }, [output.length, clear]);

  return (
    <div className="flex h-full flex-col">
      {rateLimitInfo && (
        <div className="mb-2 bg-yellow-600/20 border border-yellow-600/50 p-3">
          <div className="flex items-center gap-2">
            <span className="text-yellow-500 font-medium">Rate Limited</span>
            <span className="text-sm text-[hsl(var(--muted-foreground))]">
              Retry attempt {rateLimitInfo.attempt} of {rateLimitInfo.maxAttempts}
            </span>
          </div>
          <RateLimitCountdown info={rateLimitInfo} />
        </div>
      )}

      <div className="flex-1 min-h-0">
        <Terminal onReady={handleTerminalReady} />
      </div>
    </div>
  );
}

function RateLimitCountdown({ info }: { info: { retryInSecs: number; startTime: Date } }) {
  const elapsed = Math.floor((Date.now() - info.startTime.getTime()) / 1000);
  const remaining = Math.max(0, info.retryInSecs - elapsed);

  const minutes = Math.floor(remaining / 60);
  const seconds = remaining % 60;

  return (
    <div className="text-sm mt-1">
      Retrying in:{" "}
      <span className="font-mono text-yellow-400">
        {minutes}:{seconds.toString().padStart(2, "0")}
      </span>
    </div>
  );
}

function formatTime(date: Date) {
  return date.toLocaleTimeString("en-US", {
    hour12: false,
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

function getColorCode(type: "output" | "error" | "info" | "success"): string {
  switch (type) {
    case "error":
      return "\x1b[31m"; // Red
    case "info":
      return "\x1b[34m"; // Blue
    case "success":
      return "\x1b[32m"; // Green
    case "output":
    default:
      return ""; // Default color
  }
}
