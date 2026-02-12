import { useWorkspaceStore, type WorkspaceTab } from '@/stores/useWorkspaceStore'
import { createTerminalTabFromLaunch } from '../../terminal/factory'
import { AGENT_PROVIDER_META } from '../constants'
import { useAgentSessionConfigLaunchState } from './useAgentSessionConfigTabState'

export function useRunSessionAction(
  tab: WorkspaceTab,
  { selectedModelDisplay, canRun }: { selectedModelDisplay: string | undefined; canRun: boolean }
) {
  const closeTab = useWorkspaceStore(s => s.closeTab)
  const openTab = useWorkspaceStore(s => s.openTab)
  const { agent, model, effort, thinking, permissionLevel } = useAgentSessionConfigLaunchState()

  return () => {
    if (!canRun) return

    openTab(
      createTerminalTabFromLaunch(
        {
          agent,
          model,
          effort,
          thinking,
          permissionLevel
        },
        {
          title: `${AGENT_PROVIDER_META[agent].label} (${selectedModelDisplay ?? model})`
        }
      )
    )
    closeTab(tab.id)
  }
}
