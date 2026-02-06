import { Brain, MessageCircle, Plus, Target } from "lucide-react";
import { useMemo } from "react";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardTitle } from "@/components/ui/card";
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty";
import { ItemGroup, ItemSeparator } from "@/components/ui/item";
import { Progress } from "@/components/ui/progress";
import { Skeleton } from "@/components/ui/skeleton";
import { useInvoke } from "@/hooks/useInvoke";
import { usePRDData } from "@/hooks/usePRDData";
import { useWorkspaceStore } from "@/stores/useWorkspaceStore";
import type { Feature } from "@/types/prd";

export function FeaturesPage() {
  const { prdData, isLoading: prdLoading, error: prdError } = usePRDData();
  const {
    data: features = [],
    isLoading: featuresLoading,
    error: featuresError,
  } = useInvoke<Feature[]>("get_features");
  const openTab = useWorkspaceStore((s) => s.openTab);

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

  const handleCreateFeature = () => {
    openTab({
      type: "feature-form",
      title: "Create Feature",
      closeable: true,
      data: { mode: "create" },
    });
  };

  const handleRambleAboutFeatures = () => {
    // TODO: Create RambleFormTabContent component (similar to BraindumpFormTabContent)
    // TODO: Default prompt should be: "I want to discuss these features: [list existing features]"
    // TODO: User can ramble about what features need, how to organize them, etc.
    // TODO: Send to Claude terminal with MCP tools to update features
    // TODO: Invalidate cache and refresh UI when done
    console.log("Ramble about features clicked - TODO: implement");
    openTab({
      type: "ramble-form", // TODO: Add this tab type
      title: "Ramble about Features",
      closeable: true,
    });
  };

  const handleBraindumpProject = () => {
    openTab({
      type: "braindump-form",
      title: "Braindump Project",
      closeable: true,
    });
  };

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
              <div className="flex items-center gap-2 flex-1">
                <Target className="h-5 w-5" />
                <CardTitle className="text-base">Features</CardTitle>
              </div>
              {features.length > 0 && (
                <>
                  <Button onClick={handleCreateFeature} size="sm" variant="outline">
                    <Plus className="h-4 w-4 mr-2" />
                    Create Feature
                  </Button>
                  <Button onClick={handleRambleAboutFeatures} size="sm" variant="outline">
                    <MessageCircle className="h-4 w-4 mr-2" />
                    Ramble about Features
                  </Button>
                </>
              )}
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
                <Brain />
              </EmptyMedia>
              <EmptyTitle>No features yet</EmptyTitle>
              <EmptyDescription>
                Get started by braindumping your project ideas. Claude will help structure them into features and tasks.
              </EmptyDescription>
            </EmptyHeader>
            <EmptyContent>
              <div className="flex flex-col gap-2">
                <Button onClick={handleBraindumpProject}>
                  <Brain className="h-4 w-4 mr-2" />
                  Braindump Project
                </Button>
                <Button onClick={handleRambleAboutFeatures} variant="outline">
                  <MessageCircle className="h-4 w-4 mr-2" />
                  Ramble about Features
                </Button>
              </div>
            </EmptyContent>
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
