import { Badge } from "@/components/ui/badge";
import { DISCIPLINE_CONFIG } from "@/constants/prd";

interface TaskIdDisplayProps {
  taskId: string;
  variant?: "default" | "badge";
  className?: string;
}

type DisciplineKey = keyof typeof DISCIPLINE_CONFIG;

export function TaskIdDisplay({ taskId, variant = "default", className = "" }: TaskIdDisplayProps) {
  const parts = taskId.split("/");
  const [featurePart, disciplinePart, numberPart] = parts;
  const disciplineConfig = disciplinePart && DISCIPLINE_CONFIG[disciplinePart as DisciplineKey];

  if (variant === "badge") {
    return (
      <div className={`flex flex-col items-start leading-tight ${className}`}>
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
    );
  }

  return (
    <div className={`flex flex-col items-start leading-tight ${className}`}>
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
  );
}
