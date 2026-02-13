import { Brain, Target } from 'lucide-react'
import { PageContent, PageHeader, PageLayout } from '@/components/layout/PageLayout'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardDescription, CardTitle } from '@/components/ui/card'
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from '@/components/ui/empty'
import {
  Item,
  ItemActions,
  ItemContent,
  ItemDescription,
  ItemGroup,
  ItemSeparator,
  ItemTitle
} from '@/components/ui/item'
import { Progress } from '@/components/ui/progress'
import { Skeleton } from '@/components/ui/skeleton'
import { createSubsystemDetailTab } from '@/components/workspace/tabs'
import { useSubsystemStats } from '@/hooks/subsystems'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'

export function SubsystemsPage() {
  const { subsystems, statsMap, progress, isLoading, error } = useSubsystemStats()
  const openTab = useWorkspaceStore(s => s.openTab)

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

  return (
    <PageLayout>
      <PageHeader>
        <Card className="py-3">
          <CardContent className="space-y-3">
            <div className="flex items-center justify-between gap-4">
              <div className="flex items-center gap-2 flex-1">
                <Target className="h-4 w-4" />
                <CardTitle className="text-base">Subsystems</CardTitle>
              </div>
              <div className="flex items-center gap-4">
                <div className="text-right">
                  <div className="text-sm font-medium">
                    Total: <span className="text-muted-foreground">{subsystems.length}</span>
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
            <CardDescription className="text-xs">Product subsystems and their associated tasks</CardDescription>
          </CardContent>
        </Card>
      </PageHeader>

      <PageContent>
        {subsystems.length === 0 ? (
          <Empty>
            <EmptyHeader>
              <EmptyMedia variant="icon">
                <Brain />
              </EmptyMedia>
              <EmptyTitle>No subsystems yet</EmptyTitle>
              <EmptyDescription>
                Subsystems appear as work is captured in the project. Run an agent session to generate initial
                structure.
              </EmptyDescription>
            </EmptyHeader>
            <EmptyContent />
          </Empty>
        ) : (
          <ItemGroup className="rounded-md border">
            {subsystems.map((subsystem, index) => {
              const stats = statsMap.get(subsystem.name) || {
                total: 0,
                done: 0,
                pending: 0,
                inProgress: 0,
                blocked: 0,
                skipped: 0
              }
              const featureProgress = stats.total > 0 ? Math.round((stats.done / stats.total) * 100) : 0

              return (
                <div key={subsystem.name}>
                  <Item
                    size="sm"
                    className="cursor-pointer hover:bg-muted/50"
                    role="button"
                    tabIndex={0}
                    onClick={() => openTab(createSubsystemDetailTab(subsystem.id))}
                    onKeyDown={e => e.key === 'Enter' && openTab(createSubsystemDetailTab(subsystem.id))}>
                    <ItemContent>
                      <ItemTitle>
                        {subsystem.displayName}
                        <Badge variant="outline" className="font-mono text-xs">
                          {subsystem.acronym}
                        </Badge>
                      </ItemTitle>
                      {subsystem.description && <ItemDescription>{subsystem.description}</ItemDescription>}
                    </ItemContent>
                    <ItemActions>
                      <div className="flex items-center gap-3 shrink-0">
                        <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
                          {stats.done > 0 && <span>{stats.done} done</span>}
                          {stats.inProgress > 0 && <span>{stats.inProgress} in progress</span>}
                          {stats.pending > 0 && <span>{stats.pending} pending</span>}
                        </div>
                        <Badge variant="outline" className="text-xs">
                          {stats.total} tasks
                        </Badge>
                        <div className="text-right shrink-0 min-w-[48px]">
                          <div className="text-sm font-semibold">{featureProgress}%</div>
                        </div>
                      </div>
                    </ItemActions>
                  </Item>
                  {index < subsystems.length - 1 && <ItemSeparator />}
                </div>
              )
            })}
          </ItemGroup>
        )}
      </PageContent>
    </PageLayout>
  )
}
