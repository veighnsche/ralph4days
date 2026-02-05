import { useMemo } from "react";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Skeleton } from "@/components/ui/skeleton";
import { usePRDData } from "@/hooks/usePRDData";
import { usePRDFilters } from "@/hooks/usePRDFilters";
import { useSidebarNavigation } from "@/hooks/useSidebarNavigation";
import { PRDBody } from "./PRDBody";
import { PRDHeader } from "./PRDHeader";
import { TaskDetailSidebar } from "./TaskDetailSidebar";

export function PRDViewer() {
  const { prdData, loading, error, refresh, usingMockData } = usePRDData();
  const { filters, setters, filteredTasks, allTags, clearFilters } = usePRDFilters(prdData);
  const { selectedTask, sidebarOpen, handleTaskClick, handleNavigateNext, handleNavigatePrev, setSidebarOpen } =
    useSidebarNavigation(filteredTasks);

  const doneTasks = useMemo(() => (prdData ? prdData.tasks.filter((t) => t.status === "done") : []), [prdData]);

  const totalTasks = prdData?.tasks.length ?? 0;
  const progressPercent = totalTasks > 0 ? Math.round((doneTasks.length / totalTasks) * 100) : 0;

  if (loading) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-[100px]" />
        <Skeleton className="h-[60px]" />
        <Skeleton className="h-[60px]" />
        <Skeleton className="h-[60px]" />
      </div>
    );
  }

  if (error) {
    return (
      <Alert variant="destructive">
        <AlertDescription>{error}</AlertDescription>
      </Alert>
    );
  }

  if (!prdData) {
    return (
      <Alert>
        <AlertDescription>No PRD data available</AlertDescription>
      </Alert>
    );
  }

  return (
    <div className="h-full flex flex-col overflow-hidden">
      {usingMockData && (
        <Alert variant="default" className="mb-2 rounded-none border-x-0 border-t-0 bg-amber-50 dark:bg-amber-950/20">
          <AlertDescription className="text-xs">
            ⚠️ <strong>Mock Mode:</strong> Using sample data. Tauri backend not detected. Start the backend with{" "}
            <code className="px-1 py-0.5 rounded bg-amber-100 dark:bg-amber-900">just dev</code> for live data.
          </AlertDescription>
        </Alert>
      )}

      <PRDHeader
        project={prdData.project}
        totalTasks={totalTasks}
        doneTasks={doneTasks.length}
        progressPercent={progressPercent}
        filteredCount={filteredTasks.length}
        filters={filters}
        setters={setters}
        allTags={allTags}
        onClearFilters={clearFilters}
        onRefresh={refresh}
      />

      <div className="flex-1 min-h-0 overflow-auto">
        <PRDBody filteredTasks={filteredTasks} onTaskClick={handleTaskClick} onClearFilters={clearFilters} />
      </div>

      <TaskDetailSidebar
        task={selectedTask}
        open={sidebarOpen}
        onClose={() => setSidebarOpen(false)}
        onNavigateNext={handleNavigateNext}
        onNavigatePrev={handleNavigatePrev}
      />
    </div>
  );
}
