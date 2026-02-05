import { Circle } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { ItemMedia } from "@/components/ui/item";
import { DISCIPLINE_CONFIG, STATUS_CONFIG } from "@/constants/prd";

interface TaskIdDisplayProps {
  taskId: string;
  variant?: "default" | "badge";
  status?: "pending" | "in_progress" | "done" | "blocked" | "skipped";
  className?: string;
}

type DisciplineKey = keyof typeof DISCIPLINE_CONFIG;

export function TaskIdDisplay({ taskId, variant = "default", status = "pending", className = "" }: TaskIdDisplayProps) {
  const parts = taskId.split("/");
  const [featurePart, disciplinePart, numberPart] = parts;
  const disciplineConfig = disciplinePart && DISCIPLINE_CONFIG[disciplinePart as DisciplineKey];
  const DisciplineIcon = disciplineConfig?.icon || Circle;
  const statusConfig = STATUS_CONFIG[status];

  if (variant === "badge") {
    return (
      <div className={`flex items-center gap-2 ${className}`}>
        <ItemMedia variant="icon" style={{ backgroundColor: statusConfig.bgColor }}>
          <DisciplineIcon style={{ color: statusConfig.color, stroke: statusConfig.color }} />
        </ItemMedia>
        <div className="flex flex-col items-start leading-tight">
          <Badge variant="outline" className="font-mono text-xs mb-0.5">
            {featurePart}
          </Badge>
          {disciplinePart && (
            <Badge
              variant="outline"
              className="font-mono text-xs mb-0.5"
              style={{
                borderColor: disciplineConfig?.color,
                backgroundColor: disciplineConfig?.bgColor,
                color: disciplineConfig?.color,
              }}
            >
              {disciplinePart}
            </Badge>
          )}
          {numberPart && (
            <Badge variant="outline" className="font-mono text-xs mb-0.5">
              {numberPart}
            </Badge>
          )}
        </div>
      </div>
    );
  }

  return (
    <div className={`flex items-center gap-2 ${className}`}>
      <ItemMedia variant="icon" style={{ backgroundColor: statusConfig.bgColor }}>
        <DisciplineIcon style={{ color: statusConfig.color, stroke: statusConfig.color }} />
      </ItemMedia>
      <div className="flex flex-col items-start leading-tight">
        <span className="text-xs font-mono text-[hsl(var(--muted-foreground))]">{featurePart}</span>
        {disciplinePart && (
          <span
            className="text-xs font-mono font-medium"
            style={{
              color: disciplineConfig?.color,
            }}
          >
            {disciplinePart}
          </span>
        )}
        {numberPart && <span className="text-xs font-mono text-[hsl(var(--muted-foreground))]">{numberPart}</span>}
      </div>
    </div>
  );
}
