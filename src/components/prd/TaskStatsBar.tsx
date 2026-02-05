import { Badge } from "@/components/ui/badge";

interface TaskStatsBarProps {
  totalTasks: number;
  doneTasks: number;
  progressPercent: number;
}

export function TaskStatsBar({ totalTasks, doneTasks, progressPercent }: TaskStatsBarProps) {
  const remainingTasks = totalTasks - doneTasks;

  return (
    <div className="flex items-center gap-6 text-xs">
      <div className="flex items-center gap-2">
        <span className="text-[hsl(var(--muted-foreground))]">Total:</span>
        <Badge variant="outline" className="h-5">
          {totalTasks}
        </Badge>
      </div>
      <div className="flex items-center gap-2">
        <span className="text-[hsl(var(--muted-foreground))]">Done:</span>
        <Badge variant="success" className="h-5">
          {doneTasks}
        </Badge>
      </div>
      <div className="flex items-center gap-2">
        <span className="text-[hsl(var(--muted-foreground))]">Remaining:</span>
        <Badge variant="outline" className="h-5">
          {remainingTasks}
        </Badge>
      </div>
      <div className="text-right min-w-[60px]">
        <div className="text-xl font-bold leading-none">{progressPercent}%</div>
        <div className="text-[10px] text-[hsl(var(--muted-foreground))] mt-0.5">Complete</div>
      </div>
    </div>
  );
}
