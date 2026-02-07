import { CheckCircle2 } from 'lucide-react'
import { Card } from '@/components/ui/card'
import { ScrollArea } from '@/components/ui/scroll-area'
import { STATUS_CONFIG } from '@/constants/prd'
import { usePRDData } from '@/hooks/usePRDData'
import { useTabMeta } from '@/hooks/useTabMeta'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import type { Task } from '@/types/prd'
import { CommentsSection } from './task-detail/CommentsSection'
import { TaskCardContent } from './task-detail/TaskCardContent'
import { TaskSidebar } from './task-detail/TaskSidebar'

export function TaskDetailTabContent({ tab }: { tab: WorkspaceTab }) {
  const entityId = tab.data?.entityId as number | undefined
  const snapshotTask = tab.data?.entity as Task | undefined

  const { tasks } = usePRDData()
  const task = (entityId != null ? tasks?.find(t => t.id === entityId) : undefined) ?? snapshotTask

  useTabMeta(tab.id, task?.title ?? 'Task Detail', CheckCircle2)

  if (!task) {
    return (
      <div className="h-full flex items-center justify-center text-muted-foreground">
        <span>Task not found</span>
      </div>
    )
  }

  const statusConfig = STATUS_CONFIG[task.status]

  return (
    <div
      className="h-full px-3 relative"
      style={{
        background: `repeating-linear-gradient(
        45deg,
        transparent,
        transparent 10px,
        ${statusConfig.color}15 10px,
        ${statusConfig.color}15 20px
      )`
      }}>
      <ScrollArea className="h-full">
        <div className="py-3 space-y-3">
          {/* Task Card */}
          <Card className="shadow-sm flex flex-row gap-0 py-0">
            {/* Main Content */}
            <div className="flex-1 min-w-0 py-4">
              <TaskCardContent task={task} />
            </div>

            {/* Properties Sidebar */}
            <div className="w-56 flex-shrink-0 border-l">
              <TaskSidebar task={task} />
            </div>
          </Card>

          {/* Comments Section */}
          <CommentsSection task={task} />
        </div>
      </ScrollArea>
    </div>
  )
}
