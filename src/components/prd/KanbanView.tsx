import { memo, useMemo } from "react";
import { TooltipProvider } from "@/components/ui/tooltip";
import { COLUMN_DEFINITIONS } from "@/constants/prd";
import type { PRDTask } from "@/types/prd";
import { KanbanColumn } from "./KanbanColumn";

interface KanbanViewProps {
  tasks: PRDTask[];
  onTaskClick: (task: PRDTask) => void;
}

export const KanbanView = memo(function KanbanView({ tasks, onTaskClick }: KanbanViewProps) {
  const tasksByStatus = useMemo(() => {
    const result: Record<PRDTask["status"], PRDTask[]> = {
      pending: [],
      in_progress: [],
      done: [],
      blocked: [],
      skipped: [],
    };
    tasks.forEach((task) => {
      result[task.status].push(task);
    });
    return result;
  }, [tasks]);

  return (
    <TooltipProvider>
      <div className="space-y-6 pb-4">
        {COLUMN_DEFINITIONS.map((column) => (
          <KanbanColumn
            key={column.status}
            status={column.status}
            label={column.label}
            icon={column.icon}
            color={column.color}
            bgColor={column.bgColor}
            tasks={tasksByStatus[column.status]}
            onTaskClick={onTaskClick}
          />
        ))}
      </div>
    </TooltipProvider>
  );
});
