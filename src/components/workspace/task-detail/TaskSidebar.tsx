import { Bot, Check, Cog, Play, Radio, User } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { QUERY_KEYS } from '@/constants/cache'
import { INFERRED_STATUS_CONFIG, PRIORITY_CONFIG, STATUS_CONFIG } from '@/constants/prd'
import { type SignalVerb, VERB_CONFIG } from '@/constants/signals'
import { useInvokeMutation } from '@/hooks/api'
import { formatDate } from '@/lib/formatDate'
import { resolveIcon } from '@/lib/iconRegistry'
import type { InferredTaskStatus } from '@/lib/taskStatus'
import { shouldShowInferredStatus } from '@/lib/taskStatus'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'
import type { Task, TaskComment } from '@/types/generated'
import { PropertyRow } from '../PropertyRow'
import { TerminalTabContent } from '../TerminalTabContent'

const PROVENANCE_CONFIG = {
  agent: { label: 'Agent', icon: Bot },
  human: { label: 'Human', icon: User },
  system: { label: 'System', icon: Cog }
} as const

function buildSignalSummaryText(signals: TaskComment[]): string | null {
  if (signals.length === 0) return null
  const counts: Record<string, number> = {}
  const pendingAsks = signals.filter(s => s.signal_verb === 'ask' && !s.signal_answered).length
  for (const s of signals) {
    if (s.signal_verb === 'flag') counts.flags = (counts.flags ?? 0) + 1
    if (s.signal_verb === 'learned') counts.learned = (counts.learned ?? 0) + 1
  }
  const parts: string[] = []
  if (counts.flags) parts.push(`${counts.flags} flag${counts.flags > 1 ? 's' : ''}`)
  if (pendingAsks) parts.push(`${pendingAsks} ask pending`)
  if (counts.learned) parts.push(`${counts.learned} learned`)
  return parts.length > 0 ? parts.join(' · ') : null
}

function getLastClosingVerb(signals: TaskComment[]): SignalVerb | null {
  for (let i = signals.length - 1; i >= 0; i--) {
    const verb = signals[i].signal_verb
    if (verb === 'done' || verb === 'partial' || verb === 'stuck') return verb as SignalVerb
  }
  return null
}

export function TaskSidebar({ task, inferredStatus }: { task: Task; inferredStatus: InferredTaskStatus }) {
  const signals = (task.comments ?? []).filter(c => c.signal_verb != null)
  const statusConfig = STATUS_CONFIG[task.status]
  const StatusIcon = statusConfig.icon
  const priorityConfig = task.priority ? PRIORITY_CONFIG[task.priority] : null
  const DisciplineIcon = resolveIcon(task.disciplineIcon)
  const openTab = useWorkspaceStore(state => state.openTab)
  const isDraftAgent = task.status === 'draft' && task.provenance === 'agent'

  const approveMutation = useInvokeMutation<{ id: number; status: string }>('set_task_status', {
    invalidateKeys: QUERY_KEYS.TASKS
  })

  const handleApprove = () => {
    approveMutation.mutate({ id: task.id, status: 'pending' })
  }

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

      {signals.length > 0 &&
        (() => {
          const sessions = new Set(signals.map(s => s.session_id))
          const summaryText = buildSignalSummaryText(signals)
          const lastClosing = getLastClosingVerb(signals)
          const lastClosingConfig = lastClosing ? VERB_CONFIG[lastClosing] : null
          const LastClosingIcon = lastClosingConfig?.icon

          return (
            <>
              <PropertyRow label="Sessions">
                <div className="flex flex-col gap-1">
                  <div className="flex items-center gap-1.5">
                    <Radio className="h-3 w-3 text-muted-foreground" />
                    <span className="text-sm">{sessions.size}</span>
                    {lastClosingConfig && LastClosingIcon && (
                      <>
                        <span className="text-xs text-muted-foreground">·</span>
                        <LastClosingIcon className="h-3 w-3" style={{ color: lastClosingConfig.color }} />
                        <span className="text-xs" style={{ color: lastClosingConfig.color }}>
                          {lastClosingConfig.label}
                        </span>
                      </>
                    )}
                  </div>
                  {summaryText && <span className="text-xs text-muted-foreground pl-4.5">{summaryText}</span>}
                </div>
              </PropertyRow>
              <Separator bleed="md" className="my-2" />
            </>
          )
        })()}

      <div className="pt-2 pb-1 space-y-1.5">
        <Button onClick={handleExecute} size="sm" className="w-full h-8" disabled={task.status === 'done'}>
          <Play className="h-3.5 w-3.5 mr-1.5" />
          Execute Task
        </Button>
        {isDraftAgent && (
          <Button
            onClick={handleApprove}
            variant="outline"
            size="sm"
            className="w-full h-8"
            disabled={approveMutation.isPending}>
            <Check className="h-3.5 w-3.5 mr-1.5" />
            {approveMutation.isPending ? 'Approving...' : 'Approve Task'}
          </Button>
        )}
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
