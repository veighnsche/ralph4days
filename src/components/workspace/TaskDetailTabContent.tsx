import { CheckCircle2 } from 'lucide-react'
import { CroppedImage } from '@/components/ui/cropped-image'
import { STATUS_CONFIG } from '@/constants/prd'
import { useDisciplines } from '@/hooks/disciplines'
import { usePRDData } from '@/hooks/tasks'
import { useTabMeta } from '@/hooks/workspace'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import type { Task } from '@/types/generated'
import { DetailPageLayout } from './DetailPageLayout'
import { CommentsSection } from './task-detail/CommentsSection'
import { TaskCardContent } from './task-detail/TaskCardContent'
import { TaskSidebar } from './task-detail/TaskSidebar'

export function TaskDetailTabContent({ tab }: { tab: WorkspaceTab }) {
  const entityId = tab.data?.entityId as number | undefined
  const snapshotTask = tab.data?.entity as Task | undefined

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
      sidebar={<TaskSidebar task={task} />}>
      <CommentsSection task={task} />
    </DetailPageLayout>
  )
}
