import { ChevronDown } from 'lucide-react'
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

interface ModelThinkingPickerProps {
  onAction: (model: Model, thinking: boolean) => void
  actionLabel: string
  actionIcon?: React.ReactNode
  disabled?: boolean
  variant?: 'default' | 'outline' | 'ghost'
  size?: 'sm' | 'default' | 'lg'
  className?: string
}

export function ModelThinkingPicker({
  onAction,
  actionLabel,
  actionIcon,
  disabled = false,
  variant = 'default',
  size = 'default',
  className
}: ModelThinkingPickerProps) {
  const { model, setModel, thinking, setThinking } = useModelThinkingPreferences()

  const handleAction = () => {
    onAction(model, thinking)
  }

  return (
    <TooltipProvider>
      <div className={cn('flex', className)}>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              type="button"
              variant={variant}
              size={size}
              onClick={handleAction}
              disabled={disabled}
              className="rounded-r-none">
              {actionIcon}
              {actionLabel}
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
              type="button"
              variant={variant}
              size={size}
              disabled={disabled}
              className="rounded-l-none border-l px-2">
              <ChevronDown className="h-4 w-4" />
              <span className="sr-only">Model options</span>
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
