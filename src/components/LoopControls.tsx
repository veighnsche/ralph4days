import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useLoopStore } from "@/stores/useLoopStore";
import { Play, Pause, Square, RotateCcw, FolderOpen } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";

export function LoopControls() {
  const {
    status,
    projectPath,
    maxIterations,
    setProjectPath,
    setMaxIterations,
    addOutput,
    clearOutput,
  } = useLoopStore();

  const isIdle = status.state === "idle";
  const isRunning = status.state === "running";
  const isPaused = status.state === "paused";
  const isRateLimited = status.state === "rate_limited";
  const canStart = isIdle && projectPath.length > 0;
  const canPause = isRunning;
  const canResume = isPaused;
  const canStop = isRunning || isPaused || isRateLimited;

  const handleSelectFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Project Directory",
      });
      if (selected && typeof selected === "string") {
        setProjectPath(selected);
      }
    } catch (e) {
      console.error("Failed to open folder dialog:", e);
    }
  };

  const handleStart = async () => {
    try {
      clearOutput();
      addOutput(`Starting loop on: ${projectPath}`, "info");
      addOutput(`Max iterations: ${maxIterations}`, "info");
      await invoke("start_loop", {
        projectPath,
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
          Project Path
        </label>
        <div className="flex gap-2">
          <Input
            value={projectPath}
            onChange={(e) => setProjectPath(e.target.value)}
            placeholder="/path/to/project"
            disabled={!isIdle}
            className="flex-1"
          />
          <Button
            variant="outline"
            size="icon"
            onClick={handleSelectFolder}
            disabled={!isIdle}
          >
            <FolderOpen className="h-4 w-4" />
          </Button>
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
