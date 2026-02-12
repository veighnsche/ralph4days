import { Bot } from 'lucide-react'
import { InlineError } from '@/components/shared'
import { Button } from '@/components/ui/button'
import { FormDescription, FormHeader, FormTitle } from '@/components/ui/form-header'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'
import { useAgentSessionLaunchPreferences } from '@/hooks/preferences'
import { useTabMeta } from '@/hooks/workspace'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import { AgentProviderPicker, ModelPicker, PermissionLevelControls } from './components'
import { useAgentSessionConfigController } from './hooks'
import { useAgentSessionConfigLaunchState } from './hooks/useAgentSessionConfigTabState'
import type { AgentSessionConfigTabParams } from './schema'
import { buildInitialLaunchConfig } from './state'
import { AgentSessionConfigStoreProvider } from './store'

function AgentSessionConfigTabBody({ tab }: { tab: WorkspaceTab }) {
  const { models, loadingModels, error, runSession } = useAgentSessionConfigController(tab)
  const { model } = useAgentSessionConfigLaunchState()

  return (
    <div className="h-full flex flex-col">
      <div className="px-4">
        <FormHeader>
          <FormTitle>Start Agent Session</FormTitle>
          <FormDescription>Configure launch options, then start an agent session.</FormDescription>
        </FormHeader>
      </div>
      <Separator />
      <ScrollArea className="flex-1 min-h-0">
        <div className="px-4 py-4 space-y-4">
          <AgentProviderPicker />
          <ModelPicker models={models} loadingModels={loadingModels} />
        </div>
      </ScrollArea>
      <Separator />
      {error && (
        <div className="px-3 pt-1.5">
          <InlineError error={error} />
        </div>
      )}
      <div className="px-3 py-1.5 flex justify-end gap-2">
        <PermissionLevelControls />
        <Button type="button" onClick={runSession} disabled={loadingModels || !model || models.length === 0}>
          Run
        </Button>
      </div>
    </div>
  )
}

export function AgentSessionConfigTabContent({
  tab,
  params
}: {
  tab: WorkspaceTab
  params: AgentSessionConfigTabParams
}) {
  useTabMeta(tab.id, 'Start Agent Session', Bot)

  const getDefaultModel = useAgentSessionLaunchPreferences(s => s.getDefaultModel)
  const effort = useAgentSessionLaunchPreferences(s => s.effort)
  const thinking = useAgentSessionLaunchPreferences(s => s.thinking)
  const permissionLevel = useAgentSessionLaunchPreferences(s => s.permissionLevel)

  const initialConfig = buildInitialLaunchConfig(params, getDefaultModel, {
    effort,
    thinking,
    permissionLevel
  })

  return (
    <AgentSessionConfigStoreProvider initialConfig={initialConfig}>
      <AgentSessionConfigTabBody tab={tab} />
    </AgentSessionConfigStoreProvider>
  )
}
