// TODO: DEPRECATED ITERATION LOGIC - Remove maxIterations logic
// - Change to simple boolean: isLoopEnabled (true = infinite, false = 1)
// - Remove setMaxIterations, use setLoopEnabled(boolean)
// - Backend should support infinite loops (run until stopped)
// - No more iteration counting/tracking

import { RotateCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";

interface LoopToggleProps {
  maxIterations: number; // TODO: Replace with isLoopEnabled: boolean
  setMaxIterations: (value: number) => void; // TODO: Replace with setLoopEnabled: (enabled: boolean) => void
  disabled?: boolean;
}

export function LoopToggle({ maxIterations, setMaxIterations, disabled }: LoopToggleProps) {
  const isLoopEnabled = maxIterations > 1;

  const handleToggle = () => {
    if (isLoopEnabled) {
      // Disable looping - run once
      setMaxIterations(1);
    } else {
      // Enable looping - run many iterations
      // TODO: This should be infinite, not 100
      setMaxIterations(100);
    }
  };

  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            onClick={handleToggle}
            disabled={disabled}
            size="icon"
            variant={isLoopEnabled ? "default" : "outline"}
            title={isLoopEnabled ? "Loop enabled" : "Loop disabled"}
            className="h-10 w-10"
          >
            <RotateCw className={`h-5 w-5 ${isLoopEnabled ? "" : "opacity-50"}`} />
          </Button>
        </TooltipTrigger>
        <TooltipContent>
          <div>Loop: {isLoopEnabled ? "Enabled" : "Disabled"}</div>
          <div className="text-[0.65rem] opacity-70">
            {/* TODO: Change to "Runs infinitely" instead of showing iteration count */}
            {isLoopEnabled ? `Runs ${maxIterations} iterations` : "Runs once"}
          </div>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}
