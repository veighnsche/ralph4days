import { Circle } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { useDisciplines } from "@/hooks/useDisciplines";
import { useFeatures } from "@/hooks/useFeatures";
import type { PRDTask } from "@/types/prd";

interface TaskIdDisplayProps {
  task: PRDTask;
  variant?: "default" | "badge";
  className?: string;
}

function formatTaskId(id: number): string {
  if (id > 999) {
    return id.toString();
  }
  return `#${id.toString().padStart(3, "0")}`;
}

export function TaskIdDisplay({ task, variant = "default", className = "" }: TaskIdDisplayProps) {
  const { configMap: disciplineMap } = useDisciplines();
  const { configMap: featureMap } = useFeatures();
  const disciplineConfig = disciplineMap[task.discipline];
  const featureConfig = featureMap.get(task.feature);
  const DisciplineIcon = disciplineConfig?.icon || Circle;

  const featureAcronym = featureConfig?.acronym || task.feature;
  const disciplineAcronym = disciplineConfig?.acronym || task.discipline;
  const formattedId = formatTaskId(task.id);

  if (variant === "badge") {
    return (
      <div className={`flex items-center gap-2 ${className}`}>
        {/* Simple icon box */}
        <div
          className="w-8 h-8 flex items-center justify-center rounded border"
          style={{
            backgroundColor: disciplineConfig?.bgColor || "transparent",
            borderColor: disciplineConfig?.color || "transparent",
          }}
        >
          <DisciplineIcon className="w-4 h-4" style={{ color: disciplineConfig?.color }} />
        </div>

        {/* Badge variant */}
        <div className="flex flex-col items-start leading-tight">
          <Badge variant="outline" className="font-mono text-xs mb-0.5">
            {featureAcronym}
          </Badge>
          <Badge
            variant="outline"
            className="font-mono text-xs mb-0.5"
            style={{
              borderColor: disciplineConfig?.color,
              backgroundColor: disciplineConfig?.bgColor,
              color: disciplineConfig?.color,
            }}
          >
            {disciplineAcronym}
          </Badge>
          <Badge variant="outline" className="font-mono text-xs mb-0.5">
            {formattedId}
          </Badge>
        </div>
      </div>
    );
  }

  // Default variant - SUPER SIMPLE
  return (
    <div className={`flex items-center gap-2 ${className}`}>
      {/* Simple icon box */}
      <div
        className="w-8 h-8 flex items-center justify-center rounded border"
        style={{
          backgroundColor: disciplineConfig?.bgColor || "transparent",
          borderColor: disciplineConfig?.color || "transparent",
        }}
      >
        <DisciplineIcon className="w-4 h-4" style={{ color: disciplineConfig?.color }} />
      </div>

      {/* Simple text */}
      <div className="flex flex-col items-start leading-tight font-mono">
        <span className="text-xs text-muted-foreground">{featureAcronym}</span>
        <span className="text-xs font-medium" style={{ color: disciplineConfig?.color }}>
          {disciplineAcronym}
        </span>
        <span className="text-xs text-muted-foreground">{formattedId}</span>
      </div>
    </div>
  );
}
