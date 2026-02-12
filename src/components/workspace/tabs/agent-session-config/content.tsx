import { Bot } from 'lucide-react'
import { useMemo } from 'react'
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
import type { AgentSessionConfigTabParams } from './schema'
import { buildInitialLaunchConfig } from './state'
import { AgentSessionConfigStoreProvider, useAgentSessionConfigStore } from './store'

function AgentSessionConfigTabBody({ tab }: { tab: WorkspaceTab }) {
  const { loadingModels, error, runSession } = useAgentSessionConfigController(tab)
  const model = useAgentSessionConfigStore(state => state.model)
  const models = useAgentSessionConfigStore(state => state.models)

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
          <ModelPicker />
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

  const initialConfig = useMemo(
    () =>
      buildInitialLaunchConfig(params, getDefaultModel, {
        effort,
        thinking,
        permissionLevel
      }),
    [effort, getDefaultModel, params, permissionLevel, thinking]
  )

  return (
    <AgentSessionConfigStoreProvider initialConfig={initialConfig}>
      <AgentSessionConfigTabBody tab={tab} />
    </AgentSessionConfigStoreProvider>
  )
}
