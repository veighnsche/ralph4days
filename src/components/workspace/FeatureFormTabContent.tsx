import { useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { Puzzle } from "lucide-react";
import { useCallback, useRef, useState } from "react";
import { toast } from "sonner";
import { FeatureForm, type FeatureFormData } from "@/components/forms/FeatureForm";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { useTabMeta } from "@/hooks/useTabMeta";
import type { WorkspaceTab } from "@/stores/useWorkspaceStore";
import { useWorkspaceStore } from "@/stores/useWorkspaceStore";

export function FeatureFormTabContent({ tab }: { tab: WorkspaceTab }) {
  const mode = tab.data?.mode ?? "create";
  useTabMeta(tab.id, mode === "create" ? "Create Feature" : "Edit Feature", Puzzle);
  const queryClient = useQueryClient();
  const { closeTab } = useWorkspaceStore();

  const [isSubmitting, setIsSubmitting] = useState(false);
  const formRef = useRef<FeatureFormData | null>(null);

  const handleFormChange = useCallback((data: FeatureFormData) => {
    formRef.current = data;
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!formRef.current) return;

    const data = formRef.current;

    setIsSubmitting(true);
    try {
      await invoke("create_feature", {
        name: data.name || data.display_name,
        displayName: data.display_name,
        acronym: data.acronym,
        description: data.description || null,
      });
      await queryClient.invalidateQueries({ queryKey: ["get_features"] });
      await queryClient.invalidateQueries({ queryKey: ["get_features_config"] });
      toast.success("Feature created");
      closeTab(tab.id);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Failed to create feature");
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="h-full flex flex-col">
      <div className="px-4 py-3 flex-shrink-0">
        <h2 className="text-lg font-semibold">{mode === "create" ? "Create Feature" : "Edit Feature"}</h2>
        <p className="text-sm text-muted-foreground">
          {mode === "create" ? "Add a new feature to your project" : "Update feature details"}
        </p>
      </div>
      <Separator />
      <ScrollArea className="flex-1 min-h-0">
        <div className="px-4">
          <FeatureForm initialData={tab.data?.entity as never} onChange={handleFormChange} disabled={isSubmitting} />
        </div>
      </ScrollArea>
      <Separator />
      <div className="px-4 py-3 flex justify-end gap-2 flex-shrink-0">
        <Button type="button" variant="outline" size="lg" onClick={() => closeTab(tab.id)} disabled={isSubmitting}>
          Cancel
        </Button>
        <Button type="submit" size="lg" disabled={isSubmitting}>
          {isSubmitting ? "Saving..." : mode === "create" ? "Create" : "Update"}
        </Button>
      </div>
    </form>
  );
}
