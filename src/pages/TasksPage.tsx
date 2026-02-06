import { MessageSquare, Plus } from "lucide-react";
import { useCallback, useMemo } from "react";
import { PRDBody } from "@/components/prd/PRDBody";
import { PRDHeader } from "@/components/prd/PRDHeader";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { usePRDData } from "@/hooks/usePRDData";
import { usePRDFilters } from "@/hooks/usePRDFilters";
import { useWorkspaceStore } from "@/stores/useWorkspaceStore";
import type { PRDTask } from "@/types/prd";

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
  const { prdData, isLoading: loading, error } = usePRDData();
  const { filters, setters, filteredTasks, allTags, clearFilters } = usePRDFilters(prdData);
  const openTab = useWorkspaceStore((s) => s.openTab);

  const doneTasks = useMemo(() => (prdData ? prdData.tasks.filter((t) => t.status === "done") : []), [prdData]);

  const totalTasks = prdData?.tasks.length ?? 0;
  const progressPercent = totalTasks > 0 ? Math.round((doneTasks.length / totalTasks) * 100) : 0;

  const handleCreateTask = () => {
    openTab({
      type: "task-form",
      title: "Create Task",
      closeable: true,
      data: { mode: "create" },
    });
  };

  const handleYapAboutTasks = () => {
    // TODO: Create YapFormTabContent component (similar to BraindumpFormTabContent)
    // TODO: Default prompt should be: "I want to talk about these tasks: [list existing task IDs]"
    // TODO: User can ramble about what they want to change/add/refine
    // TODO: Send to Claude terminal with MCP tools to update tasks
    // TODO: Invalidate cache and refresh UI when done
    console.log("Yap about tasks clicked - TODO: implement");
    openTab({
      type: "yap-form", // TODO: Add this tab type
      title: "Yap about Tasks",
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

  const handleTaskClick = useCallback(
    (task: PRDTask) => {
      openTab({
        type: "task-detail",
        title: task.title,
        closeable: true,
        data: { entityId: task.id, entity: task },
      });
    },
    [openTab]
  );

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
      <div className="flex-shrink-0">
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
        {totalTasks > 0 && (
          <div className="px-4 pb-2 flex gap-2">
            <Button onClick={handleCreateTask} size="sm">
              <Plus className="h-4 w-4 mr-2" />
              Create Task
            </Button>
            <Button onClick={handleYapAboutTasks} size="sm" variant="outline">
              <MessageSquare className="h-4 w-4 mr-2" />
              Yap about Tasks
            </Button>
          </div>
        )}
      </div>

      <div className="flex-1 min-h-0 overflow-auto">
        <PRDBody
          filteredTasks={filteredTasks}
          totalTasks={totalTasks}
          onTaskClick={handleTaskClick}
          onClearFilters={clearFilters}
          onBraindump={handleBraindumpProject}
          onYap={handleYapAboutTasks}
        />
      </div>
    </div>
  );
}
