import { CheckCircle2 } from 'lucide-react'
import { CroppedImage } from '@/components/ui/cropped-image'
import { STATUS_CONFIG } from '@/constants/prd'
import { useDisciplines } from '@/hooks/disciplines'
import { usePRDData } from '@/hooks/tasks'
import { useTabMeta } from '@/hooks/workspace'
import { computeInferredStatus } from '@/lib/taskStatus'
import { NOOP_TAB_LIFECYCLE, type WorkspaceTab } from '@/stores/useWorkspaceStore'
import type { Task } from '@/types/generated'
import { DetailPageLayout } from '../DetailPageLayout'
import { CommentsSection } from '../task-detail'
import { TaskCardContent } from '../task-detail/TaskCardContent'
import { TaskSidebar } from '../task-detail/TaskSidebar'

export type TaskDetailTabParams = {
  entityId: number
  entity?: Task
}

function parseTaskDetailTabParams(params: unknown): TaskDetailTabParams {
  if (typeof params !== 'object' || params == null || Array.isArray(params)) {
    throw new Error('Invalid task detail tab params: expected object')
  }
  const candidate = params as Record<string, unknown>
  if (!Number.isInteger(candidate.entityId)) {
    throw new Error('Invalid task detail tab params.entityId')
  }
  const parsed: TaskDetailTabParams = {
    entityId: candidate.entityId as number
  }
  if (candidate.entity !== undefined) {
    if (typeof candidate.entity !== 'object' || candidate.entity == null || Array.isArray(candidate.entity)) {
      throw new Error('Invalid task detail tab params.entity')
    }
    parsed.entity = candidate.entity as Task
  }
  return parsed
}

export function createTaskDetailTab(task: Task): Omit<WorkspaceTab, 'id'> {
  return {
    type: 'task-detail',
    component: TaskDetailTabContent,
    title: task.title,
    key: String(task.id),
    closeable: true,
    lifecycle: NOOP_TAB_LIFECYCLE,
    params: {
      entityId: task.id,
      entity: task
    } satisfies TaskDetailTabParams
  }
}

export function TaskDetailTabContent({ tab }: { tab: WorkspaceTab }) {
  const { entityId, entity: snapshotTask } = parseTaskDetailTabParams(tab.params)

  const { tasks } = usePRDData()
  const task = (entityId != null ? tasks?.find(t => t.id === entityId) : undefined) ?? snapshotTask
  const { disciplines } = useDisciplines()

  useTabMeta(tab.id, task?.title ?? 'Task Detail', CheckCircle2)

  if (!task) {
    return (
      <div className="h-full flex items-center justify-center text-muted-foreground">
        <span>Task not found</span>
      </div>
    )
  }

  const statusConfig = STATUS_CONFIG[task.status]
  const disc = disciplines.find(d => d.name === task.discipline)
  const stripCrop = disc?.imagePath ? disc?.crops?.strip : undefined
  const faceCrop = disc?.crops?.face
  const eyelinePercent = faceCrop ? Math.round((faceCrop.y + faceCrop.h / 2) * 100) : 30

  return (
    <DetailPageLayout
      accentColor={statusConfig.color}
      sidebarImage={
        stripCrop && (
          <CroppedImage
            disciplineName={task.discipline}
            label="strip"
            crop={stripCrop}
            className="absolute inset-0 w-full h-full object-cover opacity-15"
            style={{ objectPosition: `center ${eyelinePercent}%` }}
          />
        )
      }
      mainContent={<TaskCardContent task={task} />}
      sidebar={<TaskSidebar task={task} inferredStatus={computeInferredStatus(task, tasks ?? [])} />}>
      <CommentsSection task={task} />
    </DetailPageLayout>
  )
}
