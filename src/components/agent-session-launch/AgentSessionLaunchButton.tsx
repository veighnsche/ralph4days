import { ChevronDown, Plus } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { useAgentSessionLaunchPreferences } from '@/hooks/preferences'
import type { AgentSessionLaunchConfig, Model } from '@/lib/agent-session-launch-config'
import { cn } from '@/lib/utils'
import { WORKSPACE_SELECTORS } from '@/test/selectors'
import { resolveLaunchConfigAgainstCatalog } from './resolveLaunchConfig'

export type { Model }

interface AgentSessionLaunchButtonProps {
  onNewTab: (config: AgentSessionLaunchConfig) => void
  onOpenRunForm: (config: AgentSessionLaunchConfig) => void
}

export function AgentSessionLaunchButton({ onNewTab, onOpenRunForm }: AgentSessionLaunchButtonProps) {
  const { agent, model, effort, thinking, permissionLevel, setLaunchConfig } = useAgentSessionLaunchPreferences()

  const launchConfig: AgentSessionLaunchConfig = { agent, model, effort, thinking, permissionLevel }

  const handleNewTab = async () => {
    const resolved = await resolveLaunchConfigAgainstCatalog(launchConfig).catch(() => launchConfig)
    setLaunchConfig(resolved)
    onNewTab(resolved)
  }

  const handleOpenRunForm = async () => {
    const resolved = await resolveLaunchConfigAgainstCatalog(launchConfig).catch(() => launchConfig)
    setLaunchConfig(resolved)
    onOpenRunForm(resolved)
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
              )}
              data-testid={WORKSPACE_SELECTORS.plusButton}>
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
                <span className="font-semibold">Effort:</span> {effort}
              </div>
              <div>
                <span className="font-semibold">Thinking:</span> {thinking ? 'On' : 'Off'}
              </div>
              <div>
                <span className="font-semibold">Permission:</span> {permissionLevel}
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
