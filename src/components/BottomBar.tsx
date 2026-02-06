// TODO: DEPRECATED ITERATION LOGIC
// - Replace maxIterations/setMaxIterations with loopEnabled/setLoopEnabled
// - start_loop should take loopEnabled: bool instead of maxIterations: number
// - Remove all references to maxIterations in this component

import { invoke } from "@tauri-apps/api/core";
import { Pause, Play, Square } from "lucide-react";
import { LoopToggle } from "@/components/LoopToggle";
import { NavigationMenu } from "@/components/NavigationMenu";
import { Settings } from "@/components/Settings";
import { Button } from "@/components/ui/button";
import type { Page } from "@/hooks/useNavigation";
import { useLoopStore } from "@/stores/useLoopStore";
import { useWorkspaceStore } from "@/stores/useWorkspaceStore";

interface BottomBarProps {
  lockedProject: string;
  currentPage: Page;
  onPageChange: (page: Page) => void;
}

export function BottomBar({ lockedProject, currentPage, onPageChange }: BottomBarProps) {
  const { status, maxIterations, setMaxIterations, addOutput, createSession } = useLoopStore(); // TODO: Replace maxIterations with loopEnabled
  const { openTab } = useWorkspaceStore();

  // State detection
  const isIdle = status.state === "idle";
  const isRunning = status.state === "running";
  const isPaused = status.state === "paused";
  const isRateLimited = status.state === "rate_limited";
  const isComplete = status.state === "complete";
  const isAborted = status.state === "aborted";

  // Primary button logic (Play/Pause)
  const canPause = isRunning;
  const canStop = isRunning || isPaused || isRateLimited;

  const handlePrimaryAction = async () => {
    try {
      if (canPause) {
        // Currently running → pause
        await invoke("pause_loop");
        addOutput("Loop paused", "info");
      } else if (isPaused) {
        // Currently paused → resume
        await invoke("resume_loop");
        addOutput("Loop resumed", "info");
      } else if (isIdle || isComplete || isAborted) {
        // Idle/complete/aborted → start new loop
        const sessionId = createSession();
        const timestamp = new Date().toLocaleTimeString("en-US", {
          hour: "2-digit",
          minute: "2-digit",
          hour12: false,
        });

        openTab({
          type: "terminal",
          title: `Loop ${timestamp}`,
          closeable: true,
          data: { sessionId },
        });

        addOutput(`Starting loop on: ${lockedProject}`, "info");
        // TODO: Remove this log - iteration count is deprecated
        addOutput(`Max iterations: ${maxIterations}`, "info");
        // TODO: Replace with: await invoke("start_loop", { loopEnabled: maxIterations > 1 })
        await invoke("start_loop", { maxIterations });
      }
    } catch (e) {
      addOutput(`Error: ${e}`, "error");
    }
  };

  const handleStop = async () => {
    try {
      await invoke("stop_loop");
      addOutput("Loop stopped", "info");
    } catch (e) {
      addOutput(`Error: ${e}`, "error");
    }
  };

  // Determine primary button icon and label
  const getPrimaryButton = () => {
    if (isRunning) {
      return { icon: Pause, label: "Pause", disabled: false };
    }
    if (isPaused) {
      return { icon: Play, label: "Resume", disabled: false };
    }
    if (isRateLimited) {
      return { icon: Play, label: "Start", disabled: true };
    }
    // idle, complete, aborted
    return { icon: Play, label: "Start", disabled: false };
  };

  const primaryButton = getPrimaryButton();
  const PrimaryIcon = primaryButton.icon;

  return (
    <div className="border-t bg-[hsl(var(--background))] px-4 py-3">
      <div className="flex items-center justify-between gap-4">
        {/* Left: Navigation Menu */}
        <div className="flex-1">
          <NavigationMenu currentPage={currentPage} onPageChange={onPageChange} />
        </div>

        {/* Center: Transport Controls and Loop Toggle */}
        <div className="flex items-center gap-3">
          <LoopToggle
            maxIterations={maxIterations}
            setMaxIterations={setMaxIterations}
            disabled={!isIdle && !isComplete && !isAborted}
          />

          <Button
            onClick={handlePrimaryAction}
            disabled={primaryButton.disabled || isRateLimited}
            size="icon"
            variant="default"
            title={primaryButton.label}
            className="h-10 w-10"
          >
            <PrimaryIcon className="h-5 w-5" />
          </Button>

          <Button
            onClick={handleStop}
            disabled={!canStop}
            size="icon"
            variant="outline"
            title="Stop"
            className="h-10 w-10"
          >
            <Square className="h-5 w-5" />
          </Button>
        </div>

        {/* Right: Settings */}
        <div className="flex-1 flex justify-end">
          <Settings />
        </div>
      </div>
    </div>
  );
}
