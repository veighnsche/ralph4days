import { Brain, ChevronDown, Plus, TerminalSquare } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { useAgentSessionLaunchPreferences } from '@/hooks/preferences'
import type { AgentSessionLaunchConfig, Model } from '@/lib/agent-session-launch-config'
import { cn } from '@/lib/utils'
import { WORKSPACE_SELECTORS } from '@/test/selectors'
import { LaunchOptionsReadout } from './LaunchOptionsReadout'
import { resolveLaunchConfigAgainstCatalog } from './resolveLaunchConfig'

export type { Model }

interface AgentSessionLaunchButtonProps {
  onNewTab: (config: AgentSessionLaunchConfig) => void
  onOpenRunForm: (config: AgentSessionLaunchConfig) => void
  onNewTestingShellTab?: () => void
}

export function AgentSessionLaunchButton({
  onNewTab,
  onOpenRunForm,
  onNewTestingShellTab
}: AgentSessionLaunchButtonProps) {
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
        {import.meta.env.DEV && onNewTestingShellTab && (
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="sm"
                onClick={onNewTestingShellTab}
                className={cn(
                  'h-7 px-2 rounded-md mr-1',
                  'text-muted-foreground hover:text-foreground hover:bg-accent/50'
                )}>
                <TerminalSquare className="h-4 w-4" />
                <span className="sr-only">New shell terminal testing tab</span>
              </Button>
            </TooltipTrigger>
            <TooltipContent>Open shell tab (testing only)</TooltipContent>
          </Tooltip>
        )}

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
            <LaunchOptionsReadout
              variant="tooltip"
              className="text-background"
              rows={[
                { label: 'Agent', value: agent, valueTitle: agent, monospace: true },
                { label: 'Model', value: model, valueTitle: model, monospace: true },
                { label: 'Effort', value: effort, valueTitle: effort, monospace: true },
                {
                  label: 'Thinking',
                  valueTitle: thinking ? 'on' : 'off',
                  value: (
                    <span className="inline-flex items-center gap-1">
                      <Brain className="h-3 w-3" aria-hidden="true" />
                      {thinking ? 'on' : 'off'}
                    </span>
                  )
                },
                { label: 'Perm', value: permissionLevel, valueTitle: permissionLevel, monospace: true }
              ]}
            />
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
