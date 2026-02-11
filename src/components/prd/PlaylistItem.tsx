import { Bot, CircleHelp, Cog, Flag, GitBranch, ListChecks, MessageSquare, User } from 'lucide-react'
import { memo } from 'react'
import { PriorityIcon, PriorityRadial } from '@/components/shared'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { INFERRED_STATUS_CONFIG, STATUS_CONFIG } from '@/constants/prd'
import { FLAG_SEVERITY_CONFIG, type FlagSeverity } from '@/constants/signals'
import type { DisciplineCropsData, Task, TaskSignalSummary } from '@/types/generated'
import { DisciplineHeadshot } from './DisciplineHeadshot'
import { DisciplineLabel } from './DisciplineLabel'

interface PlaylistItemProps {
  task: Task
  crops?: DisciplineCropsData
  unresolvedDeps?: number
  isNowPlaying?: boolean
  signalSummary?: TaskSignalSummary
  onClick: () => void
}

const PROVENANCE_ICONS = { agent: Bot, human: User, system: Cog } as const

function SignalIndicators({ summary }: { summary: TaskSignalSummary }) {
  const hasPendingAsks = summary.pendingAsks > 0
  const hasFlags = summary.flagCount > 0
  if (!(hasPendingAsks || hasFlags)) return null

  const flagColor = summary.maxFlagSeverity
    ? (FLAG_SEVERITY_CONFIG[summary.maxFlagSeverity as FlagSeverity]?.color ?? 'var(--priority-medium)')
    : 'var(--priority-medium)'

  return (
    <>
      {hasPendingAsks && (
        <div className="flex items-center gap-1" style={{ color: 'var(--secondary)' }}>
          <CircleHelp className="h-3 w-3" />
          <span className="text-xs">{summary.pendingAsks}</span>
        </div>
      )}
      {hasFlags && (
        <div className="flex items-center gap-1" style={{ color: flagColor }}>
          <Flag className="h-3 w-3" />
          <span className="text-xs">{summary.flagCount}</span>
        </div>
      )}
    </>
  )
}

function PlaylistItemIndicators({
  task,
  unresolvedDeps,
  signalSummary
}: {
  task: Task
  unresolvedDeps: number
  signalSummary?: TaskSignalSummary
}) {
  const totalDeps = task.dependsOn?.length ?? 0
  const hasSignals = signalSummary && (signalSummary.pendingAsks > 0 || signalSummary.flagCount > 0)
  const hasAny =
    (task.comments?.length ?? 0) > 0 ||
    totalDeps > 0 ||
    (task.acceptanceCriteria?.length ?? 0) > 0 ||
    task.priority ||
    hasSignals

  if (!hasAny) return null

  return (
    <div className="flex items-center gap-2 text-muted-foreground">
      {signalSummary && <SignalIndicators summary={signalSummary} />}
      {task.comments && task.comments.length > 0 && (
        <div className="flex items-center gap-1">
          <MessageSquare className="h-3 w-3" />
          <span className="text-xs">{task.comments.length}</span>
        </div>
      )}
      {totalDeps > 0 && (
        <div
          className="flex items-center gap-1"
          style={unresolvedDeps > 0 ? { color: INFERRED_STATUS_CONFIG.waiting_on_deps.color } : undefined}>
          <GitBranch className="h-3 w-3" />
          <span className="text-xs">
            {unresolvedDeps}/{totalDeps}
          </span>
        </div>
      )}
      {task.acceptanceCriteria && task.acceptanceCriteria.length > 0 && (
        <div className="flex items-center gap-1">
          <ListChecks className="h-3 w-3" />
          <span className="text-xs">{task.acceptanceCriteria.length}</span>
        </div>
      )}
      {task.priority && <PriorityIcon priority={task.priority} />}
    </div>
  )
}

function ProvenanceIcon({ provenance }: { provenance: string }) {
  const Icon = PROVENANCE_ICONS[provenance as keyof typeof PROVENANCE_ICONS] ?? Cog
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Icon className="h-3.5 w-3.5 text-muted-foreground flex-shrink-0 cursor-help" />
      </TooltipTrigger>
      <TooltipContent>Created by {provenance}</TooltipContent>
    </Tooltip>
  )
}

export const PlaylistItem = memo(function PlaylistItem({
  task,
  crops,
  unresolvedDeps = 0,
  isNowPlaying = false,
  signalSummary,
  onClick
}: PlaylistItemProps) {
  const statusConfig = STATUS_CONFIG[task.status]
  const hasHeadshot = !!crops?.face

  return (
    <button
      type="button"
      className="grid grid-cols-[auto_1fr_auto] gap-x-2.5 border-l-4 relative overflow-hidden cursor-pointer rounded-md py-3 px-4 transition-all duration-200 hover:opacity-80 min-h-22 text-left w-full"
      style={{
        borderLeftColor: statusConfig.color,
        backgroundColor: statusConfig.bgColor,
        opacity: task.status === 'done' || task.status === 'skipped' || task.status === 'draft' ? 0.5 : 1
      }}
      onClick={onClick}>
      {task.priority && <PriorityRadial priority={task.priority} />}

      {hasHeadshot && (
        <DisciplineHeadshot disciplineName={task.discipline} faceCrop={crops?.face ?? { x: 0, y: 0, w: 1, h: 1 }} />
      )}

      {/* Col 1: Task ID */}
      <div
        className={`row-span-full self-start relative z-10 flex flex-col items-start leading-tight font-mono ${hasHeadshot ? 'ml-22' : ''}`}>
        <span className="text-xs text-muted-foreground">{task.featureAcronym}</span>
        <DisciplineLabel acronym={task.disciplineAcronym} color={task.disciplineColor} />
        <span className="text-xs text-muted-foreground">
          {task.id > 999 ? task.id.toString() : `#${task.id.toString().padStart(3, '0')}`}
        </span>
      </div>

      {/* Col 2: Title */}
      <div className="min-w-0 relative z-10">
        <div
          className={`flex items-center gap-1.5 truncate ${isNowPlaying ? 'text-base font-medium' : 'text-sm font-medium'}`}
          style={isNowPlaying ? { color: statusConfig.color } : undefined}>
          {task.provenance && <ProvenanceIcon provenance={task.provenance} />}
          <span className="truncate">{task.title}</span>
          {isNowPlaying && <span className="ml-1 text-xs opacity-70 flex-shrink-0">[NOW PLAYING]</span>}
        </div>
        {task.description && <p className="text-muted-foreground text-sm line-clamp-2">{task.description}</p>}
      </div>

      {/* Col 3: Indicators + Tags */}
      <div className="flex flex-col items-end justify-between self-stretch relative z-10">
        <PlaylistItemIndicators task={task} unresolvedDeps={unresolvedDeps} signalSummary={signalSummary} />
        {task.tags && task.tags.length > 0 && (
          <div className="flex flex-wrap justify-end gap-1">
            {task.tags.map(tag => (
              <Badge key={tag} variant="outline" className="text-xs px-2.5 py-0.5 h-5 min-w-[3rem]">
                {tag}
              </Badge>
            ))}
          </div>
        )}
      </div>

      {/* Full-width row: BlockedBy alert */}
      {task.blockedBy && (
        <Alert variant="destructive" className="col-start-2 col-span-2 mt-1 py-1.5 px-2">
          <AlertDescription className="text-xs flex items-center gap-1.5">{task.blockedBy}</AlertDescription>
        </Alert>
      )}
    </button>
  )
})
