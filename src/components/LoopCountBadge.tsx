import { RotateCw } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import type { LoopState } from "@/stores/useLoopStore";

interface LoopStatus {
  state: LoopState;
  current_iteration: number;
  max_iterations: number;
}

interface LoopCountBadgeProps {
  status: LoopStatus;
  maxIterations: number;
  setMaxIterations: (value: number) => void;
}

export function LoopCountBadge({ status, maxIterations, setMaxIterations }: LoopCountBadgeProps) {
  const isIdle = status.state === "idle";

  // Calculate iterations remaining (counts down as loop progresses)
  const iterationsRemaining = !isIdle ? status.max_iterations - status.current_iteration : maxIterations;

  return (
    <Badge variant="secondary" className="gap-1.5 px-3 py-1.5">
      <RotateCw className="h-3.5 w-3.5" />
      {isIdle ? (
        <input
          type="text"
          inputMode="numeric"
          value={maxIterations}
          onChange={(e) => {
            const val = parseInt(e.target.value, 10);
            if (!Number.isNaN(val) && val >= 1 && val <= 1000) {
              setMaxIterations(val);
            } else if (e.target.value === "") {
              setMaxIterations(1);
            }
          }}
          onBlur={(e) => {
            if (e.target.value === "" || parseInt(e.target.value, 10) < 1) {
              setMaxIterations(1);
            }
          }}
          className="w-8 bg-transparent text-sm font-mono text-center outline-none focus:outline-none border-0 p-0"
          title="Max iterations"
        />
      ) : (
        <span
          className="text-sm font-mono w-8 text-center"
          title={`${status.current_iteration} / ${status.max_iterations} complete`}
        >
          {iterationsRemaining}
        </span>
      )}
    </Badge>
  );
}
