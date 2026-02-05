import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useLoopStore } from "@/stores/useLoopStore";
import { Play, Pause, Square, RotateCcw } from "lucide-react";

interface LoopControlsProps {
  lockedProject: string;
}

export function LoopControls({ lockedProject }: LoopControlsProps) {
  const {
    status,
    maxIterations,
    setMaxIterations,
    addOutput,
    clearOutput,
  } = useLoopStore();

  // Extract project name from path
  const projectName = lockedProject.split('/').pop() || 'Unknown';

  const isIdle = status.state === "idle";
  const isRunning = status.state === "running";
  const isPaused = status.state === "paused";
  const isRateLimited = status.state === "rate_limited";
  const canStart = isIdle;
  const canPause = isRunning;
  const canResume = isPaused;
  const canStop = isRunning || isPaused || isRateLimited;

  const handleStart = async () => {
    try {
      clearOutput();
      addOutput(`Starting loop on: ${lockedProject}`, "info");
      addOutput(`Max iterations: ${maxIterations}`, "info");
      await invoke("start_loop", {
        maxIterations,
      });
    } catch (e) {
      addOutput(`Error: ${e}`, "error");
    }
  };

  const handlePause = async () => {
    try {
      await invoke("pause_loop");
      addOutput("Loop paused", "info");
    } catch (e) {
      addOutput(`Error: ${e}`, "error");
    }
  };

  const handleResume = async () => {
    try {
      await invoke("resume_loop");
      addOutput("Loop resumed", "info");
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

  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <label className="text-sm font-medium text-[hsl(var(--muted-foreground))]">
          Locked Project
        </label>
        <div className="border border-[hsl(var(--input))] bg-[hsl(var(--muted))] px-3 py-2 text-sm space-y-1">
          <div className="font-mono text-xs break-all">{lockedProject}</div>
          <div className="text-[hsl(var(--muted-foreground))]">{projectName}/.ralph</div>
        </div>
      </div>

      <div className="space-y-2">
        <label className="text-sm font-medium text-[hsl(var(--muted-foreground))]">
          Max Iterations
        </label>
        <Input
          type="number"
          value={maxIterations}
          onChange={(e) => setMaxIterations(parseInt(e.target.value) || 100)}
          min={1}
          max={1000}
          disabled={!isIdle}
        />
      </div>

      <div className="flex flex-wrap gap-2 pt-2">
        <Button onClick={handleStart} disabled={!canStart} className="gap-2">
          <Play className="h-4 w-4" />
          Start
        </Button>

        <Button
          onClick={handlePause}
          disabled={!canPause}
          variant="secondary"
          className="gap-2"
        >
          <Pause className="h-4 w-4" />
          Pause
        </Button>

        <Button
          onClick={handleResume}
          disabled={!canResume}
          variant="secondary"
          className="gap-2"
        >
          <RotateCcw className="h-4 w-4" />
          Resume
        </Button>

        <Button
          onClick={handleStop}
          disabled={!canStop}
          variant="destructive"
          className="gap-2"
        >
          <Square className="h-4 w-4" />
          Stop
        </Button>
      </div>
    </div>
  );
}
