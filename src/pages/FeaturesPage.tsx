import { Brain, MessageCircle, Target } from 'lucide-react'
import { PageContent, PageHeader, PageLayout } from '@/components/layout/PageLayout'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardTitle } from '@/components/ui/card'
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from '@/components/ui/empty'
import { ItemGroup, ItemSeparator } from '@/components/ui/item'
import { Progress } from '@/components/ui/progress'
import { Skeleton } from '@/components/ui/skeleton'
import { useFeatureStats } from '@/hooks/useFeatureStats'
import { useWorkspaceActions } from '@/hooks/useWorkspaceActions'

export function FeaturesPage() {
  const { features, statsMap, progress, isLoading, error } = useFeatureStats()
  const { openBraindumpTab } = useWorkspaceActions()

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
                <CardTitle className="text-base">Features</CardTitle>
              </div>
              <div className="flex items-center gap-4">
                <div className="text-right">
                  <div className="text-sm font-medium">
                    Total: <span className="text-muted-foreground">{features.length}</span>
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
            <CardDescription className="text-xs">Product features and their associated tasks</CardDescription>
          </CardContent>
        </Card>
      </PageHeader>

      <PageContent>
        {features.length === 0 ? (
          <Empty>
            <EmptyHeader>
              <EmptyMedia variant="icon">
                <Brain />
              </EmptyMedia>
              <EmptyTitle>No features yet</EmptyTitle>
              <EmptyDescription>
                Get started by braindumping your project ideas. Claude will help structure them into features and tasks.
              </EmptyDescription>
            </EmptyHeader>
            <EmptyContent>
              <div className="flex flex-col gap-2">
                <Button onClick={() => openBraindumpTab('Braindump Project')}>
                  <Brain className="h-4 w-4 mr-2" />
                  Braindump Project
                </Button>
                <Button onClick={() => openBraindumpTab('Ramble about Features')} variant="outline">
                  <MessageCircle className="h-4 w-4 mr-2" />
                  Ramble about Features
                </Button>
              </div>
            </EmptyContent>
          </Empty>
        ) : (
          <ItemGroup className="rounded-md border">
            {features.map((feature, index) => {
              const stats = statsMap.get(feature.name) || {
                total: 0,
                done: 0,
                pending: 0,
                inProgress: 0,
                blocked: 0,
                skipped: 0
              }
              const featureProgress = stats.total > 0 ? Math.round((stats.done / stats.total) * 100) : 0

              return (
                <div key={feature.name}>
                  <div className="p-4 hover:bg-muted/50 transition-colors">
                    <div className="flex items-start justify-between gap-4">
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 mb-1">
                          <h3 className="font-medium">{feature.displayName}</h3>
                          <Badge variant="outline" className="text-xs">
                            {stats.total} tasks
                          </Badge>
                        </div>
                        {feature.description && (
                          <p className="text-sm text-muted-foreground mb-2">{feature.description}</p>
                        )}
                        <div className="flex items-center gap-2 text-xs text-muted-foreground">
                          {stats.done > 0 && <span>{stats.done} done</span>}
                          {stats.inProgress > 0 && <span>{stats.inProgress} in progress</span>}
                          {stats.pending > 0 && <span>{stats.pending} pending</span>}
                        </div>
                      </div>
                      <div className="text-right shrink-0">
                        <div className="text-lg font-semibold">{featureProgress}%</div>
                        <div className="text-xs text-muted-foreground">complete</div>
                      </div>
                    </div>
                  </div>
                  {index < features.length - 1 && <ItemSeparator />}
                </div>
              )
            })}
          </ItemGroup>
        )}
      </PageContent>
    </PageLayout>
  )
}
