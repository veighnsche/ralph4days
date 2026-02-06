import { useEffect, useRef, useState } from "react";
import type { DisciplineConfig } from "@/hooks/useDisciplines";
import { DisciplineForm, type DisciplineFormData } from "../forms/DisciplineForm";
import { EntityModal } from "./EntityModal";

export interface DisciplineModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSubmit: (data: DisciplineFormData) => Promise<void>;
  initialData?: Partial<DisciplineConfig>;
  mode?: "create" | "edit";
}

/**
 * Discipline create/edit modal.
 * Combines EntityModal wrapper with DisciplineForm.
 */
export function DisciplineModal({ open, onOpenChange, onSubmit, initialData, mode = "create" }: DisciplineModalProps) {
  const [isSubmitting, setIsSubmitting] = useState(false);
  const formRef = useRef<DisciplineFormData | null>(null);

  const handleFormSubmit = (data: DisciplineFormData) => {
    formRef.current = data;
  };

  const handleModalSubmit = async () => {
    if (!formRef.current) return;

    setIsSubmitting(true);
    try {
      await onSubmit(formRef.current);
      onOpenChange(false);
      formRef.current = null;
    } catch (error) {
      console.error("Failed to save discipline:", error);
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
      title={mode === "create" ? "Create Discipline" : "Edit Discipline"}
      description={mode === "create" ? "Add a new discipline to your project" : "Update discipline details"}
      onSubmit={handleModalSubmit}
      onCancel={handleCancel}
      isSubmitting={isSubmitting}
      submitLabel={mode === "create" ? "Create" : "Update"}
    >
      <DisciplineForm initialData={initialData} onSubmit={handleFormSubmit} isSubmitting={isSubmitting} />
    </EntityModal>
  );
}
