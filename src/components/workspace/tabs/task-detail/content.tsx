import { CheckCircle2 } from 'lucide-react'
import { CroppedImage } from '@/components/ui/cropped-image'
import { STATUS_CONFIG } from '@/constants/prd'
import { useDisciplines } from '@/hooks/disciplines'
import { usePRDData } from '@/hooks/tasks'
import { useTabMeta } from '@/hooks/workspace'
import { computeInferredStatus } from '@/lib/taskStatus'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import { DetailPageLayout } from '../../DetailPageLayout'
import { CommentsSection } from '../../task-detail'
import { TaskCardContent } from '../../task-detail/TaskCardContent'
import { TaskSidebar } from '../../task-detail/TaskSidebar'
import {
  TASK_DETAIL_TAB_EMPTY_MESSAGE,
  TASK_DETAIL_TAB_FALLBACK_EYELINE_PERCENT,
  TASK_DETAIL_TAB_FALLBACK_TITLE
} from './constants'
import type { TaskDetailTabParams } from './schema'

export function TaskDetailTabContent({ tab, params }: { tab: WorkspaceTab; params: TaskDetailTabParams }) {
  const { entityId, entity: snapshotTask } = params

  const { tasks } = usePRDData()
  const task = (entityId != null ? tasks?.find(t => t.id === entityId) : undefined) ?? snapshotTask
  const { disciplines } = useDisciplines()

  useTabMeta(tab.id, task?.title ?? TASK_DETAIL_TAB_FALLBACK_TITLE, CheckCircle2)

  if (!task) {
    return (
      <div className="h-full flex items-center justify-center text-muted-foreground">
        <span>{TASK_DETAIL_TAB_EMPTY_MESSAGE}</span>
      </div>
    )
  }

  const statusConfig = STATUS_CONFIG[task.status]
  const disc = disciplines.find(d => d.name === task.discipline)
  const stripCrop = disc?.imagePath ? disc?.crops?.strip : undefined
  const faceCrop = disc?.crops?.face
  const eyelinePercent = faceCrop
    ? Math.round((faceCrop.y + faceCrop.h / 2) * 100)
    : TASK_DETAIL_TAB_FALLBACK_EYELINE_PERCENT

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
