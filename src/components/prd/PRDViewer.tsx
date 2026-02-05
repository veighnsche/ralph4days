import { useMemo } from "react";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Skeleton } from "@/components/ui/skeleton";
import { usePRDData } from "@/hooks/usePRDData";
import { usePRDFilters } from "@/hooks/usePRDFilters";
import { useSidebarNavigation } from "@/hooks/useSidebarNavigation";
import { PRDHeader } from "./PRDHeader";
import { PRDBody } from "./PRDBody";
import { TaskDetailSidebar } from "./TaskDetailSidebar";

export function PRDViewer() {
  const { prdData, loading, error } = usePRDData();
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
      />

      <PRDBody filteredTasks={filteredTasks} onTaskClick={handleTaskClick} onClearFilters={clearFilters} />

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
