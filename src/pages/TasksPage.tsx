import { useCallback } from 'react'
import { PageContent, PageHeader, PageLayout } from '@/components/layout/PageLayout'
import { PRDBody } from '@/components/prd/PRDBody'
import { PRDHeader } from '@/components/prd/PRDHeader'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Skeleton } from '@/components/ui/skeleton'
import { useInvoke } from '@/hooks/useInvoke'
import { usePRDData } from '@/hooks/usePRDData'
import { usePRDFilters } from '@/hooks/usePRDFilters'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'
import type { ProjectInfo, ProjectProgress, Task } from '@/types/prd'

// TODO: Implement task-bound terminal system (POC tested 2026-02-06)
// - Add play button to tasks, bind to terminal, generate task prompt
// - Listen for PTY close, update task status, clear binding

export function TasksPage() {
  const { tasks, isLoading: tasksLoading, error } = usePRDData()
  const { data: progress } = useInvoke<ProjectProgress>('get_project_progress')
  const { data: allTags = [] } = useInvoke<string[]>('get_all_tags')
  const { data: projectInfo } = useInvoke<ProjectInfo>('get_project_info')
  const { filters, setters, filteredTasks, clearFilters } = usePRDFilters(tasks, allTags)
  const openTab = useWorkspaceStore(s => s.openTab)

  const totalTasks = progress?.totalTasks ?? 0
  const doneTasks = progress?.doneTasks ?? 0
  const progressPercent = progress?.progressPercent ?? 0

  const handleBraindumpProject = () => {
    openTab({
      type: 'braindump-form',
      title: 'Braindump Project',
      closeable: true
    })
  }

  const handleYapAboutTasks = () => {
    openTab({
      type: 'braindump-form',
      title: 'Yap about Tasks',
      closeable: true
    })
  }

  const handleTaskClick = useCallback(
    (task: Task) => {
      openTab({
        type: 'task-detail',
        title: task.title,
        closeable: true,
        data: { entityId: task.id, entity: task }
      })
    },
    [openTab]
  )

  const loading = tasksLoading

  if (loading) {
    return (
      <PageLayout>
        <PageHeader>
          <Skeleton className="h-[200px]" />
        </PageHeader>
        <PageContent>
          <div className="space-y-4">
            <Skeleton className="h-[60px]" />
            <Skeleton className="h-[60px]" />
            <Skeleton className="h-[60px]" />
          </div>
        </PageContent>
      </PageLayout>
    )
  }

  if (error) {
    return (
      <PageLayout>
        <PageContent>
          <Alert variant="destructive">
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        </PageContent>
      </PageLayout>
    )
  }

  if (!tasks) {
    return (
      <PageLayout>
        <PageContent>
          <Alert>
            <AlertDescription>No task data available</AlertDescription>
          </Alert>
        </PageContent>
      </PageLayout>
    )
  }

  return (
    <PageLayout>
      <PageHeader>
        <PRDHeader
          project={projectInfo ?? { title: 'Project' }}
          totalTasks={totalTasks}
          doneTasks={doneTasks}
          progressPercent={progressPercent}
          filteredCount={filteredTasks.length}
          filters={filters}
          setters={setters}
          allTags={allTags}
          onClearFilters={clearFilters}
        />
      </PageHeader>

      <PageContent>
        <PRDBody
          filteredTasks={filteredTasks}
          totalTasks={totalTasks}
          onTaskClick={handleTaskClick}
          onClearFilters={clearFilters}
          onBraindump={handleBraindumpProject}
          onYap={handleYapAboutTasks}
        />
      </PageContent>
    </PageLayout>
  )
}
