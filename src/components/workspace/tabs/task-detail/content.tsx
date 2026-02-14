import { CheckCircle2 } from 'lucide-react'
import { TaskPriorityCorner } from '@/components/shared'
import { CroppedImage } from '@/components/ui/cropped-image'
import { STATUS_CONFIG } from '@/constants/prd'
import { buildInvokeQueryKey, useInvoke, useInvokeMutation } from '@/hooks/api'
import { useDisciplines } from '@/hooks/disciplines'
import { usePRDData } from '@/hooks/tasks'
import {
  patchTaskInTaskDetailCache,
  patchTaskInTaskDetailCacheOptimistically,
  patchTaskListItemInTaskListCache,
  patchTaskListItemInTaskListCacheOptimistically
} from '@/hooks/tasks/taskCache'
import { buildOptimisticTaskFromUpdateTask, type UpdateTaskVariables } from '@/hooks/tasks/updateTaskMutation'
import { useTabMeta } from '@/hooks/workspace'
import { computeInferredStatus } from '@/lib/taskStatus'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import type { Task, TaskListItem } from '@/types/generated'
import { DetailPageLayout } from '../../DetailPageLayout'
import { CommentsSection } from '../../task-detail'
import { TaskCardContent } from '../../task-detail/components/TaskCardContent'
import { TaskSidebar } from '../../task-detail/components/TaskSidebar'
import {
  TASK_DETAIL_TAB_EMPTY_MESSAGE,
  TASK_DETAIL_TAB_FALLBACK_EYELINE_PERCENT,
  TASK_DETAIL_TAB_FALLBACK_TITLE
} from './constants'
import type { TaskDetailTabParams } from './schema'

export function TaskDetailTabContent({ tab, params }: { tab: WorkspaceTab; params: TaskDetailTabParams }) {
  const { entityId } = params

  const { tasks: taskListItems } = usePRDData('workspace')
  const {
    data: task,
    isLoading: taskLoading,
    error: taskError
  } = useInvoke<Task>('get_task', entityId != null ? { id: entityId } : undefined, {
    queryDomain: 'workspace',
    enabled: entityId != null
  })
  const { disciplines } = useDisciplines('workspace')

  useTabMeta(tab.id, task?.title ?? TASK_DETAIL_TAB_FALLBACK_TITLE, CheckCircle2)

  const updateTaskMutation = useInvokeMutation<UpdateTaskVariables, Task>('update_task', {
    queryDomain: 'workspace',
    optimisticUpdate: ({ queryClient, variables, queryDomain }) => {
      if (!task) throw new Error('[task-detail] update_task optimisticUpdate called without task loaded')

      const optimisticTask = buildOptimisticTaskFromUpdateTask(task, variables.params)
      const rollbackDetail = patchTaskInTaskDetailCacheOptimistically(queryClient, optimisticTask, queryDomain)

      const listQueryKey = buildInvokeQueryKey('get_task_list_items', undefined, queryDomain)
      const listItems = queryClient.getQueryData<TaskListItem[]>(listQueryKey)
      const listItem = listItems?.find(item => item.id === task.id)
      const rollbackList = listItem
        ? patchTaskListItemInTaskListCacheOptimistically(
            queryClient,
            { ...listItem, priority: optimisticTask.priority },
            queryDomain
          )
        : undefined

      return () => {
        rollbackList?.()
        rollbackDetail()
      }
    },
    updateCache: ({ queryClient, data, queryDomain }) => {
      patchTaskInTaskDetailCache(queryClient, data, queryDomain)

      const listQueryKey = buildInvokeQueryKey('get_task_list_items', undefined, queryDomain)
      const listItems = queryClient.getQueryData<TaskListItem[]>(listQueryKey)
      const listItem = listItems?.find(item => item.id === data.id)
      if (!listItem) return

      if (listItem.priority === data.priority) return
      patchTaskListItemInTaskListCache(queryClient, { ...listItem, priority: data.priority }, queryDomain)
    }
  })

  if (entityId == null) {
    return (
      <div className="h-full flex items-center justify-center text-muted-foreground">
        <span>{TASK_DETAIL_TAB_EMPTY_MESSAGE}</span>
      </div>
    )
  }

  if (taskLoading) {
    return (
      <div className="h-full flex items-center justify-center text-muted-foreground">
        <span>Loading task...</span>
      </div>
    )
  }

  if (taskError) {
    return (
      <div className="h-full flex items-center justify-center text-muted-foreground">
        <span>{`Failed to load task: ${taskError.message}`}</span>
      </div>
    )
  }

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

  const cyclePriority = () => {
    const current = task.priority
    const next = current === 'low' ? 'medium' : current === 'medium' ? 'high' : current === 'high' ? undefined : 'low'

    updateTaskMutation.mutate({
      params: {
        id: task.id,
        subsystem: task.subsystem,
        discipline: task.discipline,
        title: task.title,
        description: task.description,
        priority: next,
        tags: task.tags,
        depends_on: task.dependsOn,
        acceptance_criteria: task.acceptanceCriteria,
        context_files: task.contextFiles,
        output_artifacts: task.outputArtifacts,
        hints: task.hints,
        estimated_turns: task.estimatedTurns,
        provenance: task.provenance,
        agent: task.agent,
        model: task.model,
        effort: task.effort,
        thinking: task.thinking
      }
    })
  }

  return (
    <DetailPageLayout
      accentColor={statusConfig.color}
      cardOverlay={
        <TaskPriorityCorner
          priority={task.priority}
          size="md"
          className="top-4 right-4"
          showUnset
          disabled={updateTaskMutation.isPending}
          onClick={cyclePriority}
        />
      }
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
      sidebar={<TaskSidebar task={task} inferredStatus={computeInferredStatus(task, taskListItems ?? [])} />}>
      <CommentsSection task={task} />
    </DetailPageLayout>
  )
}
