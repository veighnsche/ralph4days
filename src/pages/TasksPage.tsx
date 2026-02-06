import { useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { Plus } from "lucide-react";
import { useMemo, useState } from "react";
import type { TaskFormData } from "@/components/forms/TaskForm";
import { TaskModal } from "@/components/modals/TaskModal";
import { PRDBody } from "@/components/prd/PRDBody";
import { PRDHeader } from "@/components/prd/PRDHeader";
import { TaskDetailSidebar } from "@/components/prd/TaskDetailSidebar";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { useDisciplines } from "@/hooks/useDisciplines";
import { useInvoke } from "@/hooks/useInvoke";
import { usePRDData } from "@/hooks/usePRDData";
import { usePRDFilters } from "@/hooks/usePRDFilters";
import { useSidebarNavigation } from "@/hooks/useSidebarNavigation";

interface FeatureConfig {
  name: string;
  display_name: string;
  acronym: string;
}

export function TasksPage() {
  const queryClient = useQueryClient();
  const { prdData, isLoading: loading, error } = usePRDData();
  const { filters, setters, filteredTasks, allTags, clearFilters } = usePRDFilters(prdData);
  const { selectedTask, sidebarOpen, handleTaskClick, handleNavigateNext, handleNavigatePrev, setSidebarOpen } =
    useSidebarNavigation(filteredTasks);
  const { configMap: disciplineConfigMap } = useDisciplines();
  const { data: featureConfigs = [] } = useInvoke<FeatureConfig[]>("get_features_config");
  const [createModalOpen, setCreateModalOpen] = useState(false);

  const doneTasks = useMemo(() => (prdData ? prdData.tasks.filter((t) => t.status === "done") : []), [prdData]);

  const totalTasks = prdData?.tasks.length ?? 0;
  const progressPercent = totalTasks > 0 ? Math.round((doneTasks.length / totalTasks) * 100) : 0;

  const handleCreateTask = async (data: TaskFormData) => {
    // Get acronyms from configs
    const featureConfig = featureConfigs.find((f) => f.name === data.feature);
    const disciplineConfig = disciplineConfigMap[data.discipline];

    if (!featureConfig) {
      throw new Error(`Feature ${data.feature} not found`);
    }
    if (!disciplineConfig) {
      throw new Error(`Discipline ${data.discipline} not found`);
    }

    await invoke("create_task", {
      feature: data.feature,
      discipline: data.discipline,
      title: data.title,
      description: data.description || null,
      priority: data.priority || null,
      tags: data.tags,
      dependsOn: data.depends_on.length > 0 ? data.depends_on : null,
      acceptanceCriteria: data.acceptance_criteria.length > 0 ? data.acceptance_criteria : null,
      featureAcronym: featureConfig.acronym,
      disciplineAcronym: disciplineConfig.acronym,
    });
    await queryClient.invalidateQueries({ queryKey: ["get_prd_content"] });
    await queryClient.invalidateQueries({ queryKey: ["get_features"] });
    await queryClient.invalidateQueries({ queryKey: ["get_features_config"] });
  };

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
        <div className="px-4 pb-2">
          <Button onClick={() => setCreateModalOpen(true)} size="sm">
            <Plus className="h-4 w-4 mr-2" />
            Create Task
          </Button>
        </div>
      </div>

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

      <TaskModal open={createModalOpen} onOpenChange={setCreateModalOpen} onSubmit={handleCreateTask} mode="create" />
    </div>
  );
}
