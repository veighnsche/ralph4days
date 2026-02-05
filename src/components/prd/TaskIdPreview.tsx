import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";

interface TaskIdPreviewProps {
  feature: string;
  discipline: string;
}

export function TaskIdPreview({ feature, discipline }: TaskIdPreviewProps) {
  const [previewId, setPreviewId] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!feature || !discipline) {
      setPreviewId(null);
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
        .then(setPreviewId)
        .catch((err) => {
          console.error("Failed to get preview ID:", err);
          setPreviewId(`${normalized}/${discipline}/?`);
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
      ) : previewId ? (
        <div className="flex flex-col items-start leading-tight">
          {previewId.split("/").map((part, i) => (
            <Badge key={i} variant="outline" className="font-mono text-xs mb-0.5">
              {part}
            </Badge>
          ))}
        </div>
      ) : (
        <span className="text-sm text-muted-foreground">—</span>
      )}
    </div>
  );
}
