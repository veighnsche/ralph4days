import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useCallback, useEffect, useState } from "react";
import { BottomBar } from "@/components/BottomBar";
import { OutputPanel } from "@/components/OutputPanel";
import { ProjectSelector } from "@/components/ProjectSelector";
import { PRDViewer } from "@/components/prd/PRDViewer";
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from "@/components/ui/resizable";
import { useTauriEvent } from "@/hooks/useTauriEvent";
import { type LoopState, useLoopStore } from "@/stores/useLoopStore";
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
    if (typeof window !== "undefined" && "__TAURI__" in window) {
      invoke<string | null>("get_locked_project")
        .then(async (project) => {
          setLockedProject(project);
          if (project) {
            const projectName = project.split("/").pop() || "Unknown";
            try {
              await getCurrentWindow().setTitle(`Ralph4days - ${projectName}`);
              console.log("Window title set to:", `Ralph4days - ${projectName}`);
            } catch (err) {
              console.error("Failed to set window title:", err);
            }
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
    if (typeof window !== "undefined" && "__TAURI__" in window) {
      invoke<typeof status>("get_loop_state").then(setStatus).catch(console.error);
    }
  }, [setStatus]);

  // Update window title when project changes
  useEffect(() => {
    if (lockedProject && typeof window !== "undefined" && "__TAURI__" in window) {
      const projectName = lockedProject.split("/").pop() || "Unknown";
      getCurrentWindow()
        .setTitle(`Ralph4days - ${projectName}`)
        .catch((err) => {
          console.error("Failed to set window title:", err);
        });
    }
  }, [lockedProject]);

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
    return (
      <ProjectSelector
        onProjectSelected={async (project) => {
          setLockedProject(project);
          const projectName = project.split("/").pop() || "Unknown";
          try {
            await getCurrentWindow().setTitle(`Ralph4days - ${projectName}`);
            console.log("Window title set to:", `Ralph4days - ${projectName}`);
          } catch (err) {
            console.error("Failed to set window title:", err);
          }
        }}
      />
    );
  }

  return (
    <ResizablePanelGroup direction="horizontal" className="h-screen">
      {/* Left: PRD */}
      <ResizablePanel defaultSize={66} minSize={40}>
        <div className="h-full flex flex-col overflow-hidden">
          <div className="flex-1 min-h-0 overflow-hidden">
            <PRDViewer />
          </div>
          {/* Bottom Bar (PRD only) */}
          <BottomBar lockedProject={lockedProject} />
        </div>
      </ResizablePanel>

      <ResizableHandle withHandle />

      {/* Right: Output */}
      <ResizablePanel defaultSize={34} minSize={20}>
        <div className="h-full p-4">
          <OutputPanel />
        </div>
      </ResizablePanel>
    </ResizablePanelGroup>
  );
}

export default App;
