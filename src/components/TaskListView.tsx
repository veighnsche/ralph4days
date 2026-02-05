import { memo, useMemo } from "react";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { Circle, Play, CheckCircle2, Ban, Slash, ChevronRight } from "lucide-react";

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

interface TaskListViewProps {
  tasks: PRDTask[];
  groupBy: "status" | "priority" | "none";
  onTaskClick: (task: PRDTask) => void;
}

export const TaskListView = memo(function TaskListView({ tasks, groupBy, onTaskClick }: TaskListViewProps) {
  const getStatusIcon = (status: PRDTask["status"]) => {
    const icons = {
      pending: <Circle className="h-4 w-4" />,
      in_progress: <Play className="h-4 w-4" />,
      done: <CheckCircle2 className="h-4 w-4" />,
      blocked: <Ban className="h-4 w-4" />,
      skipped: <Slash className="h-4 w-4" />,
    };
    return icons[status];
  };

  const getStatusBadge = (status: PRDTask["status"]) => {
    const variants: Record<PRDTask["status"], "default" | "success" | "warning" | "destructive"> = {
      pending: "default",
      in_progress: "warning",
      done: "success",
      blocked: "destructive",
      skipped: "default",
    };
    const labels: Record<PRDTask["status"], string> = {
      pending: "pending",
      in_progress: "in progress",
      done: "done",
      blocked: "blocked",
      skipped: "skipped",
    };
    return (
      <Badge variant={variants[status]} className="gap-1 text-xs">
        {getStatusIcon(status)}
        {labels[status]}
      </Badge>
    );
  };

  const getPriorityBadge = (priority?: PRDTask["priority"]) => {
    if (!priority) return null;
    const variants: Record<NonNullable<PRDTask["priority"]>, "default" | "warning" | "destructive"> = {
      low: "default",
      medium: "default",
      high: "warning",
      critical: "destructive",
    };
    return (
      <Badge variant={variants[priority]} className="text-xs">
        {priority}
      </Badge>
    );
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

  const groups = useMemo(() => {
    if (groupBy === "none") {
      return [{ label: "All Tasks", tasks }];
    }

    if (groupBy === "status") {
      const statusOrder: PRDTask["status"][] = ["blocked", "in_progress", "pending", "done", "skipped"];
      const statusLabels = {
        blocked: "Blocked",
        in_progress: "In Progress",
        pending: "Pending",
        done: "Done",
        skipped: "Skipped",
      };
      return statusOrder
        .map((status) => ({
          label: statusLabels[status],
          tasks: tasks.filter((t) => t.status === status),
        }))
        .filter((group) => group.tasks.length > 0);
    }

    if (groupBy === "priority") {
      const priorityOrder: Array<PRDTask["priority"] | undefined> = ["critical", "high", "medium", "low", undefined];
      const priorityLabels = {
        critical: "Critical Priority",
        high: "High Priority",
        medium: "Medium Priority",
        low: "Low Priority",
        undefined: "No Priority",
      };
      return priorityOrder
        .map((priority) => ({
          label: priorityLabels[priority as keyof typeof priorityLabels],
          tasks: tasks.filter((t) => t.priority === priority),
        }))
        .filter((group) => group.tasks.length > 0);
    }

    return [{ label: "All Tasks", tasks }];
  }, [tasks, groupBy]);

  return (
    <div className="space-y-6">
      {groups.map((group) => (
        <div key={group.label}>
          {groupBy !== "none" && (
            <h2 className="text-sm font-semibold mb-3 text-[hsl(var(--muted-foreground))]">
              {group.label} ({group.tasks.length})
            </h2>
          )}
          <div className="space-y-2">
            {group.tasks.map((task) => (
              <Card
                key={task.id}
                className="cursor-pointer hover:shadow-md transition-all border-l-4 p-4"
                style={{ borderLeftColor: getPriorityColor(task.priority) }}
                onClick={() => onTaskClick(task)}
              >
                <div className="flex items-start gap-3">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-start gap-2 mb-2">
                      <h3 className="font-medium text-sm flex-1">{task.title}</h3>
                      <ChevronRight className="h-4 w-4 text-[hsl(var(--muted-foreground))] flex-shrink-0 mt-0.5" />
                    </div>
                    {task.description && (
                      <p className="text-xs text-[hsl(var(--muted-foreground))] mb-2 line-clamp-2">
                        {task.description}
                      </p>
                    )}
                    {task.blocked_by && (
                      <div className="text-xs text-[hsl(var(--destructive))] mb-2 flex items-start gap-1">
                        <Ban className="h-3 w-3 mt-0.5 flex-shrink-0" />
                        <span className="line-clamp-1">{task.blocked_by}</span>
                      </div>
                    )}
                    <div className="flex items-center gap-2 flex-wrap">
                      {getStatusBadge(task.status)}
                      {getPriorityBadge(task.priority)}
                      {task.acceptance_criteria && task.acceptance_criteria.length > 0 && (
                        <Badge variant="outline" className="text-xs">
                          {task.acceptance_criteria.length} criteria
                        </Badge>
                      )}
                      {task.depends_on && task.depends_on.length > 0 && (
                        <Badge variant="outline" className="text-xs">
                          {task.depends_on.length} dependencies
                        </Badge>
                      )}
                      {task.tags && task.tags.length > 0 && (
                        <>
                          {task.tags.slice(0, 2).map((tag) => (
                            <span
                              key={tag}
                              className="text-xs bg-[hsl(var(--muted))] px-1.5 py-0.5 rounded"
                            >
                              {tag}
                            </span>
                          ))}
                          {task.tags.length > 2 && (
                            <span className="text-xs text-[hsl(var(--muted-foreground))]">
                              +{task.tags.length - 2}
                            </span>
                          )}
                        </>
                      )}
                      <span className="text-xs text-[hsl(var(--muted-foreground))] font-mono ml-auto">
                        {task.id}
                      </span>
                    </div>
                  </div>
                </div>
              </Card>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
});
