import { Layers, Plus } from "lucide-react";
import { useMemo } from "react";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardTitle } from "@/components/ui/card";
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty";
import { ItemGroup, ItemSeparator } from "@/components/ui/item";
import { Progress } from "@/components/ui/progress";
import { Skeleton } from "@/components/ui/skeleton";
import { useDisciplines } from "@/hooks/useDisciplines";
import { usePRDData } from "@/hooks/usePRDData";
import { useWorkspaceStore } from "@/stores/useWorkspaceStore";

export function DisciplinesPage() {
  const { prdData, isLoading: prdLoading, error: prdError } = usePRDData();
  const { disciplines } = useDisciplines();
  const openTab = useWorkspaceStore((s) => s.openTab);

  // Calculate task counts per discipline
  const disciplineStats = useMemo(() => {
    if (!prdData) return new Map();
    const stats = new Map<string, { total: number; done: number; pending: number; inProgress: number }>();

    for (const task of prdData.tasks) {
      if (!stats.has(task.discipline)) {
        stats.set(task.discipline, { total: 0, done: 0, pending: 0, inProgress: 0 });
      }
      const stat = stats.get(task.discipline)!;
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

  const loading = prdLoading || disciplines.length === 0;

  const handleCreateDiscipline = () => {
    openTab({
      type: "discipline-form",
      title: "Create Discipline",
      closeable: true,
      data: { mode: "create" },
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

  if (prdError) {
    return (
      <div className="h-full p-4">
        <Alert variant="destructive">
          <AlertDescription>{prdError}</AlertDescription>
        </Alert>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col overflow-hidden">
      {/* Disciplines Header */}
      <div className="flex-shrink-0 p-4 pb-0">
        <Card className="py-3">
          <CardContent className="space-y-3">
            <div className="flex items-center justify-between gap-4">
              <div className="flex items-center gap-2 flex-1">
                <Layers className="h-4 w-4" />
                <CardTitle className="text-base">Disciplines</CardTitle>
              </div>
              <Button onClick={handleCreateDiscipline} size="sm" variant="outline">
                <Plus className="h-4 w-4 mr-2" />
                Create Discipline
              </Button>
              <div className="flex items-center gap-4">
                <div className="text-right">
                  <div className="text-sm font-medium">
                    Total: <span className="text-[hsl(var(--muted-foreground))]">{disciplines.length}</span>
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
                  <div className="text-lg font-semibold leading-none">{progressPercent}%</div>
                  <div className="text-[10px] text-[hsl(var(--muted-foreground))]">Complete</div>
                </div>
              </div>
            </div>
            <Progress value={progressPercent} className="h-1.5" />
            <CardDescription className="text-xs">Work categories and their task distribution</CardDescription>
          </CardContent>
        </Card>
      </div>

      {/* Disciplines List */}
      <div className="flex-1 min-h-0 overflow-auto p-4">
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
              const Icon = discipline.icon;
              const stats = disciplineStats.get(discipline.name) || { total: 0, done: 0, pending: 0, inProgress: 0 };
              const progress = stats.total > 0 ? Math.round((stats.done / stats.total) * 100) : 0;

              return (
                <div key={discipline.name}>
                  <div className="p-4 hover:bg-muted/50 transition-colors">
                    <div className="flex items-start justify-between gap-4">
                      <div className="flex items-start gap-3 flex-1 min-w-0">
                        <div
                          className="p-2 rounded-md shrink-0"
                          style={{
                            backgroundColor: discipline.bgColor,
                            color: discipline.color,
                          }}
                        >
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
                        <div className="text-lg font-semibold">{progress}%</div>
                        <div className="text-xs text-muted-foreground">complete</div>
                      </div>
                    </div>
                  </div>
                  {index < disciplines.length - 1 && <ItemSeparator />}
                </div>
              );
            })}
          </ItemGroup>
        )}
      </div>
    </div>
  );
}
