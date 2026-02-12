import {
  createAgentSessionConfigTab,
  createDisciplineDetailTab,
  createFeatureDetailTab,
  createTaskDetailTab,
  createTerminalTab
} from '@/components/workspace/tabs'
import type { Agent, AgentSessionLaunchConfig, Effort, PermissionLevel } from '@/hooks/preferences'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'
import type { FeatureData, Task } from '@/types/generated'

export function useWorkspaceActions() {
  const openTab = useWorkspaceStore(s => s.openTab)

  return {
    openDisciplineDetailTab: (discipline: { name: string; displayName: string }) =>
      openTab(createDisciplineDetailTab(discipline)),

    openFeatureDetailTab: (feature: FeatureData) => openTab(createFeatureDetailTab(feature)),

    openTaskDetailTab: (task: Task) => openTab(createTaskDetailTab(task)),

    openTerminalFromLaunchConfig: (config: AgentSessionLaunchConfig) => openTab(createTerminalTab(config)),

    openAgentSessionConfigTab: (config: AgentSessionLaunchConfig) => openTab(createAgentSessionConfigTab(config)),

    openDefaultTerminalTab: () => openTab(createTerminalTab({ title: 'Terminal 1' })),

    openTerminalTab: (
      agent: Agent,
      model: string,
      effort: Effort,
      thinking: boolean,
      permissionLevel: PermissionLevel,
      initPrompt?: string
    ) => {
      return openTab(createTerminalTab({ agent, model, effort, thinking, permissionLevel, initPrompt }))
    }
  }
}
