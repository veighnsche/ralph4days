import { ChevronDown } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
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
import { type Agent, type Model, useModelThinkingPreferences } from '@/hooks/preferences'
import { terminalBridgeListModels } from '@/lib/terminal/terminalBridgeClient'
import { cn } from '@/lib/utils'

export type { Model } from '@/hooks/preferences'

interface ModelThinkingPickerProps {
  onAction: (agent: Agent, model: Model, thinking: boolean) => void
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
  const { agent, setAgent, model, setModel, thinking, setThinking, getDefaultModel } = useModelThinkingPreferences()
  const [models, setModels] = useState<string[]>([])

  const fallbackModels = useMemo(
    () => ({
      claude: ['claude-sonnet-4', 'claude-opus-4', 'claude-haiku-3.5'],
      codex: ['gpt-5-codex', 'gpt-5', 'o4-mini']
    }),
    []
  )

  useEffect(() => {
    let cancelled = false
    terminalBridgeListModels(agent)
      .then(res => {
        if (cancelled) return
        const nextModels = res.models.length > 0 ? res.models : fallbackModels[agent]
        setModels(nextModels)
        if (!nextModels.includes(model)) {
          setModel(nextModels[0] ?? getDefaultModel(agent))
        }
      })
      .catch(() => {
        if (cancelled) return
        const nextModels = fallbackModels[agent]
        setModels(nextModels)
        if (!nextModels.includes(model)) {
          setModel(nextModels[0] ?? getDefaultModel(agent))
        }
      })
    return () => {
      cancelled = true
    }
  }, [agent, fallbackModels, getDefaultModel, model, setModel])

  const handleAction = () => {
    onAction(agent, model, thinking)
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
            <DropdownMenuLabel>Agent</DropdownMenuLabel>
            <DropdownMenuRadioGroup value={agent} onValueChange={v => setAgent(v as Agent)}>
              <DropdownMenuRadioItem value="claude">Claude</DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="codex">Codex</DropdownMenuRadioItem>
            </DropdownMenuRadioGroup>

            <DropdownMenuSeparator />

            <DropdownMenuLabel>Model</DropdownMenuLabel>
            <DropdownMenuRadioGroup value={model} onValueChange={v => setModel(v as Model)}>
              {models.map(modelName => (
                <DropdownMenuRadioItem key={modelName} value={modelName}>
                  {modelName}
                </DropdownMenuRadioItem>
              ))}
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
