import { Brain, FileX } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from '@/components/ui/empty'
import type { DisciplineCropsData, Task } from '@/types/generated'
import { PlaylistView } from './PlaylistView'

interface PRDBodyProps {
  filteredTasks: Task[]
  totalTasks: number
  cropsStore: Map<string, DisciplineCropsData>
  onTaskClick: (task: Task) => void
  onClearFilters: () => void
}

export function PRDBody({ filteredTasks, totalTasks, cropsStore, onTaskClick, onClearFilters }: PRDBodyProps) {
  if (filteredTasks.length === 0) {
    if (totalTasks === 0) {
      return (
        <Empty>
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <Brain />
            </EmptyMedia>
            <EmptyTitle>No tasks yet</EmptyTitle>
            <EmptyDescription>
              Start an agent session from the workspace (+) and execute tasks as they are created.
            </EmptyDescription>
          </EmptyHeader>
          <EmptyContent />
        </Empty>
      )
    }

    return (
      <Empty>
        <EmptyHeader>
          <EmptyMedia variant="icon">
            <FileX />
          </EmptyMedia>
          <EmptyTitle>No tasks found</EmptyTitle>
          <EmptyDescription>
            No tasks match your current filters. Try adjusting your search criteria or clearing filters.
          </EmptyDescription>
        </EmptyHeader>
        <EmptyContent>
          <Button variant="outline" onClick={onClearFilters}>
            Clear all filters
          </Button>
        </EmptyContent>
      </Empty>
    )
  }

  return <PlaylistView tasks={filteredTasks} cropsStore={cropsStore} onTaskClick={onTaskClick} />
}
