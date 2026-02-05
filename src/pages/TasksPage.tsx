import { useMemo } from "react";
import { PRDBody } from "@/components/prd/PRDBody";
import { PRDHeader } from "@/components/prd/PRDHeader";
import { TaskDetailSidebar } from "@/components/prd/TaskDetailSidebar";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Skeleton } from "@/components/ui/skeleton";
import { usePRDData } from "@/hooks/usePRDData";
import { usePRDFilters } from "@/hooks/usePRDFilters";
import { useSidebarNavigation } from "@/hooks/useSidebarNavigation";

export function TasksPage() {
  const { prdData, isLoading: loading, error, refetch: refresh } = usePRDData();
  const { filters, setters, filteredTasks, allTags, clearFilters } = usePRDFilters(prdData);
  const { selectedTask, sidebarOpen, handleTaskClick, handleNavigateNext, handleNavigatePrev, setSidebarOpen } =
    useSidebarNavigation(filteredTasks);

  const doneTasks = useMemo(() => (prdData ? prdData.tasks.filter((t) => t.status === "done") : []), [prdData]);

  const totalTasks = prdData?.tasks.length ?? 0;
  const progressPercent = totalTasks > 0 ? Math.round((doneTasks.length / totalTasks) * 100) : 0;

  if (loading) {
    return (
      <div className="h-full flex flex-col overflow-hidden">
        <div className="flex-shrink-0 p-4">
          <Skeleton className="h-[200px]" />
        </div>
        <div className="flex-1 p-4 space-y-4">
          <Skeleton className="h-[60px]" />
          <Skeleton className="h-[60px]" />
          <Skeleton className="h-[60px]" />
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

  if (!prdData) {
    return (
      <div className="h-full p-4">
        <Alert>
          <AlertDescription>No PRD data available</AlertDescription>
        </Alert>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col overflow-hidden">
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
