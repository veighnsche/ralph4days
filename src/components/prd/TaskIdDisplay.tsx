import { Circle } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { DISCIPLINE_CONFIG } from "@/constants/prd";
import type { PRDTask } from "@/types/prd";

interface TaskIdDisplayProps {
  task: PRDTask;
  variant?: "default" | "badge";
  className?: string;
}

export function TaskIdDisplay({ task, variant = "default", className = "" }: TaskIdDisplayProps) {
  const disciplineConfig = DISCIPLINE_CONFIG[task.discipline];
  const DisciplineIcon = disciplineConfig?.icon || Circle;

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
            {task.feature}
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
            {task.discipline}
          </Badge>
          <Badge variant="outline" className="font-mono text-xs mb-0.5">
            {task.id}
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
        <span className="text-xs text-muted-foreground">{task.feature}</span>
        <span className="text-xs font-medium" style={{ color: disciplineConfig?.color }}>
          {task.discipline}
        </span>
        <span className="text-xs text-muted-foreground">{task.id}</span>
      </div>
    </div>
  );
}
