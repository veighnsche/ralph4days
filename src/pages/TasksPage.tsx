import { useCallback } from "react";
import { PageContent, PageHeader, PageLayout } from "@/components/layout/PageLayout";
import { PRDBody } from "@/components/prd/PRDBody";
import { PRDHeader } from "@/components/prd/PRDHeader";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Skeleton } from "@/components/ui/skeleton";
import { useInvoke } from "@/hooks/useInvoke";
import { usePRDData } from "@/hooks/usePRDData";
import { usePRDFilters } from "@/hooks/usePRDFilters";
import { useWorkspaceStore } from "@/stores/useWorkspaceStore";
import type { ProjectInfo, ProjectProgress, Task } from "@/types/prd";

/**
 * PLANNED: Task-Bound Terminal System
 *
 * Future implementation will bind tasks to specific terminal/output tabs:
 *
 * TODO: 1. Add Play Button to Tasks
 *    - Add play icon button next to each task in PlaylistView
 *    - onClick: open new terminal, bind to task, send task prompt
 *    - Store binding in WorkspaceTab.data: { taskId: string }
 *
 * TODO: 2. Generate Task Prompt
 *    - Create prompt_builder function that includes:
 *      - Task title, description, acceptance criteria
 *      - Feature context (what feature this belongs to)
 *      - Discipline context (coding standards, tools)
 *      - Dependencies (other tasks that are done)
 *    - Send prompt to bound terminal automatically
 *
 * TODO: 3. Terminal Lifecycle Hooks
 *    - Listen for ralph://pty_closed event
 *    - When terminal closes, check if it has taskId binding
 *    - Update task status (in_progress -> pending or done)
 *    - Clear binding from store
 *
 * TODO: 4. Task Status Synchronization
 *    - Parse terminal output for success/failure indicators
 *    - Update task status in real-time
 *    - Show progress in task card (e.g., "Running in Terminal 3")
 *
 * TODO: 5. Play Button States
 *    - Disabled if task is "done"
 *    - Shows "Resume" if task was interrupted
 *    - Shows "Running..." if task is active
 *    - Panics if bound terminal was closed unexpectedly
 *
 * Reference: Proof-of-concept tested 2026-02-06 with "Open Claude" + "Type in Bound" buttons
 * Pattern established: openTab() -> wait -> send_terminal_input() -> closeTab()
 */

export function TasksPage() {
  const { tasks, isLoading: tasksLoading, error } = usePRDData();
  const { data: progress } = useInvoke<ProjectProgress>("get_project_progress");
  const { data: allTags = [] } = useInvoke<string[]>("get_all_tags");
  const { data: projectInfo } = useInvoke<ProjectInfo>("get_project_info");
  const { filters, setters, filteredTasks, clearFilters } = usePRDFilters(tasks, allTags);
  const openTab = useWorkspaceStore((s) => s.openTab);

  const totalTasks = progress?.totalTasks ?? 0;
  const doneTasks = progress?.doneTasks ?? 0;
  const progressPercent = progress?.progressPercent ?? 0;

  const handleBraindumpProject = () => {
    openTab({
      type: "braindump-form",
      title: "Braindump Project",
      closeable: true,
    });
  };

  const handleYapAboutTasks = () => {
    openTab({
      type: "braindump-form",
      title: "Yap about Tasks",
      closeable: true,
    });
  };

  const handleTaskClick = useCallback(
    (task: Task) => {
      openTab({
        type: "task-detail",
        title: task.title,
        closeable: true,
        data: { entityId: task.id, entity: task },
      });
    },
    [openTab]
  );

  const loading = tasksLoading;

  if (loading) {
    return (
      <PageLayout>
        <PageHeader>
          <Skeleton className="h-[200px]" />
        </PageHeader>
        <PageContent>
          <div className="space-y-4">
            <Skeleton className="h-[60px]" />
            <Skeleton className="h-[60px]" />
            <Skeleton className="h-[60px]" />
          </div>
        </PageContent>
      </PageLayout>
    );
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
    );
  }

  if (!tasks) {
    return (
      <PageLayout>
        <PageContent>
          <Alert>
            <AlertDescription>No task data available</AlertDescription>
          </Alert>
        </PageContent>
      </PageLayout>
    );
  }

  return (
    <PageLayout>
      <PageHeader>
        <PRDHeader
          project={projectInfo ?? { title: "Project" }}
          totalTasks={totalTasks}
          doneTasks={doneTasks}
          progressPercent={progressPercent}
          filteredCount={filteredTasks.length}
          filters={filters}
          setters={setters}
          allTags={allTags}
          onClearFilters={clearFilters}
        />
      </PageHeader>

      <PageContent>
        <PRDBody
          filteredTasks={filteredTasks}
          totalTasks={totalTasks}
          onTaskClick={handleTaskClick}
          onClearFilters={clearFilters}
          onBraindump={handleBraindumpProject}
          onYap={handleYapAboutTasks}
        />
      </PageContent>
    </PageLayout>
  );
}
