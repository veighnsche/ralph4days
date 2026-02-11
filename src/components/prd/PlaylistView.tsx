import { AlertCircle, ChevronDown } from 'lucide-react'
import { Fragment, useMemo, useState } from 'react'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { ItemGroup, ItemSeparator } from '@/components/ui/item'
import { TooltipProvider } from '@/components/ui/tooltip'
import { useSignalSummaries } from '@/hooks/tasks'
import type { DisciplineCropsData, Task } from '@/types/generated'
import { PlaylistItem } from './PlaylistItem'

interface PlaylistViewProps {
  tasks: Task[]
  cropsStore: Map<string, DisciplineCropsData>
  onTaskClick: (task: Task) => void
}

function countUnresolvedDeps(task: Task, statusById: Map<number, string>): number {
  if (!task.dependsOn || task.dependsOn.length === 0) return 0
  return task.dependsOn.filter(id => {
    const s = statusById.get(id)
    return s !== 'done' && s !== 'skipped'
  }).length
}

export function PlaylistView({ tasks, cropsStore, onTaskClick }: PlaylistViewProps) {
  const [issuesOpen, setIssuesOpen] = useState(true)

  const taskIds = useMemo(() => tasks.map(t => t.id), [tasks])
  const { summaries } = useSignalSummaries(taskIds)

  const statusById = new Map(tasks.map(t => [t.id, t.status]))

  const { blockedSkipped, done, inProgress, pending } = (() => {
    const result = {
      blockedSkipped: [] as Task[],
      done: [] as Task[],
      inProgress: [] as Task[],
      pending: [] as Task[]
    }

    tasks.forEach(task => {
      if (task.status === 'blocked' || task.status === 'skipped') {
        result.blockedSkipped.push(task)
      } else if (task.status === 'done') {
        result.done.push(task)
      } else if (task.status === 'in_progress') {
        result.inProgress.push(task)
      } else if (task.status === 'pending') {
        result.pending.push(task)
      }
    })

    return result
  })()

  const hasBlockedOrSkipped = blockedSkipped.length > 0

  return (
    <TooltipProvider>
      <div className="flex flex-col gap-3 pb-4">
        {hasBlockedOrSkipped && (
          <Collapsible open={issuesOpen} onOpenChange={setIssuesOpen}>
            <CollapsibleTrigger className="w-full group">
              <div
                className="text-sm flex items-center gap-2 px-1 hover:opacity-70 transition-opacity"
                style={{ color: 'hsl(var(--status-blocked))' }}>
                <AlertCircle className="h-4 w-4" />
                Issues Requiring Attention
                <span className="text-xs font-normal opacity-70">({blockedSkipped.length})</span>
                <ChevronDown
                  className="h-4 w-4 ml-auto transition-transform"
                  style={{ transform: issuesOpen ? 'rotate(0deg)' : 'rotate(-90deg)' }}
                />
              </div>
            </CollapsibleTrigger>
            <CollapsibleContent className="mt-2">
              <ItemGroup className="rounded-md">
                {blockedSkipped.map((task, index) => (
                  <Fragment key={task.id}>
                    <PlaylistItem
                      task={task}
                      crops={cropsStore.get(task.discipline)}
                      unresolvedDeps={countUnresolvedDeps(task, statusById)}
                      signalSummary={summaries[task.id]}
                      onClick={() => onTaskClick(task)}
                    />
                    {index < blockedSkipped.length - 1 && <ItemSeparator />}
                  </Fragment>
                ))}
              </ItemGroup>
            </CollapsibleContent>
          </Collapsible>
        )}

        <ItemGroup className="rounded-md">
          {done.map(task => (
            <Fragment key={task.id}>
              <PlaylistItem
                task={task}
                crops={cropsStore.get(task.discipline)}
                unresolvedDeps={countUnresolvedDeps(task, statusById)}
                signalSummary={summaries[task.id]}
                onClick={() => onTaskClick(task)}
              />
              <ItemSeparator />
            </Fragment>
          ))}

          {inProgress.map(task => (
            <Fragment key={task.id}>
              <PlaylistItem
                task={task}
                crops={cropsStore.get(task.discipline)}
                unresolvedDeps={countUnresolvedDeps(task, statusById)}
                signalSummary={summaries[task.id]}
                isNowPlaying
                onClick={() => onTaskClick(task)}
              />
              <ItemSeparator />
            </Fragment>
          ))}

          {pending.map((task, index) => (
            <Fragment key={task.id}>
              <PlaylistItem
                task={task}
                crops={cropsStore.get(task.discipline)}
                unresolvedDeps={countUnresolvedDeps(task, statusById)}
                signalSummary={summaries[task.id]}
                onClick={() => onTaskClick(task)}
              />
              {index < pending.length - 1 && <ItemSeparator />}
            </Fragment>
          ))}

          {tasks.length === 0 && (
            <div className="flex items-center justify-center h-32">
              <p className="text-sm text-muted-foreground">No tasks in playlist</p>
            </div>
          )}
        </ItemGroup>
      </div>
    </TooltipProvider>
  )
}
