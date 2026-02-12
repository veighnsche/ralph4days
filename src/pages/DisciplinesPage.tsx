import { Layers } from 'lucide-react'
import { PageContent, PageHeader, PageLayout } from '@/components/layout/PageLayout'
import { DisciplineLabel } from '@/components/prd/DisciplineLabel'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardDescription, CardTitle } from '@/components/ui/card'
import { CroppedImage } from '@/components/ui/cropped-image'
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from '@/components/ui/empty'
import { Progress } from '@/components/ui/progress'
import { Skeleton } from '@/components/ui/skeleton'
import { DisciplineDetailTabContent } from '@/components/workspace/DisciplineDetailTabContent'
import { useDisciplineStats, useStackMetadata } from '@/hooks/disciplines'
import { NOOP_TAB_LIFECYCLE, useWorkspaceStore } from '@/stores/useWorkspaceStore'

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
      lifecycle: NOOP_TAB_LIFECYCLE,
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

  const stackNameMap = new Map<number, string>()
  for (const stack of stacks) {
    stackNameMap.set(stack.stackId, stack.name)
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
          <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4">
            {disciplines.map(discipline => {
              const Icon = discipline.icon
              const stats = statsMap.get(discipline.name) || {
                total: 0,
                done: 0,
                pending: 0,
                inProgress: 0,
                blocked: 0,
                skipped: 0
              }
              const stackName = discipline.stackId != null ? stackNameMap.get(discipline.stackId) : undefined

              return (
                <button
                  key={discipline.name}
                  type="button"
                  className="text-left transition-all duration-200 hover:scale-[1.03] rounded-lg"
                  style={
                    {
                      '--disc-color': discipline.color
                    } as React.CSSProperties
                  }
                  onClick={() => handleDisciplineClick(discipline.name, discipline.displayName)}>
                  <div
                    className="h-full rounded-lg border-2 bg-card overflow-hidden cursor-pointer transition-shadow duration-200 hover:shadow-[0_0_12px_var(--disc-color)]"
                    style={{ borderColor: discipline.color }}>
                    <div className="flex items-center justify-between px-2.5 py-1.5">
                      <DisciplineLabel acronym={discipline.acronym} color={discipline.color} className="font-bold" />
                      {stackName && (
                        <Badge variant="outline" className="text-[10px] h-4 px-1">
                          {stackName}
                        </Badge>
                      )}
                    </div>

                    <div className="px-2.5 pb-1">
                      {discipline.imagePath && discipline.crops?.card ? (
                        <CroppedImage
                          disciplineName={discipline.name}
                          label="card"
                          crop={discipline.crops.card}
                          className="rounded w-full h-[180px]"
                        />
                      ) : (
                        <div
                          className="w-full h-[180px] rounded flex items-center justify-center"
                          style={{ backgroundColor: discipline.bgColor }}>
                          <Icon className="h-16 w-16" style={{ color: discipline.color }} />
                        </div>
                      )}
                    </div>

                    <div className="px-2.5 py-2 space-y-0.5">
                      <h4 className="font-bold text-sm leading-tight">{discipline.displayName}</h4>
                      <p className="text-[11px] text-muted-foreground">
                        {stats.total} tasks{stats.inProgress > 0 && ` \u2022 ${stats.inProgress} active`}
                      </p>
                    </div>
                  </div>
                </button>
              )
            })}
          </div>
        )}
      </PageContent>
    </PageLayout>
  )
}
