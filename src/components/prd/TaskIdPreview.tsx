import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { Skeleton } from "@/components/ui/skeleton";
import type { PRDTask } from "@/types/prd";
import { TaskIdDisplay } from "./TaskIdDisplay";

interface TaskIdPreviewProps {
  feature: string;
  discipline: string;
}

export function TaskIdPreview({ feature, discipline }: TaskIdPreviewProps) {
  const [previewTask, setPreviewTask] = useState<PRDTask | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!feature || !discipline) {
      setPreviewTask(null);
      return;
    }

    const normalized = feature.toLowerCase().replace(/\s+/g, "-");

    // Debounce: wait 300ms after user stops typing
    const timer = setTimeout(() => {
      setLoading(true);
      invoke<string>("get_next_task_id", {
        feature: normalized,
        discipline,
      })
        .then((id) => {
          // Create a minimal task object for preview
          const task: PRDTask = {
            id: parseInt(id, 10),
            feature: normalized,
            discipline: discipline as PRDTask["discipline"],
            title: "Preview task",
            status: "pending",
          };
          setPreviewTask(task);
        })
        .catch((err) => {
          console.error("Failed to get preview ID:", err);
          setPreviewTask(null);
        })
        .finally(() => setLoading(false));
    }, 300);

    return () => clearTimeout(timer);
  }, [feature, discipline]);

  if (!feature && !discipline) {
    return <div className="text-sm text-muted-foreground">Fill in feature and discipline to see task ID preview</div>;
  }

  return (
    <div className="flex items-start gap-2">
      <span className="text-sm font-medium">Task ID:</span>
      {loading ? (
        <Skeleton className="h-12 w-24" />
      ) : previewTask ? (
        <TaskIdDisplay task={previewTask} variant="badge" />
      ) : (
        <span className="text-sm text-muted-foreground">—</span>
      )}
    </div>
  );
}
