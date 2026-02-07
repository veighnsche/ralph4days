import { Layers } from 'lucide-react'
import { PageContent, PageHeader, PageLayout } from '@/components/layout/PageLayout'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardDescription, CardTitle } from '@/components/ui/card'
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from '@/components/ui/empty'
import { ItemGroup, ItemSeparator } from '@/components/ui/item'
import { Progress } from '@/components/ui/progress'
import { Skeleton } from '@/components/ui/skeleton'
import { useDisciplines } from '@/hooks/useDisciplines'
import { useInvoke } from '@/hooks/useInvoke'
import type { GroupStats, ProjectProgress } from '@/types/prd'

export function DisciplinesPage() {
  const { disciplines } = useDisciplines()
  const { data: disciplineStats = [], isLoading: statsLoading } = useInvoke<GroupStats[]>('get_discipline_stats')
  const { data: progress } = useInvoke<ProjectProgress>('get_project_progress')

  const totalTasks = progress?.totalTasks ?? 0
  const doneTasks = progress?.doneTasks ?? 0
  const progressPercent = progress?.progressPercent ?? 0

  // Build lookup map from stats array
  const statsMap = new Map<string, GroupStats>()
  for (const stat of disciplineStats) {
    statsMap.set(stat.name, stat)
  }

  const loading = statsLoading || disciplines.length === 0

  if (loading) {
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
                    Done: <span style={{ color: 'var(--status-done)' }}>{doneTasks}</span>
                  </div>
                </div>
                <div className="text-right">
                  <div className="text-sm font-medium">
                    Remaining: <span className="text-muted-foreground">{totalTasks - doneTasks}</span>
                  </div>
                </div>
                <div className="text-right min-w-[60px]">
                  <div className="text-lg font-semibold leading-none">{progressPercent}%</div>
                  <div className="text-[10px] text-muted-foreground">Complete</div>
                </div>
              </div>
            </div>
            <Progress value={progressPercent} className="h-1.5" />
            <CardDescription className="text-xs">Work categories and their task distribution</CardDescription>
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
          <ItemGroup className="rounded-md border">
            {disciplines.map((discipline, index) => {
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
                <div key={discipline.name}>
                  <div className="p-4 hover:bg-muted/50 transition-colors">
                    <div className="flex items-start justify-between gap-4">
                      <div className="flex items-start gap-3 flex-1 min-w-0">
                        <div
                          className="p-2 rounded-md shrink-0"
                          style={{
                            backgroundColor: discipline.bgColor,
                            color: discipline.color
                          }}>
                          <Icon className="h-4 w-4" />
                        </div>
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2 mb-1">
                            <h3 className="font-medium">{discipline.displayName}</h3>
                            <Badge variant="outline" className="text-xs">
                              {stats.total} tasks
                            </Badge>
                          </div>
                          <div className="flex items-center gap-2 text-xs text-muted-foreground">
                            {stats.done > 0 && <span>{stats.done} done</span>}
                            {stats.inProgress > 0 && <span>{stats.inProgress} in progress</span>}
                            {stats.pending > 0 && <span>{stats.pending} pending</span>}
                          </div>
                        </div>
                      </div>
                      <div className="text-right shrink-0">
                        <div className="text-lg font-semibold">{discProgress}%</div>
                        <div className="text-xs text-muted-foreground">complete</div>
                      </div>
                    </div>
                  </div>
                  {index < disciplines.length - 1 && <ItemSeparator />}
                </div>
              )
            })}
          </ItemGroup>
        )}
      </PageContent>
    </PageLayout>
  )
}
