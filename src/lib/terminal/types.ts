export interface TerminalSessionConfig {
  sessionId: string;
  mcpMode?: string;
  model?: string | null;
  thinking?: boolean | null;
}

export interface TerminalSessionHandlers {
  onOutput?: (data: Uint8Array) => void;
  onClosed?: (exitCode: number) => void;
  onError?: (error: string) => void;
}
