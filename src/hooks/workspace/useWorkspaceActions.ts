import { FeatureDetailTabContent, TaskDetailTabContent, TerminalTabContent } from '@/components/workspace'
import type { Agent, Effort } from '@/hooks/preferences'
import { NOOP_TAB_LIFECYCLE, useWorkspaceStore } from '@/stores/useWorkspaceStore'
import type { FeatureData, Task } from '@/types/generated'

export function useWorkspaceActions() {
  const openTab = useWorkspaceStore(s => s.openTab)

  return {
    openFeatureDetailTab: (feature: FeatureData) =>
      openTab({
        type: 'feature-detail',
        component: FeatureDetailTabContent,
        title: feature.displayName,
        closeable: true,
        lifecycle: NOOP_TAB_LIFECYCLE,
        data: { entityId: feature.name }
      }),

    openTaskDetailTab: (task: Task) =>
      openTab({
        type: 'task-detail',
        component: TaskDetailTabContent,
        title: task.title,
        closeable: true,
        lifecycle: NOOP_TAB_LIFECYCLE,
        data: { entityId: task.id, entity: task }
      }),

    openTerminalTab: (agent: Agent, model: string, effort: Effort, thinking: boolean, initPrompt?: string) => {
      const agentLabel = agent === 'codex' ? 'Codex' : 'Claude'
      return openTab({
        type: 'terminal',
        component: TerminalTabContent,
        title: `${agentLabel} (${model})`,
        closeable: true,
        lifecycle: NOOP_TAB_LIFECYCLE,
        data: { agent, model, effort: agent === 'claude' ? effort : undefined, thinking, initPrompt }
      })
    }
  }
}
