import { ChevronDown, Plus } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { type Agent, type Model, useModelThinkingPreferences } from '@/hooks/preferences'
import { cn } from '@/lib/utils'

export type { Model } from '@/hooks/preferences'

interface ModelThinkingTabButtonProps {
  onNewTab: (agent: Agent, model: Model, thinking: boolean) => void
  onOpenRunForm: (agent: Agent, model: Model, thinking: boolean) => void
}

export function ModelThinkingTabButton({ onNewTab, onOpenRunForm }: ModelThinkingTabButtonProps) {
  const { agent, model, thinking } = useModelThinkingPreferences()

  const handleNewTab = () => {
    onNewTab(agent, model, thinking)
  }

  const handleOpenRunForm = () => {
    onOpenRunForm(agent, model, thinking)
  }

  return (
    <TooltipProvider>
      <div className="flex flex-none">
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="sm"
              onClick={handleNewTab}
              className={cn(
                'h-7 px-2 rounded-md rounded-r-none',
                'text-muted-foreground hover:text-foreground hover:bg-accent/50'
              )}>
              <Plus className="h-4 w-4" />
              <span className="sr-only">New terminal</span>
            </Button>
          </TooltipTrigger>
          <TooltipContent>
            <div className="space-y-1">
              <div>
                <span className="font-semibold">Agent:</span> {agent}
              </div>
              <div>
                <span className="font-semibold">Model:</span> {model}
              </div>
              <div>
                <span className="font-semibold">Thinking:</span> {thinking ? 'On' : 'Off'}
              </div>
            </div>
          </TooltipContent>
        </Tooltip>

        <Button
          variant="ghost"
          size="sm"
          onClick={handleOpenRunForm}
          className={cn(
            'h-7 px-1 rounded-md rounded-l-none border-l border-border/50',
            'text-muted-foreground hover:text-foreground hover:bg-accent/50'
          )}>
          <ChevronDown className="h-3 w-3" />
          <span className="sr-only">Open run form</span>
        </Button>
      </div>
    </TooltipProvider>
  )
}
