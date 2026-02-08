import { Layers } from 'lucide-react'
import { PageContent, PageHeader, PageLayout } from '@/components/layout/PageLayout'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardDescription, CardTitle } from '@/components/ui/card'
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from '@/components/ui/empty'
import { Progress } from '@/components/ui/progress'
import { Skeleton } from '@/components/ui/skeleton'
import { DisciplineDetailTabContent } from '@/components/workspace/DisciplineDetailTabContent'
import { useDisciplineStats, useStackMetadata } from '@/hooks/disciplines'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'

export function DisciplinesPage() {
  const { disciplines, statsMap, progress, isLoading: disciplinesLoading } = useDisciplineStats()
  const { stacks, isLoading: stacksLoading } = useStackMetadata()
  const openTab = useWorkspaceStore(state => state.openTab)

  const handleDisciplineClick = (disciplineName: string, displayName: string) => {
    openTab({
      type: 'discipline-detail',
      component: DisciplineDetailTabContent,
      title: displayName,
      closeable: true,
      data: {
        entityId: disciplineName
      }
    })
  }

  const isLoading = disciplinesLoading || stacksLoading

  if (isLoading) {
    return (
      <PageLayout>
        <PageHeader>
          <Skeleton className="h-[120px]" />
        </PageHeader>
        <PageContent>
          <div className="space-y-4">
            <Skeleton className="h-[100px]" />
            <Skeleton className="h-[100px]" />
            <Skeleton className="h-[100px]" />
          </div>
        </PageContent>
      </PageLayout>
    )
  }

  const disciplinesByStack = new Map<number, typeof disciplines>()
  disciplines.forEach(disc => {
    if (disc.stackId != null) {
      const existing = disciplinesByStack.get(disc.stackId) || []
      disciplinesByStack.set(disc.stackId, [...existing, disc])
    }
  })

  const uncategorized = disciplines.filter(d => d.stackId == null)

  return (
    <PageLayout>
      <PageHeader>
        <Card className="py-3">
          <CardContent className="space-y-3">
            <div className="flex items-center justify-between gap-4">
              <div className="flex items-center gap-2 flex-1">
                <Layers className="h-4 w-4" />
                <CardTitle className="text-base">Disciplines</CardTitle>
              </div>
              <div className="flex items-center gap-4">
                <div className="text-right">
                  <div className="text-sm font-medium">
                    Total: <span className="text-muted-foreground">{disciplines.length}</span>
                  </div>
                </div>
                <div className="text-right">
                  <div className="text-sm font-medium">
                    Done: <span style={{ color: 'var(--status-done)' }}>{progress.done}</span>
                  </div>
                </div>
                <div className="text-right">
                  <div className="text-sm font-medium">
                    Remaining: <span className="text-muted-foreground">{progress.total - progress.done}</span>
                  </div>
                </div>
                <div className="text-right min-w-[60px]">
                  <div className="text-lg font-semibold leading-none">{progress.percent}%</div>
                  <div className="text-[10px] text-muted-foreground">Complete</div>
                </div>
              </div>
            </div>
            <Progress value={progress.percent} className="h-1.5" />
            <CardDescription className="text-xs">Team roster - deploy specialists for the job</CardDescription>
          </CardContent>
        </Card>
      </PageHeader>

      <PageContent>
        {disciplines.length === 0 ? (
          <Empty>
            <EmptyHeader>
              <EmptyMedia variant="icon">
                <Layers />
              </EmptyMedia>
              <EmptyTitle>No disciplines configured</EmptyTitle>
              <EmptyDescription>Disciplines define the types of work in your project</EmptyDescription>
            </EmptyHeader>
            <EmptyContent />
          </Empty>
        ) : (
          <div className="space-y-8">
            {stacks
              .filter(stack => disciplinesByStack.has(stack.stackId))
              .map(stack => {
                const stackDisciplines = disciplinesByStack.get(stack.stackId) || []
                return (
                  <div key={stack.stackId} className="space-y-3">
                    <div className="space-y-1">
                      <div className="flex items-baseline gap-2">
                        <h3 className="text-sm font-semibold">{stack.name}</h3>
                        <Badge variant="outline" className="text-xs">
                          {stackDisciplines.length} specialists
                        </Badge>
                      </div>
                      <p className="text-xs text-muted-foreground">{stack.description}</p>
                      <p className="text-xs text-muted-foreground italic">{stack.visualIdentity.tone}</p>
                    </div>

                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-3">
                      {stackDisciplines.map(discipline => {
                        const Icon = discipline.icon
                        const stats = statsMap.get(discipline.name) || {
                          total: 0,
                          done: 0,
                          pending: 0,
                          inProgress: 0,
                          blocked: 0,
                          skipped: 0
                        }
                        const discProgress = stats.total > 0 ? Math.round((stats.done / stats.total) * 100) : 0

                        return (
                          <button
                            key={discipline.name}
                            type="button"
                            className="text-left"
                            onClick={() => handleDisciplineClick(discipline.name, discipline.displayName)}>
                            <Card className="h-full hover:bg-muted/30 transition-colors cursor-pointer">
                              <CardContent className="p-4 space-y-3">
                                <div className="flex items-start justify-between gap-2">
                                  <div
                                    className="p-2 rounded-md shrink-0"
                                    style={{
                                      backgroundColor: discipline.bgColor,
                                      color: discipline.color
                                    }}>
                                    <Icon className="h-5 w-5" />
                                  </div>
                                  <Badge variant="secondary" className="text-xs font-mono">
                                    {discipline.acronym}
                                  </Badge>
                                </div>

                                <div className="space-y-1">
                                  <h4 className="font-medium text-sm">{discipline.displayName}</h4>
                                  <div className="flex items-center gap-2 text-xs text-muted-foreground">
                                    <span>{stats.total} tasks</span>
                                    {stats.inProgress > 0 && <span>• {stats.inProgress} active</span>}
                                  </div>
                                </div>

                                <div className="space-y-1">
                                  <div className="flex items-center justify-between text-xs">
                                    <span className="text-muted-foreground">Progress</span>
                                    <span className="font-semibold">{discProgress}%</span>
                                  </div>
                                  <Progress value={discProgress} className="h-1" />
                                </div>
                              </CardContent>
                            </Card>
                          </button>
                        )
                      })}
                    </div>
                  </div>
                )
              })}

            {uncategorized.length > 0 && (
              <div className="space-y-3">
                <div className="space-y-1">
                  <div className="flex items-baseline gap-2">
                    <h3 className="text-sm font-semibold">Custom Disciplines</h3>
                    <Badge variant="outline" className="text-xs">
                      {uncategorized.length} specialists
                    </Badge>
                  </div>
                  <p className="text-xs text-muted-foreground">User-defined disciplines</p>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-3">
                  {uncategorized.map(discipline => {
                    const Icon = discipline.icon
                    const stats = statsMap.get(discipline.name) || {
                      total: 0,
                      done: 0,
                      pending: 0,
                      inProgress: 0,
                      blocked: 0,
                      skipped: 0
                    }
                    const discProgress = stats.total > 0 ? Math.round((stats.done / stats.total) * 100) : 0

                    return (
                      <button
                        key={discipline.name}
                        type="button"
                        className="text-left"
                        onClick={() => handleDisciplineClick(discipline.name, discipline.displayName)}>
                        <Card className="h-full hover:bg-muted/30 transition-colors cursor-pointer">
                          <CardContent className="p-4 space-y-3">
                            <div className="flex items-start justify-between gap-2">
                              <div
                                className="p-2 rounded-md shrink-0"
                                style={{
                                  backgroundColor: discipline.bgColor,
                                  color: discipline.color
                                }}>
                                <Icon className="h-5 w-5" />
                              </div>
                              <Badge variant="secondary" className="text-xs font-mono">
                                {discipline.acronym}
                              </Badge>
                            </div>

                            <div className="space-y-1">
                              <h4 className="font-medium text-sm">{discipline.displayName}</h4>
                              <div className="flex items-center gap-2 text-xs text-muted-foreground">
                                <span>{stats.total} tasks</span>
                                {stats.inProgress > 0 && <span>• {stats.inProgress} active</span>}
                              </div>
                            </div>

                            <div className="space-y-1">
                              <div className="flex items-center justify-between text-xs">
                                <span className="text-muted-foreground">Progress</span>
                                <span className="font-semibold">{discProgress}%</span>
                              </div>
                              <Progress value={discProgress} className="h-1" />
                            </div>
                          </CardContent>
                        </Card>
                      </button>
                    )
                  })}
                </div>
              </div>
            )}
          </div>
        )}
      </PageContent>
    </PageLayout>
  )
}
