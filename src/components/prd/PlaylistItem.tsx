import { ArrowDown, ArrowUp, Bot, Cog, Equal, GitBranch, ListChecks, MessageSquare, User } from 'lucide-react'
import { memo } from 'react'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { Item, ItemActions, ItemContent, ItemDescription, ItemTitle } from '@/components/ui/item'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { INFERRED_STATUS_CONFIG, PRIORITY_CONFIG, STATUS_CONFIG } from '@/constants/prd'
import type { DisciplineCropsData, Task } from '@/types/generated'
import { DisciplineHeadshot } from './DisciplineHeadshot'
import { DisciplineLabel } from './DisciplineLabel'
import { TaskIdDisplay } from './TaskIdDisplay'

interface DisciplineImageEntry {
  imageUrl: string
  crops?: DisciplineCropsData
}

interface PlaylistItemProps {
  task: Task
  image?: DisciplineImageEntry
  isNowPlaying?: boolean
  isIssue?: boolean
  onClick: () => void
}

const PRIORITY_ICONS = { high: ArrowUp, medium: Equal, low: ArrowDown } as const

const PROVENANCE_ICONS = { agent: Bot, human: User, system: Cog } as const

function PlaylistItemIndicators({
  task,
  priorityConfig
}: {
  task: Task
  priorityConfig: (typeof PRIORITY_CONFIG)[keyof typeof PRIORITY_CONFIG] | null
}) {
  const hasAny =
    (task.comments?.length ?? 0) > 0 ||
    (task.dependsOn?.length ?? 0) > 0 ||
    (task.acceptanceCriteria?.length ?? 0) > 0 ||
    priorityConfig

  if (!hasAny) return null

  return (
    <div className="absolute top-2 right-3 flex items-center gap-2 text-muted-foreground z-10">
      {task.comments && task.comments.length > 0 && (
        <div className="flex items-center gap-1">
          <MessageSquare className="h-3 w-3" />
          <span className="text-xs">{task.comments.length}</span>
        </div>
      )}
      {task.dependsOn && task.dependsOn.length > 0 && (
        <div
          className="flex items-center gap-1"
          style={
            task.inferredStatus === 'waiting_on_deps'
              ? { color: INFERRED_STATUS_CONFIG.waiting_on_deps.color }
              : undefined
          }>
          <GitBranch className="h-3 w-3" />
          <span className="text-xs">{task.dependsOn.length}</span>
        </div>
      )}
      {task.acceptanceCriteria && task.acceptanceCriteria.length > 0 && (
        <div className="flex items-center gap-1">
          <ListChecks className="h-3 w-3" />
          <span className="text-xs">{task.acceptanceCriteria.length}</span>
        </div>
      )}
      {priorityConfig &&
        (() => {
          const PriorityIcon = PRIORITY_ICONS[task.priority as keyof typeof PRIORITY_ICONS] ?? Equal
          return <PriorityIcon className="h-3 w-3" style={{ color: priorityConfig.color }} />
        })()}
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

function PlaylistItemActions({ task }: { task: Task }) {
  return (
    <ItemActions className="flex-col items-end justify-end self-stretch gap-2 relative z-10">
      <div className="flex flex-wrap gap-1 justify-end">
        {task.tags?.map(tag => (
          <Badge key={tag} variant="outline" className="text-xs px-2.5 py-0.5 h-5 min-w-[3rem]">
            {tag}
          </Badge>
        ))}
      </div>
    </ItemActions>
  )
}

function getItemStyle(status: Task['status'], statusConfig: (typeof STATUS_CONFIG)[keyof typeof STATUS_CONFIG]) {
  return {
    borderLeftColor: statusConfig.color,
    backgroundColor: statusConfig.bgColor,
    opacity: status === 'done' || status === 'skipped' ? 0.5 : 1
  }
}

export const PlaylistItem = memo(function PlaylistItem({
  task,
  image,
  isNowPlaying = false,
  onClick
}: PlaylistItemProps) {
  const statusConfig = STATUS_CONFIG[task.status]
  const priorityConfig = task.priority ? PRIORITY_CONFIG[task.priority] : null
  const hasHeadshot = image?.imageUrl && image?.crops?.face

  return (
    <Item
      size="sm"
      variant="default"
      className="cursor-pointer transition-all duration-200 hover:opacity-80 border-l-4 relative overflow-hidden min-h-22"
      style={getItemStyle(task.status, statusConfig)}
      onClick={onClick}>
      {priorityConfig && (
        <div
          className="absolute top-0 right-0 w-32 h-32 pointer-events-none"
          style={{
            background: `radial-gradient(circle at top right, ${priorityConfig.bgColor} 0%, transparent 70%)`
          }}
        />
      )}

      <PlaylistItemIndicators task={task} priorityConfig={priorityConfig} />

      {hasHeadshot && (
        <DisciplineHeadshot
          imageUrl={image.imageUrl}
          faceCrop={image.crops?.face ?? { x: 0, y: 0, w: 1, h: 1 }}
          bgColor={statusConfig.bgColor}
        />
      )}

      <div className={`flex-shrink-0 self-start relative z-10 ${hasHeadshot ? 'ml-22' : ''}`}>
        {hasHeadshot ? (
          <div className="flex flex-col items-start leading-tight font-mono">
            <span className="text-xs text-muted-foreground">{task.featureAcronym}</span>
            <DisciplineLabel acronym={task.disciplineAcronym} color={task.disciplineColor} />
            <span className="text-xs text-muted-foreground">
              {task.id > 999 ? task.id.toString() : `#${task.id.toString().padStart(3, '0')}`}
            </span>
          </div>
        ) : (
          <TaskIdDisplay task={task} />
        )}
      </div>

      <ItemContent className="gap-0 relative z-10 min-w-0">
        <ItemTitle
          className={`flex items-center gap-1.5 truncate ${isNowPlaying ? 'text-base' : 'text-sm'}`}
          style={isNowPlaying ? { color: statusConfig.color } : undefined}>
          {task.provenance && <ProvenanceIcon provenance={task.provenance} />}
          <span className="truncate">{task.title}</span>
          {isNowPlaying && <span className="ml-1 text-xs opacity-70 flex-shrink-0">[NOW PLAYING]</span>}
        </ItemTitle>

        {task.description && <ItemDescription className="truncate">{task.description}</ItemDescription>}

        {task.blockedBy && (
          <Alert variant="destructive" className="mt-1 py-1.5 px-2">
            <AlertDescription className="text-xs flex items-center gap-1.5">{task.blockedBy}</AlertDescription>
          </Alert>
        )}
      </ItemContent>

      <PlaylistItemActions task={task} />
    </Item>
  )
})
