import { Bot, Check, Cog, Play, Radio, Sparkles, User, WandSparkles } from 'lucide-react'
import type { ReactNode } from 'react'
import { InlineError } from '@/components/shared'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { QUERY_KEYS } from '@/constants/cache'
import { INFERRED_STATUS_CONFIG, PRIORITY_CONFIG, STATUS_CONFIG } from '@/constants/prd'
import { type SignalVerb, VERB_CONFIG } from '@/constants/signals'
import { useInvokeMutation } from '@/hooks/api'
import { formatDate } from '@/lib/formatDate'
import type { InferredTaskStatus } from '@/lib/taskStatus'
import { shouldShowInferredStatus } from '@/lib/taskStatus'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'
import type { Task, TaskSignal } from '@/types/generated'
import { PropertyRow } from '../../PropertyRow'
import { createTerminalTab } from '../../tabs'
import { type LaunchSource, useResolvedTaskLaunch } from '../hooks/useResolvedTaskLaunch'
import { DisciplineSelect } from './DisciplineSelect'

const PROVENANCE_CONFIG = {
  agent: { label: 'Agent', icon: Bot },
  human: { label: 'Human', icon: User },
  system: { label: 'System', icon: Cog }
} as const

function sourceIcon(source: LaunchSource) {
  if (source === 'task') {
    return (
      <span title="Task override">
        <Sparkles className="h-3 w-3 text-muted-foreground" />
      </span>
    )
  }
  if (source === 'default') {
    return (
      <span title="Model default fallback">
        <WandSparkles className="h-3 w-3 text-muted-foreground" />
      </span>
    )
  }
  return null
}

function buildSignalSummaryText(signals: TaskSignal[]): string | null {
  if (signals.length === 0) return null
  const counts: Record<string, number> = {}
  const pendingAsks = signals.filter(signal => signal.signal_verb === 'ask' && !signal.answer).length
  for (const signal of signals) {
    if (signal.signal_verb === 'flag') counts.flags = (counts.flags ?? 0) + 1
    if (signal.signal_verb === 'learned') counts.learned = (counts.learned ?? 0) + 1
  }
  const parts: string[] = []
  if (counts.flags) parts.push(`${counts.flags} flag${counts.flags > 1 ? 's' : ''}`)
  if (pendingAsks) parts.push(`${pendingAsks} ask pending`)
  if (counts.learned) parts.push(`${counts.learned} learned`)
  return parts.length > 0 ? parts.join(' · ') : null
}

function getLastClosingVerb(signals: TaskSignal[]): SignalVerb | null {
  for (let i = signals.length - 1; i >= 0; i--) {
    const verb = signals[i].signal_verb
    if (verb === 'done' || verb === 'partial' || verb === 'stuck') return verb
  }
  return null
}

function renderDependsOnBadges(dependsOn: number[]): ReactNode {
  if (dependsOn.length === 0) return null
  return (
    <div className="flex flex-wrap gap-1 items-center">
      <span className="text-xs text-muted-foreground">Depends on:</span>
      {dependsOn.map(depId => (
        <Badge key={depId} variant="outline" className="text-xs font-mono px-1.5 py-0 h-4">
          #{depId.toString().padStart(3, '0')}
        </Badge>
      ))}
    </div>
  )
}

function renderProvenanceRow(provenance: Task['provenance']) {
  if (!provenance) return null
  const { label, icon: ProvIcon } = PROVENANCE_CONFIG[provenance]
  return (
    <div className="flex items-center gap-1.5">
      <ProvIcon className="h-3 w-3 text-muted-foreground" />
      <span className="text-xs text-muted-foreground">{label}</span>
    </div>
  )
}

function buildStatusSection(
  task: Task,
  statusConfig: (typeof STATUS_CONFIG)[Task['status']],
  inferredStatus: InferredTaskStatus,
  dependsOn: number[]
) {
  const showInferredStatus = shouldShowInferredStatus(task.status, inferredStatus)
  const inferredConfig = showInferredStatus ? INFERRED_STATUS_CONFIG[inferredStatus] : null
  const dependsOnBadges = renderDependsOnBadges(dependsOn)

  return (
    <PropertyRow label="Status">
      <div className="flex flex-col gap-1.5">
        <div className="flex items-center gap-1.5">
          <statusConfig.icon className="h-3.5 w-3.5" style={{ color: statusConfig.color }} />
          <span className="text-sm" style={{ color: statusConfig.color }}>
            {statusConfig.label}
          </span>
          {task.estimatedTurns != null && (
            <span className="text-xs text-muted-foreground ml-1">· ~{task.estimatedTurns} turns</span>
          )}
        </div>
        {inferredConfig ? (
          <div className="flex items-start gap-1.5 pl-5">
            <span className="text-xs text-muted-foreground mt-0.5">→</span>
            <div className="flex flex-col gap-1">
              <div className="flex items-center gap-1.5">
                <inferredConfig.icon className="h-3 w-3" style={{ color: inferredConfig.color }} />
                <span className="text-xs font-medium" style={{ color: inferredConfig.color }}>
                  {inferredConfig.label}
                </span>
              </div>
              {dependsOnBadges}
            </div>
          </div>
        ) : (
          <div className="pl-5">{dependsOnBadges}</div>
        )}
      </div>
    </PropertyRow>
  )
}

function buildSessionsSection(signals: TaskSignal[]) {
  const sessions = new Set(signals.map(signal => signal.session_id))
  const summary = buildSignalSummaryText(signals)
  const lastClosing = getLastClosingVerb(signals)
  const lastClosingConfig = lastClosing ? VERB_CONFIG[lastClosing] : null
  const LastClosingIcon = lastClosingConfig?.icon

  const sections: ReactNode[] = [
    <PropertyRow key="sessions" label="Sessions">
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
        {summary && <span className="text-xs text-muted-foreground pl-4.5">{summary}</span>}
      </div>
    </PropertyRow>,
    <Separator key="sessions-sep" bleed="md" className="my-2" />
  ]

  return sections
}

function buildCreatedSection(task: Task) {
  const createdProvenance = renderProvenanceRow(task.provenance)
  const rows: ReactNode[] = []
  if (task.created) {
    rows.push(
      <PropertyRow key="created" label="Created">
        <div className="flex items-center gap-1.5">
          <span className="text-xs text-muted-foreground">{formatDate(task.created)}</span>
          {createdProvenance && (
            <>
              <span className="text-xs text-muted-foreground">·</span>
              {createdProvenance}
            </>
          )}
        </div>
      </PropertyRow>
    )
  }
  if (!task.created && createdProvenance) {
    rows.push(
      <PropertyRow key="created-by" label="Created by">
        {createdProvenance}
      </PropertyRow>
    )
  }
  if (task.updated) {
    rows.push(
      <PropertyRow key="updated" label="Updated">
        <span className="text-xs text-muted-foreground">{formatDate(task.updated)}</span>
      </PropertyRow>
    )
  }
  if (task.completed) {
    rows.push(
      <PropertyRow key="completed" label="Completed">
        <span className="text-xs" style={{ color: STATUS_CONFIG.done.color }}>
          {formatDate(task.completed)}
        </span>
      </PropertyRow>
    )
  }
  return rows
}

// biome-ignore lint/complexity/noExcessiveCognitiveComplexity: Layout assembly requires many conditional sections in a single component.
export function TaskSidebar({ task, inferredStatus }: { task: Task; inferredStatus: InferredTaskStatus }) {
  const { id: taskId, status, subsystemDisplayName, tags, subsystem, discipline, title, description } = task
  const signals = (task.signals ?? []).filter(signal => signal.signal_verb != null)
  const shouldShowSignals = signals.length > 0
  const statusConfig = STATUS_CONFIG[status]
  const priorityConfig = task.priority ? PRIORITY_CONFIG[task.priority] : null
  const openTab = useWorkspaceStore(state => state.openTab)
  const isDraftAgent = status === 'draft' && task.provenance === 'agent'
  const dependsOn = task.dependsOn ?? []

  const {
    resolvedAgent,
    resolvedModel,
    resolvedEffort,
    resolvedThinking,
    resolvedModelSupportsEffort,
    agentSource,
    modelSource,
    effortSource,
    thinkingSource
  } = useResolvedTaskLaunch(task)

  const approveMutation = useInvokeMutation<{ id: number; status: string }>('set_task_status', {
    invalidateKeys: QUERY_KEYS.TASKS
  })
  const updateTaskMutation = useInvokeMutation<
    {
      params: {
        id: number
        subsystem: string
        discipline: string
        title: string
        description?: string
        priority?: Task['priority']
        tags: string[]
        depends_on: number[]
        acceptance_criteria: string[]
        context_files: string[]
        output_artifacts: string[]
        hints?: string
        estimated_turns?: number
        provenance?: Task['provenance']
        agent?: string
        model?: string
        effort?: string
        thinking?: boolean
      }
    },
    void
  >('update_task', {
    invalidateKeys: QUERY_KEYS.TASKS
  })

  const handleApprove = () => {
    approveMutation.mutate({ id: task.id, status: 'pending' })
  }

  const handleExecute = () => {
    openTab(
      createTerminalTab({
        taskId,
        title: `Task #${taskId.toString().padStart(3, '0')}`,
        agent: resolvedAgent ?? undefined,
        model: resolvedModel ?? undefined,
        effort: resolvedEffort ?? undefined,
        thinking: resolvedThinking ?? undefined
      })
    )
  }

  const handleDisciplineSelect = (disciplineName: string) =>
    disciplineName !== discipline &&
    updateTaskMutation.mutate({
      params: {
        id: taskId,
        subsystem,
        discipline: disciplineName,
        title,
        description,
        priority: task.priority,
        tags,
        depends_on: dependsOn,
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

  const sections: ReactNode[] = []
  sections.push(
    buildStatusSection(task, statusConfig, inferredStatus, dependsOn),
    <div key="run" className="pt-2 pb-1 space-y-1.5">
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
    </div>,
    <PropertyRow key="priority" label="Priority">
      {priorityConfig ? (
        <span className="text-sm" style={{ color: priorityConfig.color }}>
          {priorityConfig.label}
        </span>
      ) : (
        <span className="text-sm text-muted-foreground">None</span>
      )}
    </PropertyRow>,
    <Separator key="priority-sep" bleed="md" className="my-2" />,
    <PropertyRow key="subsystem" label="Subsystem">
      <span className="text-sm">{subsystemDisplayName}</span>
    </PropertyRow>,
    <PropertyRow key="discipline" label="Discipline">
      <DisciplineSelect value={discipline} onSelect={handleDisciplineSelect} disabled={updateTaskMutation.isPending} />
    </PropertyRow>,
    <InlineError
      key="discipline-error"
      error={updateTaskMutation.error}
      onDismiss={updateTaskMutation.reset}
      className="mt-1 mb-2"
    />,
    <PropertyRow key="launch" label="Launch">
      <div className="space-y-1 text-xs text-muted-foreground">
        <div className="flex items-center gap-1.5">
          <span>Agent:</span>
          <code className="font-mono bg-muted px-1.5 py-0.5 rounded">{resolvedAgent ?? 'unset'}</code>
          {sourceIcon(agentSource)}
        </div>
        <div className="flex items-center gap-1.5">
          <span>Model:</span>
          <code className="font-mono bg-muted px-1.5 py-0.5 rounded">{resolvedModel ?? 'No model set'}</code>
          {sourceIcon(modelSource)}
        </div>
        {resolvedModelSupportsEffort && (
          <div className="flex items-center gap-1.5">
            <span>Effort:</span>
            <code className="font-mono bg-muted px-1.5 py-0.5 rounded">{resolvedEffort ?? 'unset'}</code>
            {sourceIcon(effortSource)}
          </div>
        )}
        <div className="flex items-center gap-1.5">
          <span>Thinking:</span>
          <code className="font-mono bg-muted px-1.5 py-0.5 rounded">
            {resolvedThinking === undefined ? 'unset' : resolvedThinking ? 'on' : 'off'}
          </code>
          {sourceIcon(thinkingSource)}
        </div>
      </div>
    </PropertyRow>
  )

  sections.push(
    ...(shouldShowSignals ? buildSessionsSection(signals) : []),
    ...(task.tags && task.tags.length > 0
      ? [
          <Separator key="tags-sep" bleed="md" className="my-2" />,
          <PropertyRow key="tags" label="Tags">
            <div className="flex flex-wrap gap-1">
              {task.tags.map(tag => (
                <Badge key={tag} variant="secondary" className="text-xs px-1.5 py-0 h-5">
                  {tag}
                </Badge>
              ))}
            </div>
          </PropertyRow>
        ]
      : []),
    <Separator key="created-sep" bleed="md" className="my-2" />,
    ...buildCreatedSection(task)
  )

  return (
    <div className="relative h-full">
      <Button
        onClick={handleExecute}
        size="icon"
        className="absolute top-0 right-0 z-10 rounded-none rounded-bl-md"
        disabled={status === 'done'}
        aria-label="Execute Task">
        <Play className="h-3.5 w-3.5" />
      </Button>

      <div className="px-4 py-4 space-y-0.5 overflow-y-auto h-full">{sections}</div>
    </div>
  )
}
