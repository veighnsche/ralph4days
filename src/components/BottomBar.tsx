import { Pause, Play, Square } from "lucide-react";
import { LoopCountBadge } from "@/components/LoopCountBadge";
import { Settings } from "@/components/Settings";
import { Button } from "@/components/ui/button";
import { universalInvoke } from "@/services/mockBackend";
import { useLoopStore } from "@/stores/useLoopStore";

interface BottomBarProps {
  lockedProject: string;
}

export function BottomBar({ lockedProject }: BottomBarProps) {
  const { status, maxIterations, setMaxIterations, addOutput, clearOutput } = useLoopStore();

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
        await universalInvoke("pause_loop");
        addOutput("Loop paused", "info");
      } else if (isPaused) {
        // Currently paused → resume
        await universalInvoke("resume_loop");
        addOutput("Loop resumed", "info");
      } else if (isIdle || isComplete || isAborted) {
        // Idle/complete/aborted → start
        clearOutput();
        addOutput(`Starting loop on: ${lockedProject}`, "info");
        addOutput(`Max iterations: ${maxIterations}`, "info");
        await universalInvoke("start_loop", { maxIterations });
      }
    } catch (e) {
      addOutput(`Error: ${e}`, "error");
    }
  };

  const handleStop = async () => {
    try {
      await universalInvoke("stop_loop");
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
        {/* Left: Empty */}
        <div className="flex-1" />

        {/* Center: Transport Controls and Iterations */}
        <div className="flex items-center gap-3">
          <LoopCountBadge status={status} maxIterations={maxIterations} setMaxIterations={setMaxIterations} />

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
            variant="ghost"
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
