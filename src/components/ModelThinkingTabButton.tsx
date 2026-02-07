/**
 * ModelThinkingTabButton - Tab bar variant of model/thinking picker
 *
 * Styled to match browser tab bar aesthetics.
 * Used in WorkspacePanel for the "new tab" plus button.
 */

import { ChevronDown, Plus } from 'lucide-react'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger
} from '@/components/ui/dropdown-menu'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { type Model, useModelThinkingPreferences } from '@/hooks/useModelThinkingPreferences'
import { cn } from '@/lib/utils'

export type { Model } from '@/hooks/useModelThinkingPreferences'

interface ModelThinkingTabButtonProps {
  /** Called when user clicks the plus button to create new tab */
  onNewTab: (model: Model, thinking: boolean) => void
}

export function ModelThinkingTabButton({ onNewTab }: ModelThinkingTabButtonProps) {
  const { model, setModel, thinking, setThinking } = useModelThinkingPreferences()

  const handleNewTab = () => {
    onNewTab(model, thinking)
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
                <span className="font-semibold">Model:</span> {model.charAt(0).toUpperCase() + model.slice(1)}
              </div>
              <div>
                <span className="font-semibold">Thinking:</span> {thinking ? 'On' : 'Off'}
              </div>
            </div>
          </TooltipContent>
        </Tooltip>

        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              variant="ghost"
              size="sm"
              className={cn(
                'h-7 px-1 rounded-md rounded-l-none border-l border-border/50',
                'text-muted-foreground hover:text-foreground hover:bg-accent/50'
              )}>
              <ChevronDown className="h-3 w-3" />
              <span className="sr-only">Terminal options</span>
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" className="w-48">
            <DropdownMenuLabel>Model</DropdownMenuLabel>
            <DropdownMenuRadioGroup value={model} onValueChange={v => setModel(v as Model)}>
              <DropdownMenuRadioItem value="haiku">Haiku (fast)</DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="sonnet">Sonnet (balanced)</DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="opus">Opus (smart)</DropdownMenuRadioItem>
            </DropdownMenuRadioGroup>

            <DropdownMenuSeparator />

            <DropdownMenuLabel>Options</DropdownMenuLabel>
            <DropdownMenuCheckboxItem checked={thinking} onCheckedChange={setThinking}>
              Extended thinking
            </DropdownMenuCheckboxItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </TooltipProvider>
  )
}
