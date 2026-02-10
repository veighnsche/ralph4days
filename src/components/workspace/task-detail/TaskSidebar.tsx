import { Bot, Cog, Play, User } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { INFERRED_STATUS_CONFIG, PRIORITY_CONFIG, STATUS_CONFIG } from '@/constants/prd'
import { formatDate } from '@/lib/formatDate'
import { resolveIcon } from '@/lib/iconRegistry'
import type { InferredTaskStatus } from '@/lib/taskStatus'
import { shouldShowInferredStatus } from '@/lib/taskStatus'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'
import type { Task } from '@/types/generated'
import { PropertyRow } from '../PropertyRow'
import { TerminalTabContent } from '../TerminalTabContent'

const PROVENANCE_CONFIG = {
  agent: { label: 'Agent', icon: Bot },
  human: { label: 'Human', icon: User },
  system: { label: 'System', icon: Cog }
} as const

export function TaskSidebar({ task, inferredStatus }: { task: Task; inferredStatus: InferredTaskStatus }) {
  const statusConfig = STATUS_CONFIG[task.status]
  const StatusIcon = statusConfig.icon
  const priorityConfig = task.priority ? PRIORITY_CONFIG[task.priority] : null
  const DisciplineIcon = resolveIcon(task.disciplineIcon)
  const openTab = useWorkspaceStore(state => state.openTab)

  const handleExecute = () => {
    openTab({
      type: 'terminal',
      component: TerminalTabContent,
      title: `Task #${task.id.toString().padStart(3, '0')}`,
      closeable: true,
      data: {
        taskId: task.id
      }
    })
  }

  return (
    <div className="px-4 py-4 space-y-0.5 overflow-y-auto h-full">
      <PropertyRow label="Status">
        <div className="flex flex-col gap-1.5">
          <div className="flex items-center gap-1.5">
            <StatusIcon className="h-3.5 w-3.5" style={{ color: statusConfig.color }} />
            <span className="text-sm" style={{ color: statusConfig.color }}>
              {statusConfig.label}
            </span>
            {task.estimatedTurns != null && (
              <span className="text-xs text-muted-foreground ml-1">· ~{task.estimatedTurns} turns</span>
            )}
          </div>

          {shouldShowInferredStatus(task.status, inferredStatus) &&
            (() => {
              const inferredConfig = INFERRED_STATUS_CONFIG[inferredStatus]
              const InferredIcon = inferredConfig.icon
              const hasDeps = task.dependsOn && task.dependsOn.length > 0

              return (
                <div className="flex items-start gap-1.5 pl-5">
                  <span className="text-xs text-muted-foreground mt-0.5">→</span>
                  <div className="flex flex-col gap-1">
                    <div className="flex items-center gap-1.5">
                      <InferredIcon className="h-3 w-3" style={{ color: inferredConfig.color }} />
                      <span className="text-xs font-medium" style={{ color: inferredConfig.color }}>
                        {inferredConfig.label}
                      </span>
                    </div>
                    {hasDeps && (
                      <div className="flex flex-wrap gap-1 items-center">
                        <span className="text-xs text-muted-foreground">Depends on:</span>
                        {task.dependsOn?.map(depId => (
                          <Badge key={depId} variant="outline" className="text-xs font-mono px-1.5 py-0 h-4">
                            #{depId.toString().padStart(3, '0')}
                          </Badge>
                        ))}
                      </div>
                    )}
                  </div>
                </div>
              )
            })()}

          {!shouldShowInferredStatus(task.status, inferredStatus) && task.dependsOn && task.dependsOn.length > 0 && (
            <div className="flex flex-wrap gap-1 items-center pl-5">
              <span className="text-xs text-muted-foreground">Depends on:</span>
              {task.dependsOn.map(depId => (
                <Badge key={depId} variant="outline" className="text-xs font-mono px-1.5 py-0 h-4">
                  #{depId.toString().padStart(3, '0')}
                </Badge>
              ))}
            </div>
          )}
        </div>
      </PropertyRow>

      <div className="pt-2 pb-1">
        <Button onClick={handleExecute} size="sm" className="w-full h-8" disabled={task.status === 'done'}>
          <Play className="h-3.5 w-3.5 mr-1.5" />
          Execute Task
        </Button>
      </div>

      <PropertyRow label="Priority">
        {priorityConfig ? (
          <span className="text-sm" style={{ color: priorityConfig.color }}>
            {priorityConfig.label}
          </span>
        ) : (
          <span className="text-sm text-muted-foreground">None</span>
        )}
      </PropertyRow>

      <Separator bleed="md" className="my-2" />

      <PropertyRow label="Feature">
        <span className="text-sm">{task.featureDisplayName}</span>
      </PropertyRow>

      <PropertyRow label="Discipline">
        <div className="flex items-center gap-1.5">
          <DisciplineIcon className="h-3.5 w-3.5" style={{ color: task.disciplineColor }} />
          <span className="text-sm" style={{ color: task.disciplineColor }}>
            {task.disciplineDisplayName}
          </span>
        </div>
      </PropertyRow>

      {task.tags && task.tags.length > 0 && (
        <>
          <Separator bleed="md" className="my-2" />
          <PropertyRow label="Tags">
            <div className="flex flex-wrap gap-1">
              {task.tags.map(tag => (
                <Badge key={tag} variant="secondary" className="text-xs px-1.5 py-0 h-5">
                  {tag}
                </Badge>
              ))}
            </div>
          </PropertyRow>
        </>
      )}

      <Separator bleed="md" className="my-2" />
      {task.created && (
        <PropertyRow label="Created">
          <div className="flex items-center gap-1.5">
            <span className="text-xs text-muted-foreground">{formatDate(task.created)}</span>
            {task.provenance &&
              (() => {
                const prov = PROVENANCE_CONFIG[task.provenance]
                const ProvIcon = prov.icon
                return (
                  <>
                    <span className="text-xs text-muted-foreground">·</span>
                    <ProvIcon className="h-3 w-3 text-muted-foreground" />
                    <span className="text-xs text-muted-foreground">{prov.label}</span>
                  </>
                )
              })()}
          </div>
        </PropertyRow>
      )}
      {!task.created && task.provenance && (
        <PropertyRow label="Created by">
          {(() => {
            const prov = PROVENANCE_CONFIG[task.provenance]
            const ProvIcon = prov.icon
            return (
              <div className="flex items-center gap-1.5">
                <ProvIcon className="h-3 w-3 text-muted-foreground" />
                <span className="text-xs text-muted-foreground">{prov.label}</span>
              </div>
            )
          })()}
        </PropertyRow>
      )}
      {task.updated && (
        <PropertyRow label="Updated">
          <span className="text-xs text-muted-foreground">{formatDate(task.updated)}</span>
        </PropertyRow>
      )}
      {task.completed && (
        <PropertyRow label="Completed">
          <span className="text-xs" style={{ color: STATUS_CONFIG.done.color }}>
            {formatDate(task.completed)}
          </span>
        </PropertyRow>
      )}
    </div>
  )
}
