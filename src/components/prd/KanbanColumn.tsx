import { memo } from "react";
import { KanbanCard } from "./KanbanCard";
import type { PRDTask } from "@/types/prd";
import type { LucideIcon } from "lucide-react";

interface KanbanColumnProps {
  status: PRDTask["status"];
  label: string;
  icon: LucideIcon;
  color: string;
  bgColor: string;
  tasks: PRDTask[];
  onTaskClick: (task: PRDTask) => void;
}

export const KanbanColumn = memo(function KanbanColumn({
  label,
  icon: Icon,
  color,
  bgColor,
  tasks,
  onTaskClick,
}: KanbanColumnProps) {
  return (
    <div className="w-full">
      <div
        className="font-semibold mb-3 flex items-center gap-2 text-sm py-2 px-3 rounded-lg"
        style={{ backgroundColor: bgColor }}
      >
        <span style={{ color }}>
          <Icon className="h-4 w-4" />
        </span>
        <span style={{ color }}>{label}</span>
        <span
          className="ml-auto px-2 py-0.5 rounded-full text-xs font-medium"
          style={{
            backgroundColor: color,
            color: "hsl(var(--card))",
          }}
        >
          {tasks.length}
        </span>
      </div>
      <div className="grid grid-cols-[repeat(auto-fill,minmax(340px,1fr))] gap-3">
        {tasks.map((task) => (
          <KanbanCard key={task.id} task={task} columnColor={color} onClick={() => onTaskClick(task)} />
        ))}
      </div>
    </div>
  );
});
