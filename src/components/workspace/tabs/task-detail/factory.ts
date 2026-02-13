import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import type { TaskDetailTabParams } from './schema'

export function createTaskDetailTab(taskId: number): Omit<WorkspaceTab, 'id'> {
  return {
    type: 'task-detail',
    title: `Task #${taskId.toString().padStart(3, '0')}`,
    key: String(taskId),
    closeable: true,
    params: {
      entityId: taskId
    } satisfies TaskDetailTabParams
  }
}
