import { useMemo } from 'react'
import { PageContent, PageHeader, PageLayout } from '@/components/layout/PageLayout'
import { PRDBody } from '@/components/prd/PRDBody'
import { PRDHeader } from '@/components/prd/PRDHeader'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Skeleton } from '@/components/ui/skeleton'
import { useInvoke } from '@/hooks/api'
import { useDisciplines } from '@/hooks/disciplines/useDisciplines'
import { usePRDData, usePRDFilters } from '@/hooks/tasks'
import { useWorkspaceActions } from '@/hooks/workspace'
import { computeProjectProgress, getAllTags } from '@/lib/stats'
import type { DisciplineCropsData, ProjectInfo } from '@/types/generated'

// TODO: Implement task-bound terminal system (POC tested 2026-02-06)
// - Add play button to tasks, bind to terminal, generate task prompt
// - Listen for PTY close, update task status, clear binding

export function TasksPage() {
  const { tasks, isLoading: tasksLoading, error } = usePRDData()
  const { data: projectInfo } = useInvoke<ProjectInfo>('get_project_info')
  const { disciplines } = useDisciplines()
  const cropsStore = useMemo(() => {
    const map = new Map<string, DisciplineCropsData>()
    for (const d of disciplines) {
      if (d.crops) map.set(d.name, d.crops)
    }
    return map
  }, [disciplines])

  const allTags = useMemo(() => getAllTags(tasks ?? []), [tasks])
  const progress = useMemo(() => computeProjectProgress(tasks ?? []), [tasks])

  const { filters, setters, filteredTasks, clearFilters } = usePRDFilters(tasks, allTags)
  const { openBraindumpTab, openTaskDetailTab } = useWorkspaceActions()

  const totalTasks = progress.totalTasks
  const doneTasks = progress.doneTasks
  const progressPercent = progress.progressPercent

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
          cropsStore={cropsStore}
          onTaskClick={openTaskDetailTab}
          onClearFilters={clearFilters}
          onBraindump={() => openBraindumpTab('Braindump Project')}
          onYap={() => openBraindumpTab('Yap about Tasks')}
        />
      </PageContent>
    </PageLayout>
  )
}
