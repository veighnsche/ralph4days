import { RotateCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";

// TODO: Wire up to new terminal-based loop system
export function LoopToggle() {
  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <Button disabled size="icon" variant="outline" title="Loop disabled" className="h-10 w-10">
            <RotateCw className="h-5 w-5 opacity-50" />
          </Button>
        </TooltipTrigger>
        <TooltipContent>
          <div>Loop: Disabled</div>
          <div className="text-[0.65rem] opacity-70">Pending terminal integration</div>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}
