import { Bot } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useAgentSessionLaunchPreferences } from '@/hooks/preferences'
import { useTabMeta } from '@/hooks/workspace'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import { AgentSessionLaunchForm } from './components/AgentSessionLaunchForm'
import { PermissionLevelControls } from './components/PermissionLevelControls'
import { useAgentSessionConfigController } from './hooks'
import { useAgentSessionConfigLaunchState } from './hooks/useAgentSessionConfigTabState'
import type { AgentSessionConfigTabParams } from './schema'
import { buildInitialLaunchConfig } from './state'
import { AgentSessionConfigStoreProvider } from './store'

function AgentSessionConfigTabBody({ tab }: { tab: WorkspaceTab }) {
  const { models, loadingModels, error, canRun, runSession } = useAgentSessionConfigController(tab)
  const { model } = useAgentSessionConfigLaunchState()

  return (
    <AgentSessionLaunchForm
      layout="two_column"
      showHeader
      models={models}
      loadingModels={loadingModels}
      error={error}
      footer={
        <div className="flex justify-end gap-2">
          <PermissionLevelControls />
          <Button
            type="button"
            onClick={runSession}
            disabled={loadingModels || !model || models.length === 0 || !canRun}>
            Run
          </Button>
        </div>
      }
    />
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
