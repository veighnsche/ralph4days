import { RotateCw } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'

// TODO: Wire up to new terminal-based task execution system
export function LoopToggle() {
  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <Button disabled size="icon" variant="outline" title="Execution disabled">
            <RotateCw className="h-4 w-4 opacity-50" />
          </Button>
        </TooltipTrigger>
        <TooltipContent>
          <div>Execution: Disabled</div>
          <div className="text-[0.65rem] opacity-70">Pending terminal integration</div>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  )
}
