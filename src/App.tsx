import { getCurrentWindow } from "@tauri-apps/api/window";
import { useCallback, useEffect, useState } from "react";
import { BottomBar } from "@/components/BottomBar";
import { OutputPanel } from "@/components/OutputPanel";
import { ProjectSelector } from "@/components/ProjectSelector";
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from "@/components/ui/resizable";
import { Toaster } from "@/components/ui/sonner";
import { useInvoke } from "@/hooks/useInvoke";
import type { Page } from "@/hooks/useNavigation";
import { useTauriEvent } from "@/hooks/useTauriEvent";
import { DisciplinesPage } from "@/pages/DisciplinesPage";
import { FeaturesPage } from "@/pages/FeaturesPage";
import { TasksPage } from "@/pages/TasksPage";
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

type LoopStatus = ReturnType<typeof useLoopStore.getState>["status"];

function App() {
  const { setStatus, addOutput, setRateLimitInfo } = useLoopStore();
  const [lockedProject, setLockedProject] = useState<string | null>(null);
  const [currentPage, setCurrentPage] = useState<Page>("tasks");

  // Fetch locked project
  const { data: fetchedProject, isLoading: isLoadingProject } = useInvoke<string | null>("get_locked_project");

  // Sync fetched project to local state (allows onProjectSelected to update without refetch)
  useEffect(() => {
    if (fetchedProject !== undefined) {
      setLockedProject(fetchedProject);
    }
  }, [fetchedProject]);

  // Set window title when project changes
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

  // Bootstrap loop state into Zustand
  const { data: initialLoopState } = useInvoke<LoopStatus>("get_loop_state", undefined, {
    staleTime: Number.POSITIVE_INFINITY,
  });

  useEffect(() => {
    if (initialLoopState) {
      setStatus(initialLoopState);
    }
  }, [initialLoopState, setStatus]);

  // Event handlers
  const handleStateChanged = useCallback(
    (event: StateChangedEvent) => {
      const current = useLoopStore.getState().status;
      setStatus({
        ...current,
        state: event.state,
        current_iteration: event.iteration,
      });
      if (event.state !== "rate_limited") {
        setRateLimitInfo(null);
      }
    },
    [setStatus, setRateLimitInfo]
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
          } catch (err) {
            console.error("Failed to set window title:", err);
          }
        }}
      />
    );
  }

  return (
    <>
      <ResizablePanelGroup direction="horizontal" className="h-screen">
        {/* Left: Pages */}
        <ResizablePanel defaultSize={50} minSize={40}>
          <div className="h-full flex flex-col overflow-hidden">
            <div className="flex-1 min-h-0 overflow-hidden relative">
              {/* Preload all pages, show/hide with CSS */}
              <div className={currentPage === "tasks" ? "h-full" : "hidden"}>
                <TasksPage />
              </div>
              <div className={currentPage === "features" ? "h-full" : "hidden"}>
                <FeaturesPage />
              </div>
              <div className={currentPage === "disciplines" ? "h-full" : "hidden"}>
                <DisciplinesPage />
              </div>
            </div>
            {/* Bottom Bar */}
            <BottomBar lockedProject={lockedProject} currentPage={currentPage} onPageChange={setCurrentPage} />
          </div>
        </ResizablePanel>

        <ResizableHandle withHandle />

        {/* Right: Output (always visible) */}
        <ResizablePanel defaultSize={50} minSize={20}>
          <div className="h-full p-4">
            <OutputPanel />
          </div>
        </ResizablePanel>
      </ResizablePanelGroup>
      <Toaster />
    </>
  );
}

export default App;
