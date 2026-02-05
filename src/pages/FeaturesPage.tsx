import { Target } from "lucide-react";
import { useMemo } from "react";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardTitle } from "@/components/ui/card";
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty";
import { ItemGroup, ItemSeparator } from "@/components/ui/item";
import { Progress } from "@/components/ui/progress";
import { Skeleton } from "@/components/ui/skeleton";
import { useInvoke } from "@/hooks/useInvoke";
import { usePRDData } from "@/hooks/usePRDData";
import type { Feature } from "@/types/prd";

export function FeaturesPage() {
  const { prdData, isLoading: prdLoading, error: prdError } = usePRDData();
  const {
    data: features = [],
    isLoading: featuresLoading,
    error: featuresError,
  } = useInvoke<Feature[]>("get_features");

  // Calculate task counts per feature
  const featureStats = useMemo(() => {
    if (!prdData) return new Map();
    const stats = new Map<string, { total: number; done: number; pending: number; inProgress: number }>();

    for (const task of prdData.tasks) {
      if (!stats.has(task.feature)) {
        stats.set(task.feature, { total: 0, done: 0, pending: 0, inProgress: 0 });
      }
      const stat = stats.get(task.feature)!;
      stat.total++;
      if (task.status === "done") stat.done++;
      if (task.status === "pending") stat.pending++;
      if (task.status === "in_progress") stat.inProgress++;
    }

    return stats;
  }, [prdData]);

  // Calculate overall progress
  const totalTasks = prdData?.tasks.length ?? 0;
  const doneTasks = prdData?.tasks.filter((t) => t.status === "done").length ?? 0;
  const progressPercent = totalTasks > 0 ? Math.round((doneTasks / totalTasks) * 100) : 0;

  const loading = prdLoading || featuresLoading;
  const error = prdError || (featuresError ? String(featuresError) : null);

  if (loading) {
    return (
      <div className="h-full flex flex-col overflow-hidden">
        <div className="flex-shrink-0 p-4">
          <Skeleton className="h-[120px]" />
        </div>
        <div className="flex-1 p-4 space-y-4">
          <Skeleton className="h-[100px]" />
          <Skeleton className="h-[100px]" />
          <Skeleton className="h-[100px]" />
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="h-full p-4">
        <Alert variant="destructive">
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col overflow-hidden">
      {/* Features Header */}
      <div className="flex-shrink-0 p-4 pb-0">
        <Card className="py-3">
          <CardContent className="space-y-3">
            <div className="flex items-center justify-between gap-4">
              <div className="flex items-center gap-2">
                <Target className="h-5 w-5" />
                <CardTitle className="text-base">Features</CardTitle>
              </div>
              <div className="flex items-center gap-4">
                <div className="text-right">
                  <div className="text-sm font-medium">
                    Total: <span className="text-[hsl(var(--muted-foreground))]">{features.length}</span>
                  </div>
                </div>
                <div className="text-right">
                  <div className="text-sm font-medium">
                    Done: <span className="text-green-600">{doneTasks}</span>
                  </div>
                </div>
                <div className="text-right">
                  <div className="text-sm font-medium">
                    Remaining: <span className="text-[hsl(var(--muted-foreground))]">{totalTasks - doneTasks}</span>
                  </div>
                </div>
                <div className="text-right min-w-[60px]">
                  <div className="text-2xl font-bold leading-none">{progressPercent}%</div>
                  <div className="text-[10px] text-[hsl(var(--muted-foreground))]">Complete</div>
                </div>
              </div>
            </div>
            <Progress value={progressPercent} className="h-1.5" />
            <CardDescription className="text-xs">Product features and their associated tasks</CardDescription>
          </CardContent>
        </Card>
      </div>

      {/* Features List */}
      <div className="flex-1 min-h-0 overflow-auto p-4">
        {features.length === 0 ? (
          <Empty>
            <EmptyHeader>
              <EmptyMedia variant="icon">
                <Target />
              </EmptyMedia>
              <EmptyTitle>No features yet</EmptyTitle>
              <EmptyDescription>Features will appear here as you create tasks</EmptyDescription>
            </EmptyHeader>
            <EmptyContent />
          </Empty>
        ) : (
          <ItemGroup className="rounded-md border">
            {features.map((feature, index) => {
              const stats = featureStats.get(feature.name) || { total: 0, done: 0, pending: 0, inProgress: 0 };
              const progress = stats.total > 0 ? Math.round((stats.done / stats.total) * 100) : 0;

              return (
                <>
                  <div key={feature.name} className="p-4 hover:bg-muted/50 transition-colors">
                    <div className="flex items-start justify-between gap-4">
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 mb-1">
                          <h3 className="font-medium">{feature.display_name}</h3>
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
                        <div className="text-2xl font-bold">{progress}%</div>
                        <div className="text-xs text-muted-foreground">complete</div>
                      </div>
                    </div>
                  </div>
                  {index < features.length - 1 && <ItemSeparator />}
                </>
              );
            })}
          </ItemGroup>
        )}
      </div>
    </div>
  );
}
