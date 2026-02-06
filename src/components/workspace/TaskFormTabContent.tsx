import { useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { ClipboardList } from "lucide-react";
import { useCallback, useRef, useState } from "react";
import { toast } from "sonner";
import { TaskForm, type TaskFormData } from "@/components/forms/TaskForm";
import { Button } from "@/components/ui/button";
import { FormDescription, FormHeader, FormTitle } from "@/components/ui/form-header";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { useDisciplines } from "@/hooks/useDisciplines";
import { useInvoke } from "@/hooks/useInvoke";
import { useTabMeta } from "@/hooks/useTabMeta";
import type { WorkspaceTab } from "@/stores/useWorkspaceStore";
import { useWorkspaceStore } from "@/stores/useWorkspaceStore";

interface FeatureConfig {
  name: string;
  display_name: string;
  acronym: string;
}

export function TaskFormTabContent({ tab }: { tab: WorkspaceTab }) {
  const mode = tab.data?.mode ?? "create";
  useTabMeta(tab.id, mode === "create" ? "Create Task" : "Edit Task", ClipboardList);
  const queryClient = useQueryClient();
  const { closeTab } = useWorkspaceStore();
  const { configMap: disciplineConfigMap } = useDisciplines();
  const { data: featureConfigs = [] } = useInvoke<FeatureConfig[]>("get_features_config");

  const [isSubmitting, setIsSubmitting] = useState(false);
  const formRef = useRef<TaskFormData | null>(null);

  const handleFormChange = useCallback((data: TaskFormData) => {
    formRef.current = data;
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!formRef.current) return;

    const data = formRef.current;
    const featureConfig = featureConfigs.find((f) => f.name === data.feature);
    const disciplineConfig = disciplineConfigMap[data.discipline];

    if (!featureConfig) {
      toast.error(`Feature ${data.feature} not found`);
      return;
    }
    if (!disciplineConfig) {
      toast.error(`Discipline ${data.discipline} not found`);
      return;
    }

    setIsSubmitting(true);
    try {
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
      toast.success("Task created");
      closeTab(tab.id);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Failed to create task");
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="h-full flex flex-col">
      <div className="px-4 flex-shrink-0">
        <FormHeader>
          <FormTitle>{mode === "create" ? "Create Task" : "Edit Task"}</FormTitle>
          <FormDescription>
            {mode === "create" ? "Add a new task to your project" : "Update task details"}
          </FormDescription>
        </FormHeader>
      </div>
      <Separator />
      <ScrollArea className="flex-1 min-h-0">
        <div className="px-4">
          <TaskForm initialData={tab.data?.entity as never} onChange={handleFormChange} disabled={isSubmitting} />
        </div>
      </ScrollArea>
      <Separator />
      <div className="px-3 py-1.5 flex justify-end gap-2 flex-shrink-0">
        <Button type="button" variant="outline" size="default" onClick={() => closeTab(tab.id)} disabled={isSubmitting}>
          Cancel
        </Button>
        <Button type="submit" size="default" disabled={isSubmitting}>
          {isSubmitting ? "Saving..." : mode === "create" ? "Create" : "Update"}
        </Button>
      </div>
    </form>
  );
}
