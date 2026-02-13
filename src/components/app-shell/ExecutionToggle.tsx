import { FastForward } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'

interface ExecutionToggleProps {
  disabled: boolean
  onClick: () => void
  title?: string
}

export function ExecutionToggle({ disabled, onClick, title }: ExecutionToggleProps) {
  const iconClass = disabled ? 'h-4 w-4 opacity-50' : 'h-4 w-4'

  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            disabled={disabled}
            size="icon"
            variant="outline"
            title={title ?? 'Execution action'}
            onClick={onClick}>
            <FastForward className={iconClass} />
          </Button>
        </TooltipTrigger>
        <TooltipContent>
          <div>Execution action</div>
          <div className="text-[0.65rem] opacity-70">{title ?? 'Task execution flow'}</div>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  )
}
