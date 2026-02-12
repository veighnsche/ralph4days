import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import type { Task } from '@/types/generated'
import type { TaskDetailTabParams } from './schema'

export function createTaskDetailTab(task: Task): Omit<WorkspaceTab, 'id'> {
  return {
    type: 'task-detail',
    title: task.title,
    key: String(task.id),
    closeable: true,
    params: {
      entityId: task.id,
      entity: task
    } satisfies TaskDetailTabParams
  }
}
