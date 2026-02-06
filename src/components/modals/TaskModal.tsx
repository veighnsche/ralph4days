import { useEffect, useRef, useState } from "react";
import type { PRDTask } from "@/types/prd";
import { TaskForm, type TaskFormData } from "../forms/TaskForm";
import { EntityModal } from "./EntityModal";

export interface TaskModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSubmit: (data: TaskFormData) => Promise<void>;
  initialData?: Partial<PRDTask>;
  mode?: "create" | "edit";
}

/**
 * Task create/edit modal.
 * Combines EntityModal wrapper with TaskForm.
 */
export function TaskModal({ open, onOpenChange, onSubmit, initialData, mode = "create" }: TaskModalProps) {
  const [isSubmitting, setIsSubmitting] = useState(false);
  const formRef = useRef<TaskFormData | null>(null);

  const handleFormSubmit = (data: TaskFormData) => {
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
      console.error("Failed to save task:", error);
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
      title={mode === "create" ? "Create Task" : "Edit Task"}
      description={mode === "create" ? "Add a new task to your project" : "Update task details"}
      onSubmit={handleModalSubmit}
      onCancel={handleCancel}
      isSubmitting={isSubmitting}
      submitLabel={mode === "create" ? "Create" : "Update"}
    >
      <TaskForm initialData={initialData} onSubmit={handleFormSubmit} isSubmitting={isSubmitting} />
    </EntityModal>
  );
}
