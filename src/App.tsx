import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { StatusBadge } from "@/components/StatusBadge";
import { LoopControls } from "@/components/LoopControls";
import { OutputPanel } from "@/components/OutputPanel";
import { ProjectSelector } from "@/components/ProjectSelector";
import { ThemeToggle } from "@/components/theme-toggle";
import { useLoopStore, LoopState } from "@/stores/useLoopStore";
import { useTauriEvent } from "@/hooks/useTauriEvent";
import "./index.css";

interface StateChangedEvent {
  event: "state_changed";
  state: LoopState;
  iteration: number;
}

interface OutputChunkEvent {
  event: "output_chunk";
  text: string;
}

interface IterationCompleteEvent {
  event: "iteration_complete";
  iteration: number;
  success: boolean;
  message: string | null;
}

interface RateLimitedEvent {
  event: "rate_limited";
  retry_in_secs: number;
  attempt: number;
  max_attempts: number;
}

interface ErrorEvent {
  event: "error";
  message: string;
}

function App() {
  const { status, setStatus, addOutput, setRateLimitInfo } = useLoopStore();
  const [lockedProject, setLockedProject] = useState<string | null>(null);
  const [isLoadingProject, setIsLoadingProject] = useState(true);

  // Check for locked project on mount and set window title
  useEffect(() => {
    if (typeof window !== 'undefined' && '__TAURI__' in window) {
      invoke<string | null>("get_locked_project")
        .then((project) => {
          setLockedProject(project);
          if (project) {
            const projectName = project.split('/').pop() || 'Unknown';
            getCurrentWindow().setTitle(`Ralph4days - ${projectName}`);
          }
          setIsLoadingProject(false);
        })
        .catch((err) => {
          console.error("Failed to get locked project:", err);
          setIsLoadingProject(false);
        });
    } else {
      setIsLoadingProject(false);
    }
  }, []);

  // Poll for initial state
  useEffect(() => {
    if (typeof window !== 'undefined' && '__TAURI__' in window) {
      invoke<typeof status>("get_loop_state").then(setStatus).catch(console.error);
    }
  }, [setStatus]);

  // Event handlers
  const handleStateChanged = useCallback(
    (event: StateChangedEvent) => {
      setStatus({
        ...status,
        state: event.state,
        current_iteration: event.iteration,
      });
      if (event.state !== "rate_limited") {
        setRateLimitInfo(null);
      }
    },
    [status, setStatus, setRateLimitInfo]
  );

  const handleOutputChunk = useCallback(
    (event: OutputChunkEvent) => {
      if (event.text.trim()) {
        addOutput(event.text);
      }
    },
    [addOutput]
  );

  const handleIterationComplete = useCallback(
    (event: IterationCompleteEvent) => {
      const msg = event.success
        ? `Iteration ${event.iteration} complete`
        : `Iteration ${event.iteration} failed: ${event.message || "Unknown error"}`;
      addOutput(msg, event.success ? "success" : "error");
    },
    [addOutput]
  );

  const handleRateLimited = useCallback(
    (event: RateLimitedEvent) => {
      setRateLimitInfo({
        retryInSecs: event.retry_in_secs,
        attempt: event.attempt,
        maxAttempts: event.max_attempts,
        startTime: new Date(),
      });
      addOutput(
        `Rate limited. Waiting ${event.retry_in_secs}s before retry (attempt ${event.attempt}/${event.max_attempts})`,
        "info"
      );
    },
    [setRateLimitInfo, addOutput]
  );

  const handleError = useCallback(
    (event: ErrorEvent) => {
      addOutput(event.message, "error");
    },
    [addOutput]
  );

  // Subscribe to Tauri events
  useTauriEvent("ralph://state_changed", handleStateChanged);
  useTauriEvent("ralph://output_chunk", handleOutputChunk);
  useTauriEvent("ralph://iteration_complete", handleIterationComplete);
  useTauriEvent("ralph://rate_limited", handleRateLimited);
  useTauriEvent("ralph://error", handleError);

  // Show loading or project picker
  if (isLoadingProject) {
    return (
      <div className="flex h-screen items-center justify-center">
        <div className="text-[hsl(var(--muted-foreground))]">Loading...</div>
      </div>
    );
  }

  if (!lockedProject) {
    return <ProjectSelector onProjectSelected={(project) => {
      setLockedProject(project);
      const projectName = project.split('/').pop() || 'Unknown';
      getCurrentWindow().setTitle(`Ralph4days - ${projectName}`);
    }} />;
  }

  const projectName = lockedProject.split('/').pop() || 'Unknown';

  return (
    <div className="flex h-screen gap-4 p-4">
      {/* Left Panel - Controls */}
      <Card className="w-80 shrink-0">
        <CardHeader className="pb-4">
          <div className="flex items-center justify-between">
            <CardTitle className="text-lg">{projectName}</CardTitle>
            <div className="flex items-center gap-2">
              <ThemeToggle />
              <StatusBadge state={status.state} />
            </div>
          </div>
          {status.state !== "idle" && (
            <div className="text-sm text-[hsl(var(--muted-foreground))]">
              Iteration {status.current_iteration} / {status.max_iterations}
            </div>
          )}
        </CardHeader>
        <CardContent>
          <LoopControls lockedProject={lockedProject} />
        </CardContent>
      </Card>

      {/* Right Panel - Output */}
      <Card className="flex-1 flex flex-col min-w-0">
        <CardHeader className="pb-2">
          <CardTitle className="text-lg">Output</CardTitle>
        </CardHeader>
        <CardContent className="flex-1 pb-4 min-h-0">
          <OutputPanel />
        </CardContent>
      </Card>
    </div>
  );
}

export default App;
