import { Badge } from "@/components/ui/badge";

interface TaskStatsBarProps {
  totalTasks: number;
  doneTasks: number;
  progressPercent: number;
}

export function TaskStatsBar({ totalTasks, doneTasks, progressPercent }: TaskStatsBarProps) {
  const remainingTasks = totalTasks - doneTasks;

  return (
    <div className="flex items-center gap-3 text-xs">
      <div className="flex items-center gap-2">
        <span className="text-muted-foreground">Total:</span>
        <Badge variant="outline" className="h-4">
          {totalTasks}
        </Badge>
      </div>
      <div className="flex items-center gap-2">
        <span className="text-muted-foreground">Done:</span>
        <Badge variant="success" className="h-4">
          {doneTasks}
        </Badge>
      </div>
      <div className="flex items-center gap-2">
        <span className="text-muted-foreground">Remaining:</span>
        <Badge variant="outline" className="h-4">
          {remainingTasks}
        </Badge>
      </div>
      <div className="text-right min-w-[60px]">
        <div className="text-base font-bold leading-none">{progressPercent}%</div>
        <div className="text-[10px] text-muted-foreground mt-0.5">Complete</div>
      </div>
    </div>
  );
}
