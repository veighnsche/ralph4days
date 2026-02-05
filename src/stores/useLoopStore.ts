import { create } from "zustand";

export type LoopState = "idle" | "running" | "paused" | "rate_limited" | "complete" | "aborted";

interface LoopStatus {
  state: LoopState;
  current_iteration: number;
  max_iterations: number;
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

interface LoopStore {
  status: LoopStatus;
  output: OutputLine[];
  rateLimitInfo: RateLimitInfo | null;
  projectPath: string;
  maxIterations: number;

  setStatus: (status: LoopStatus) => void;
  setProjectPath: (path: string) => void;
  setMaxIterations: (max: number) => void;
  addOutput: (text: string, type?: OutputLine["type"]) => void;
  clearOutput: () => void;
  setRateLimitInfo: (info: RateLimitInfo | null) => void;
  reset: () => void;
}

let outputIdCounter = 0;

const initialStatus: LoopStatus = {
  state: "idle",
  current_iteration: 0,
  max_iterations: 0,
  stagnant_count: 0,
  rate_limit_retries: 0,
  last_progress_hash: null,
  project_path: null,
};

export const useLoopStore = create<LoopStore>((set) => ({
  status: initialStatus,
  output: [],
  rateLimitInfo: null,
  projectPath: "",
  maxIterations: 100,

  setStatus: (status) => set({ status }),

  setProjectPath: (path) => set({ projectPath: path }),

  setMaxIterations: (max) => set({ maxIterations: max }),

  addOutput: (text, type = "output") =>
    set((state) => ({
      output: [
        ...state.output,
        {
          id: ++outputIdCounter,
          text,
          timestamp: new Date(),
          type,
        },
      ].slice(-5000), // Keep last 5000 lines
    })),

  clearOutput: () => set({ output: [] }),

  setRateLimitInfo: (info) => set({ rateLimitInfo: info }),

  reset: () =>
    set({
      status: initialStatus,
      output: [],
      rateLimitInfo: null,
    }),
}));
