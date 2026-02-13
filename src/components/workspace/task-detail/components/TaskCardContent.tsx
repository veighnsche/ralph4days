import { AlertCircle, CheckCircle2, FileCode } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { Separator } from '@/components/ui/separator'
import {
  createDisciplineDetailTab,
  createSubsystemDetailTab,
  createTaskDetailTab,
  useWorkspaceTabContext
} from '@/components/workspace/tabs'
import { STATUS_CONFIG } from '@/constants/prd'
import { useInvoke } from '@/hooks/api'
import { useDisciplines } from '@/hooks/disciplines'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'
import type { SubsystemData, Task } from '@/types/generated'
import { TaskIdDisplay } from '../../../prd/TaskIdDisplay'

export function TaskCardContent({ task }: { task: Task }) {
  const currentTab = useWorkspaceTabContext()
  const openTabAfter = useWorkspaceStore(s => s.openTabAfter)
  const { disciplines } = useDisciplines()
  const { data: subsystems = [] } = useInvoke<SubsystemData[]>('get_subsystems', undefined, {
    staleTime: 5 * 60 * 1000
  })
  const subsystemId = subsystems.find(subsystem => subsystem.name === task.subsystem)?.id
  const disciplineId = disciplines.find(discipline => discipline.name === task.discipline)?.id
  const openRelatedTabAfterCurrent = (tab: ReturnType<typeof createTaskDetailTab>) => {
    openTabAfter(currentTab.id, tab)
  }

  const sections: React.ReactNode[] = []

  sections.push(
    <div key="body" className="px-6 space-y-3">
      <div className="space-y-1.5">
        <TaskIdDisplay
          task={task}
          variant="full"
          onTaskIdClick={() => openRelatedTabAfterCurrent(createTaskDetailTab(task.id))}
          onSubsystemClick={
            subsystemId != null ? () => openRelatedTabAfterCurrent(createSubsystemDetailTab(subsystemId)) : undefined
          }
          onDisciplineClick={
            disciplineId != null ? () => openRelatedTabAfterCurrent(createDisciplineDetailTab(disciplineId)) : undefined
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

  if (task.acceptanceCriteria && task.acceptanceCriteria.length > 0) {
    sections.push(
      <div key="criteria" className="px-6 space-y-2">
        <h2 className="text-sm font-medium text-muted-foreground">Acceptance Criteria</h2>
        <ul className="space-y-1.5">
          {task.acceptanceCriteria.map(criterion => (
            <li key={criterion} className="flex items-start gap-2.5 text-sm">
              <div
                className="mt-1 w-4 h-4 rounded-sm border flex items-center justify-center flex-shrink-0"
                style={{
                  borderColor: task.status === 'done' ? STATUS_CONFIG.done.color : 'hsl(var(--border))',
                  backgroundColor: task.status === 'done' ? STATUS_CONFIG.done.bgColor : 'transparent'
                }}>
                {task.status === 'done' && (
                  <CheckCircle2 className="w-3 h-3" style={{ color: STATUS_CONFIG.done.color }} />
                )}
              </div>
              <span className={task.status === 'done' ? 'line-through text-muted-foreground' : ''}>{criterion}</span>
            </li>
          ))}
        </ul>
      </div>
    )
  }

  if (
    (task.contextFiles && task.contextFiles.length > 0) ||
    (task.outputArtifacts && task.outputArtifacts.length > 0)
  ) {
    sections.push(
      <div key="files" className="px-6 space-y-2">
        <h2 className="text-sm font-medium text-muted-foreground">Files</h2>
        <div className="space-y-1.5">
          {task.contextFiles && task.contextFiles.length > 0 && (
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
          {task.outputArtifacts && task.outputArtifacts.length > 0 && (
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
  }

  return (
    <div className="space-y-6">
      {sections.flatMap((section, i) => (i === 0 ? [section] : [<Separator key={`sep-${i}`} />, section]))}
    </div>
  )
}
