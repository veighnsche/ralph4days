// TODO: DEPRECATED ITERATION LOGIC
// - Remove max_iterations and current_iteration from LoopStatus interface
// - current_iteration can stay for display purposes only
// - Replace maxIterations in store with loopEnabled: boolean
// - setMaxIterations -> setLoopEnabled(boolean)

import { create } from "zustand";

export type LoopState = "idle" | "running" | "paused" | "rate_limited" | "complete" | "aborted";

interface LoopStatus {
  state: LoopState;
  current_iteration: number; // TODO: Keep for display only, not loop control
  max_iterations: number; // TODO: Remove, use loopEnabled in config instead
  stagnant_count: number;
  rate_limit_retries: number;
  last_progress_hash: string | null;
  project_path: string | null;
}

interface OutputLine {
  id: number;
  text: string;
  timestamp: Date;
  type: "output" | "error" | "info" | "success";
}

interface RateLimitInfo {
  retryInSecs: number;
  attempt: number;
  maxAttempts: number;
  startTime: Date;
}

interface OutputSession {
  id: string;
  output: OutputLine[];
  rateLimitInfo: RateLimitInfo | null;
  createdAt: Date;
}

interface LoopStore {
  status: LoopStatus;
  sessions: Map<string, OutputSession>;
  activeSessionId: string | null;
  projectPath: string;
  maxIterations: number; // TODO: Replace with loopEnabled: boolean

  setStatus: (status: LoopStatus) => void;
  setProjectPath: (path: string) => void;
  setMaxIterations: (max: number) => void; // TODO: Replace with setLoopEnabled: (enabled: boolean) => void
  createSession: () => string;
  setActiveSession: (sessionId: string) => void;
  addOutput: (text: string, type?: OutputLine["type"], sessionId?: string) => void;
  clearSession: (sessionId: string) => void;
  setRateLimitInfo: (info: RateLimitInfo | null, sessionId?: string) => void;
  getSession: (sessionId: string) => OutputSession | undefined;
  reset: () => void;
}

let outputIdCounter = 0;

const initialStatus: LoopStatus = {
  state: "idle",
  current_iteration: 0, // TODO: Keep for display only
  max_iterations: 0, // TODO: Remove
  stagnant_count: 0,
  rate_limit_retries: 0,
  last_progress_hash: null,
  project_path: null,
};

export const useLoopStore = create<LoopStore>((set, get) => ({
  status: initialStatus,
  sessions: new Map(),
  activeSessionId: null,
  projectPath: "",
  maxIterations: 1, // TODO: Replace with loopEnabled: false

  setStatus: (status) => set({ status }),

  setProjectPath: (path) => set({ projectPath: path }),

  // TODO: Replace with setLoopEnabled: (enabled) => set({ loopEnabled: enabled })
  setMaxIterations: (max) => set({ maxIterations: max }),

  createSession: () => {
    const sessionId = `session-${Date.now()}`;
    const session: OutputSession = {
      id: sessionId,
      output: [],
      rateLimitInfo: null,
      createdAt: new Date(),
    };
    set((state) => {
      const newSessions = new Map(state.sessions);
      newSessions.set(sessionId, session);
      return { sessions: newSessions, activeSessionId: sessionId };
    });
    return sessionId;
  },

  setActiveSession: (sessionId) => set({ activeSessionId: sessionId }),

  addOutput: (text, type = "output", sessionId) => {
    const targetSessionId = sessionId ?? get().activeSessionId;
    if (!targetSessionId) return;

    set((state) => {
      const newSessions = new Map(state.sessions);
      const session = newSessions.get(targetSessionId);
      if (!session) return state;

      const updatedSession: OutputSession = {
        ...session,
        output: [
          ...session.output,
          {
            id: ++outputIdCounter,
            text,
            timestamp: new Date(),
            type,
          },
        ].slice(-5000), // Keep last 5000 lines
      };
      newSessions.set(targetSessionId, updatedSession);
      return { sessions: newSessions };
    });
  },

  clearSession: (sessionId) =>
    set((state) => {
      const newSessions = new Map(state.sessions);
      const session = newSessions.get(sessionId);
      if (!session) return state;

      newSessions.set(sessionId, { ...session, output: [], rateLimitInfo: null });
      return { sessions: newSessions };
    }),

  setRateLimitInfo: (info, sessionId) => {
    const targetSessionId = sessionId ?? get().activeSessionId;
    if (!targetSessionId) return;

    set((state) => {
      const newSessions = new Map(state.sessions);
      const session = newSessions.get(targetSessionId);
      if (!session) return state;

      newSessions.set(targetSessionId, { ...session, rateLimitInfo: info });
      return { sessions: newSessions };
    });
  },

  getSession: (sessionId) => get().sessions.get(sessionId),

  reset: () =>
    set({
      status: initialStatus,
      sessions: new Map(),
      activeSessionId: null,
    }),
}));
