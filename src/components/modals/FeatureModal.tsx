import { useCallback, useEffect, useRef, useState } from "react";
import { toast } from "sonner";
import type { Feature } from "@/types/prd";
import { FeatureForm, type FeatureFormData } from "../forms/FeatureForm";
import { EntityModal } from "./EntityModal";

export interface FeatureModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSubmit: (data: FeatureFormData) => Promise<void>;
  initialData?: Partial<Feature>;
  mode?: "create" | "edit";
}

/**
 * Feature create/edit modal.
 * Combines EntityModal wrapper with FeatureForm.
 */
export function FeatureModal({ open, onOpenChange, onSubmit, initialData, mode = "create" }: FeatureModalProps) {
  const [isSubmitting, setIsSubmitting] = useState(false);
  const formRef = useRef<FeatureFormData | null>(null);

  const handleFormChange = useCallback((data: FeatureFormData) => {
    formRef.current = data;
  }, []);

  const handleModalSubmit = async () => {
    if (!formRef.current) return;

    setIsSubmitting(true);
    try {
      await onSubmit(formRef.current);
      toast.success(mode === "create" ? "Feature created" : "Feature updated");
      onOpenChange(false);
      formRef.current = null;
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Failed to save feature");
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleCancel = () => {
    formRef.current = null;
  };

  useEffect(() => {
    if (!open) {
      formRef.current = null;
      setIsSubmitting(false);
    }
  }, [open]);

  return (
    <EntityModal
      open={open}
      onOpenChange={onOpenChange}
      title={mode === "create" ? "Create Feature" : "Edit Feature"}
      description={mode === "create" ? "Add a new feature to your project" : "Update feature details"}
      onSubmit={handleModalSubmit}
      onCancel={handleCancel}
      isSubmitting={isSubmitting}
      submitLabel={mode === "create" ? "Create" : "Update"}
    >
      <FeatureForm initialData={initialData} onChange={handleFormChange} disabled={isSubmitting} />
    </EntityModal>
  );
}
