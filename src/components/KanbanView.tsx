import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Circle, Play, CheckCircle2, Ban, Slash } from "lucide-react";

interface PRDTask {
  id: string;
  title: string;
  description?: string;
  status: "pending" | "in_progress" | "done" | "blocked" | "skipped";
  priority?: "low" | "medium" | "high" | "critical";
  tags?: string[];
  depends_on?: string[];
  blocked_by?: string;
  created?: string;
  updated?: string;
  completed?: string;
  acceptance_criteria?: string[];
}

interface KanbanViewProps {
  tasks: PRDTask[];
  onTaskClick: (task: PRDTask) => void;
}

export function KanbanView({ tasks, onTaskClick }: KanbanViewProps) {
  const columns: Array<{
    status: PRDTask["status"];
    label: string;
    icon: React.ReactNode;
    color: string;
  }> = [
    {
      status: "pending",
      label: "Pending",
      icon: <Circle className="h-4 w-4" />,
      color: "hsl(var(--muted-foreground))",
    },
    {
      status: "in_progress",
      label: "In Progress",
      icon: <Play className="h-4 w-4" />,
      color: "hsl(var(--warning))",
    },
    {
      status: "blocked",
      label: "Blocked",
      icon: <Ban className="h-4 w-4" />,
      color: "hsl(var(--destructive))",
    },
    {
      status: "done",
      label: "Done",
      icon: <CheckCircle2 className="h-4 w-4" />,
      color: "hsl(var(--success))",
    },
    {
      status: "skipped",
      label: "Skipped",
      icon: <Slash className="h-4 w-4" />,
      color: "hsl(var(--muted-foreground))",
    },
  ];

  const getTasksByStatus = (status: PRDTask["status"]) => {
    return tasks.filter((task) => task.status === status);
  };

  const getPriorityColor = (priority?: PRDTask["priority"]) => {
    if (!priority) return "transparent";
    const colors = {
      low: "hsl(var(--muted))",
      medium: "hsl(var(--primary))",
      high: "hsl(var(--warning))",
      critical: "hsl(var(--destructive))",
    };
    return colors[priority];
  };

  return (
    <div className="flex gap-4 overflow-x-auto pb-4">
      {columns.map((column) => {
        const columnTasks = getTasksByStatus(column.status);
        return (
          <div key={column.status} className="flex-shrink-0 w-[300px]">
            <div
              className="font-semibold mb-3 flex items-center gap-2 text-sm sticky top-0 bg-[hsl(var(--background))] py-2 z-10"
              style={{ color: column.color }}
            >
              {column.icon}
              {column.label}
              <span className="ml-auto bg-[hsl(var(--muted))] px-2 py-0.5 rounded-full text-xs">
                {columnTasks.length}
              </span>
            </div>
            <div className="space-y-2">
              {columnTasks.map((task) => (
                <Card
                  key={task.id}
                  className="cursor-pointer hover:shadow-md transition-shadow border-l-4"
                  style={{ borderLeftColor: getPriorityColor(task.priority) }}
                  onClick={() => onTaskClick(task)}
                >
                  <CardHeader className="p-3 pb-2">
                    <div className="font-medium text-sm line-clamp-2">{task.title}</div>
                  </CardHeader>
                  <CardContent className="p-3 pt-0 space-y-2">
                    {task.description && (
                      <p className="text-xs text-[hsl(var(--muted-foreground))] line-clamp-2">
                        {task.description}
                      </p>
                    )}
                    {task.blocked_by && (
                      <div className="text-xs text-[hsl(var(--destructive))] flex items-start gap-1">
                        <Ban className="h-3 w-3 mt-0.5 flex-shrink-0" />
                        <span className="line-clamp-1">{task.blocked_by}</span>
                      </div>
                    )}
                    <div className="flex items-center gap-1 flex-wrap">
                      {task.priority && (
                        <Badge variant="outline" className="text-xs px-1.5 py-0">
                          {task.priority}
                        </Badge>
                      )}
                      {task.acceptance_criteria && task.acceptance_criteria.length > 0 && (
                        <Badge variant="outline" className="text-xs px-1.5 py-0">
                          {task.acceptance_criteria.length} AC
                        </Badge>
                      )}
                      {task.depends_on && task.depends_on.length > 0 && (
                        <Badge variant="outline" className="text-xs px-1.5 py-0">
                          {task.depends_on.length} deps
                        </Badge>
                      )}
                    </div>
                    {task.tags && task.tags.length > 0 && (
                      <div className="flex gap-1 flex-wrap">
                        {task.tags.slice(0, 3).map((tag) => (
                          <span
                            key={tag}
                            className="text-xs bg-[hsl(var(--muted))] px-1.5 py-0.5 rounded"
                          >
                            {tag}
                          </span>
                        ))}
                        {task.tags.length > 3 && (
                          <span className="text-xs text-[hsl(var(--muted-foreground))]">
                            +{task.tags.length - 3}
                          </span>
                        )}
                      </div>
                    )}
                    <div className="text-xs text-[hsl(var(--muted-foreground))] font-mono">
                      {task.id}
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          </div>
        );
      })}
    </div>
  );
}
