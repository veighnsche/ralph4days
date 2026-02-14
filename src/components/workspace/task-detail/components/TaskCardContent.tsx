import { AlertCircle, FileCode, Plus } from 'lucide-react'
import { Fragment, useState } from 'react'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import {
  createDisciplineDetailTab,
  createSubsystemDetailTab,
  createTaskDetailTab,
  useWorkspaceTabContext
} from '@/components/workspace/tabs'
import { STATUS_CONFIG } from '@/constants/prd'
import { useInvoke, useInvokeMutation } from '@/hooks/api'
import { useDisciplines } from '@/hooks/disciplines'
import { buildUpdateArgsFromTask, type UpdateTaskVariables } from '@/hooks/tasks/updateTaskMutation'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'
import type { SubsystemData, Task } from '@/types/generated'
import { TaskIdDisplay } from '../../../prd/TaskIdDisplay'
import { AcceptanceCriterionItem } from './AcceptanceCriterionItem'
import { addAcceptanceCriterion, toggleAcceptanceCriterion, updateAcceptanceCriterionText } from './acceptanceCriteria'

const EMPTY_SUBSYSTEMS: SubsystemData[] = []

export function TaskCardContent({ task }: { task: Task }) {
  const [pendingAutoEditCriteria, setPendingAutoEditCriteria] = useState<string[] | null>(null)
  const currentTab = useWorkspaceTabContext()
  const openTabAfter = useWorkspaceStore(s => s.openTabAfter)
  const { disciplines } = useDisciplines()
  const { data: subsystemsData } = useInvoke<SubsystemData[]>('subsystems_list', undefined, {
    staleTime: 5 * 60 * 1000
  })
  const subsystems = subsystemsData ?? EMPTY_SUBSYSTEMS
  const subsystemId = subsystems.find(subsystem => subsystem.name === task.subsystem)?.id
  const disciplineId = disciplines.find(discipline => discipline.name === task.discipline)?.id
  const openRelatedTabAfterCurrent = (tab: ReturnType<typeof createTaskDetailTab>) => {
    openTabAfter(currentTab.id, tab)
  }
  const updateTaskMutation = useInvokeMutation<UpdateTaskVariables, Task>('tasks_update', {
    queryDomain: 'workspace',
    invalidateKeys: [['tasks_get', { id: task.id }], ['tasks_list_items']]
  })

  const submitCriteriaUpdate = (nextCriteria: string[]) => {
    updateTaskMutation.mutate(buildUpdateArgsFromTask(task, { acceptanceCriteria: nextCriteria }))
  }

  const handleCriterionToggle = (criterionIndex: number) => {
    if (task.status === 'done') return

    const nextCriteria = toggleAcceptanceCriterion(task.acceptanceCriteria, criterionIndex)
    submitCriteriaUpdate(nextCriteria)
  }

  const handleCriterionTextSave = (criterionIndex: number, nextText: string) => {
    if (task.status === 'done') return

    const nextCriteria = updateAcceptanceCriterionText(task.acceptanceCriteria, criterionIndex, nextText)
    submitCriteriaUpdate(nextCriteria)
  }

  const handleCriterionAdd = () => {
    if (task.status === 'done') return

    const nextCriteria = addAcceptanceCriterion(task.acceptanceCriteria)
    setPendingAutoEditCriteria(nextCriteria)
    submitCriteriaUpdate(nextCriteria)
  }

  const shouldAutoEditNewCriterion =
    pendingAutoEditCriteria !== null &&
    pendingAutoEditCriteria.length === task.acceptanceCriteria.length &&
    pendingAutoEditCriteria.every((criterion, criterionIndex) => criterion === task.acceptanceCriteria[criterionIndex])

  const sections: Array<{ id: string; node: React.ReactNode }> = []

  sections.push({
    id: 'body',
    node: (
      <div className="px-6 space-y-3">
        <div className="space-y-1.5">
          <TaskIdDisplay
            task={task}
            variant="full"
            onTaskIdClick={() => openRelatedTabAfterCurrent(createTaskDetailTab(task.id))}
            onSubsystemClick={
              subsystemId != null ? () => openRelatedTabAfterCurrent(createSubsystemDetailTab(subsystemId)) : undefined
            }
            onDisciplineClick={
              disciplineId != null
                ? () => openRelatedTabAfterCurrent(createDisciplineDetailTab(disciplineId))
                : undefined
            }
          />
          <h1 className="text-xl font-semibold leading-tight">{task.title}</h1>
        </div>

        {task.status === 'blocked' && task.hints && (
          <div
            className="flex items-start gap-3 rounded-md px-3 py-2.5 text-sm"
            style={{
              backgroundColor: STATUS_CONFIG.blocked.bgColor,
              color: STATUS_CONFIG.blocked.color
            }}>
            <AlertCircle className="h-4 w-4 mt-0.5 flex-shrink-0" />
            <div>
              <span className="font-medium">Blocked — </span>
              {task.hints}
            </div>
          </div>
        )}

        {task.description && (
          <>
            <h2 className="text-sm font-medium text-muted-foreground">Description</h2>
            <p className="text-sm leading-relaxed whitespace-pre-wrap">{task.description}</p>
          </>
        )}
        {task.hints && (
          <div className="border-l-2 border-muted-foreground/20 pl-3">
            <p className="text-sm text-muted-foreground leading-relaxed whitespace-pre-wrap">{task.hints}</p>
          </div>
        )}
      </div>
    )
  })

  sections.push({
    id: 'criteria',
    node: (
      <div className="group/criteria px-6 space-y-2">
        <div className="flex items-center gap-1">
          <h2 className="text-sm font-medium text-muted-foreground">Acceptance Criteria</h2>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={handleCriterionAdd}
            aria-label="Add acceptance criterion"
            disabled={task.status === 'done' || updateTaskMutation.isPending}
            className="h-5 w-5 p-0 text-muted-foreground opacity-0 pointer-events-none transition-opacity group-hover/criteria:opacity-100 group-hover/criteria:pointer-events-auto focus-visible:opacity-100 focus-visible:pointer-events-auto disabled:opacity-40 disabled:pointer-events-none">
            <Plus className="h-3 w-3" />
          </Button>
        </div>
        {task.acceptanceCriteria.length > 0 && (
          <ul className="space-y-1.5">
            {task.acceptanceCriteria.map((criterion, criterionIndex) => (
              <AcceptanceCriterionItem
                key={`${criterion}-${criterionIndex.toString()}`}
                criterion={criterion}
                criterionIndex={criterionIndex}
                isTaskDone={task.status === 'done'}
                isPending={updateTaskMutation.isPending}
                onToggle={handleCriterionToggle}
                onSaveText={handleCriterionTextSave}
                autoStartEditing={shouldAutoEditNewCriterion && criterionIndex === 0}
                onAutoStartEditConsumed={() => setPendingAutoEditCriteria(null)}
              />
            ))}
          </ul>
        )}
      </div>
    )
  })

  if (task.contextFiles.length > 0 || task.outputArtifacts.length > 0) {
    sections.push({
      id: 'files',
      node: (
        <div className="px-6 space-y-2">
          <h2 className="text-sm font-medium text-muted-foreground">Files</h2>
          <div className="space-y-1.5">
            {task.contextFiles.length > 0 && (
              <div className="flex flex-wrap items-center gap-1.5">
                <span className="text-xs text-muted-foreground">In:</span>
                {task.contextFiles.map(file => (
                  <Badge key={file} variant="outline" className="text-xs font-mono px-2 py-0.5 h-5 gap-1">
                    <FileCode className="h-3 w-3 text-muted-foreground" />
                    {file}
                  </Badge>
                ))}
              </div>
            )}
            {task.outputArtifacts.length > 0 && (
              <div className="flex flex-wrap items-center gap-1.5">
                <span className="text-xs text-muted-foreground">Out:</span>
                {task.outputArtifacts.map(artifact => (
                  <Badge key={artifact} variant="outline" className="text-xs font-mono px-2 py-0.5 h-5 gap-1">
                    <FileCode className="h-3 w-3 text-muted-foreground" />
                    {artifact}
                  </Badge>
                ))}
              </div>
            )}
          </div>
        </div>
      )
    })
  }

  return (
    <div className="space-y-6">
      {sections.map(({ id, node }, index) => (
        <Fragment key={id}>
          {index > 0 && <Separator />}
          {node}
        </Fragment>
      ))}
    </div>
  )
}
