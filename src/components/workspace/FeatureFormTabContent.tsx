import { zodResolver } from "@hookform/resolvers/zod";
import { Puzzle } from "lucide-react";
import { useForm } from "react-hook-form";
import { toast } from "sonner";
import { FeatureFormFields } from "@/components/forms/FeatureForm";
import { useInvokeMutation } from "@/hooks/useInvokeMutation";
import { type FeatureFormData, featureSchema } from "@/lib/schemas";
import type { WorkspaceTab } from "@/stores/useWorkspaceStore";
import { useWorkspaceStore } from "@/stores/useWorkspaceStore";
import { EntityFormTabContent } from "./EntityFormTabContent";

export function FeatureFormTabContent({ tab }: { tab: WorkspaceTab }) {
  const { closeTab } = useWorkspaceStore();

  const form = useForm<FeatureFormData>({
    resolver: zodResolver(featureSchema),
    defaultValues: {
      name: "",
      displayName: "",
      acronym: "",
      description: "",
    },
  });

  const createFeature = useInvokeMutation<Record<string, unknown>>("create_feature", {
    invalidateKeys: [["get_features"], ["get_features_config"], ["get_tasks"], ["get_feature_stats"]],
    onSuccess: () => {
      toast.success("Feature created");
      closeTab(tab.id);
    },
    onError: (err) => toast.error(err.message),
  });

  const handleSubmit = (data: FeatureFormData) => {
    createFeature.mutate({
      name: data.name || data.displayName,
      displayName: data.displayName,
      acronym: data.acronym,
      description: data.description || null,
    });
  };

  return (
    <EntityFormTabContent
      tab={tab}
      icon={Puzzle}
      entityName="Feature"
      form={form}
      onSubmit={handleSubmit}
      isPending={createFeature.isPending}
    >
      <FeatureFormFields disabled={createFeature.isPending} />
    </EntityFormTabContent>
  );
}
