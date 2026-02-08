import {
  BraindumpFormTabContent,
  DisciplineFormTabContent,
  FeatureDetailTabContent,
  FeatureFormTabContent,
  TaskDetailTabContent,
  TaskFormTabContent,
  TerminalTabContent
} from '@/components/workspace'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'
import type { FeatureData, Task } from '@/types/generated'

export function useWorkspaceActions() {
  const openTab = useWorkspaceStore(s => s.openTab)

  return {
    openBraindumpTab: (title: string) =>
      openTab({ type: 'braindump-form', component: BraindumpFormTabContent, title, closeable: true }),

    openCreateTaskTab: () =>
      openTab({
        type: 'task-form',
        component: TaskFormTabContent,
        title: 'Create Task',
        closeable: true,
        data: { mode: 'create' }
      }),

    openCreateFeatureTab: () =>
      openTab({
        type: 'feature-form',
        component: FeatureFormTabContent,
        title: 'Create Feature',
        closeable: true,
        data: { mode: 'create' }
      }),

    openCreateDisciplineTab: () =>
      openTab({
        type: 'discipline-form',
        component: DisciplineFormTabContent,
        title: 'Create Discipline',
        closeable: true,
        data: { mode: 'create' }
      }),

    openFeatureDetailTab: (feature: FeatureData) =>
      openTab({
        type: 'feature-detail',
        component: FeatureDetailTabContent,
        title: feature.displayName,
        closeable: true,
        data: { entityId: feature.name }
      }),

    openTaskDetailTab: (task: Task) =>
      openTab({
        type: 'task-detail',
        component: TaskDetailTabContent,
        title: task.title,
        closeable: true,
        data: { entityId: task.id, entity: task }
      }),

    openTerminalTab: (model: string, thinking: boolean) =>
      openTab({
        type: 'terminal',
        component: TerminalTabContent,
        title: `Claude (${model})`,
        closeable: true,
        data: { model, thinking }
      })
  }
}
