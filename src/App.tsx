import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarInset,
  SidebarProvider,
  SidebarTrigger,
} from "@/components/ui/sidebar";
import { Separator } from "@/components/ui/separator";
import { StatusBadge } from "@/components/StatusBadge";
import { LoopControls } from "@/components/LoopControls";
import { OutputPanel } from "@/components/OutputPanel";
import { ProjectSelector } from "@/components/ProjectSelector";
import { Settings } from "@/components/Settings";
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
        .then(async (project) => {
          setLockedProject(project);
          if (project) {
            const projectName = project.split('/').pop() || 'Unknown';
            try {
              await getCurrentWindow().setTitle(`Ralph4days - ${projectName}`);
              console.log('Window title set to:', `Ralph4days - ${projectName}`);
            } catch (err) {
              console.error('Failed to set window title:', err);
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
    if (typeof window !== 'undefined' && '__TAURI__' in window) {
      invoke<typeof status>("get_loop_state").then(setStatus).catch(console.error);
    }
  }, [setStatus]);

  // Update window title when project changes
  useEffect(() => {
    if (lockedProject && typeof window !== 'undefined' && '__TAURI__' in window) {
      const projectName = lockedProject.split('/').pop() || 'Unknown';
      getCurrentWindow().setTitle(`Ralph4days - ${projectName}`).catch(err => {
        console.error('Failed to set window title:', err);
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
    return <ProjectSelector onProjectSelected={async (project) => {
      setLockedProject(project);
      const projectName = project.split('/').pop() || 'Unknown';
      try {
        await getCurrentWindow().setTitle(`Ralph4days - ${projectName}`);
        console.log('Window title set to:', `Ralph4days - ${projectName}`);
      } catch (err) {
        console.error('Failed to set window title:', err);
      }
    }} />;
  }

  const projectName = lockedProject.split('/').pop() || 'Unknown';

  return (
    <SidebarProvider>
      <Sidebar>
        <SidebarHeader>
          <div className="flex items-center justify-between px-4 py-2">
            <div className="font-semibold">{projectName}</div>
            <StatusBadge state={status.state} />
          </div>
          {status.state !== "idle" && (
            <div className="px-4 text-sm text-[hsl(var(--muted-foreground))]">
              Iteration {status.current_iteration} / {status.max_iterations}
            </div>
          )}
        </SidebarHeader>
        <Separator />
        <SidebarContent>
          <div className="p-4">
            <LoopControls lockedProject={lockedProject} />
          </div>
        </SidebarContent>
        <SidebarFooter>
          <div className="p-4">
            <Settings />
          </div>
        </SidebarFooter>
      </Sidebar>

      <SidebarInset>
        <header className="flex h-16 shrink-0 items-center gap-2 border-b px-4">
          <SidebarTrigger className="-ml-1" />
          <Separator orientation="vertical" className="mr-2 h-4" />
          <h1 className="font-semibold">Output</h1>
        </header>
        <div className="flex flex-1 flex-col gap-4 p-4">
          <OutputPanel />
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}

export default App;
