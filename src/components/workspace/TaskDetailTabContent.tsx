import { AlertCircle, Ban, Calendar, CheckCircle2, Circle, ClipboardList, Play, Slash, Tag } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { useTabMeta } from "@/hooks/useTabMeta";
import type { WorkspaceTab } from "@/stores/useWorkspaceStore";
import type { PRDTask } from "@/types/prd";
import { TaskIdDisplay } from "../prd/TaskIdDisplay";

export function TaskDetailTabContent({ tab }: { tab: WorkspaceTab }) {
  const task = tab.data?.entity as PRDTask | undefined;
  useTabMeta(tab.id, task?.title ?? "Task Detail", ClipboardList);

  if (!task) {
    return (
      <div className="h-full flex items-center justify-center text-muted-foreground">
        <span>Task not found</span>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      <div className="px-4 py-3 flex-shrink-0">
        <h2 className="text-lg font-semibold">{task.title}</h2>
        <div className="mt-1">
          <TaskIdDisplay task={task} />
        </div>
      </div>
      <Separator />
      <ScrollArea className="flex-1 min-h-0">
        <div className="px-4 space-y-4">
          {/* Status and Priority */}
          <div className="flex gap-2 flex-wrap">
            {getStatusBadge(task.status)}
            {getPriorityBadge(task.priority)}
          </div>

          {/* Description */}
          {task.description && (
            <div>
              <h3 className="font-semibold mb-2">Description</h3>
              <p className="text-sm text-[hsl(var(--muted-foreground))] whitespace-pre-wrap">{task.description}</p>
            </div>
          )}

          <Separator />

          {/* Blocked By */}
          {task.blocked_by && (
            <div className="bg-[hsl(var(--destructive)/0.1)] border border-[hsl(var(--destructive))] rounded-lg p-3">
              <div className="flex items-start gap-2">
                <AlertCircle className="h-4 w-4 text-[hsl(var(--destructive))] mt-0.5" />
                <div>
                  <div className="font-semibold text-sm text-[hsl(var(--destructive))]">Blocked</div>
                  <p className="text-sm mt-1">{task.blocked_by}</p>
                </div>
              </div>
            </div>
          )}

          {/* Dependencies */}
          {task.depends_on && task.depends_on.length > 0 && (
            <div>
              <h3 className="font-semibold mb-2 text-sm">Dependencies</h3>
              <div className="flex gap-2 flex-wrap">
                {task.depends_on.map((depId) => (
                  <Badge key={depId} variant="outline" className="font-mono text-xs">
                    Task #{depId}
                  </Badge>
                ))}
              </div>
            </div>
          )}

          {/* Acceptance Criteria */}
          {task.acceptance_criteria && task.acceptance_criteria.length > 0 && (
            <div>
              <h3 className="font-semibold mb-2">Acceptance Criteria</h3>
              <ul className="space-y-2">
                {task.acceptance_criteria.map((criterion) => (
                  <li key={criterion} className="flex items-start gap-2">
                    <CheckCircle2 className="h-4 w-4 mt-0.5 text-[hsl(var(--muted-foreground))]" />
                    <span className="text-sm">{criterion}</span>
                  </li>
                ))}
              </ul>
            </div>
          )}

          {/* Tags */}
          {task.tags && task.tags.length > 0 && (
            <div>
              <h3 className="font-semibold mb-2 text-sm flex items-center gap-2">
                <Tag className="h-4 w-4" />
                Tags
              </h3>
              <div className="flex gap-2 flex-wrap">
                {task.tags.map((tag) => (
                  <Badge key={tag} variant="outline">
                    {tag}
                  </Badge>
                ))}
              </div>
            </div>
          )}

          <Separator />

          {/* Timestamps */}
          <div className="space-y-2">
            <h3 className="font-semibold text-sm flex items-center gap-2">
              <Calendar className="h-4 w-4" />
              Timeline
            </h3>
            <div className="text-xs text-[hsl(var(--muted-foreground))] space-y-1">
              {task.created && (
                <div className="flex justify-between">
                  <span>Created:</span>
                  <span className="font-medium">{task.created}</span>
                </div>
              )}
              {task.updated && (
                <div className="flex justify-between">
                  <span>Updated:</span>
                  <span className="font-medium">{task.updated}</span>
                </div>
              )}
              {task.completed && (
                <div className="flex justify-between">
                  <span>Completed:</span>
                  <span className="font-medium">{new Date(task.completed).toLocaleString()}</span>
                </div>
              )}
            </div>
          </div>
        </div>
      </ScrollArea>
    </div>
  );
}

function getStatusIcon(status: PRDTask["status"]) {
  const icons = {
    pending: <Circle className="h-4 w-4" />,
    in_progress: <Play className="h-4 w-4" />,
    done: <CheckCircle2 className="h-4 w-4" />,
    blocked: <Ban className="h-4 w-4" />,
    skipped: <Slash className="h-4 w-4" />,
  };
  return icons[status];
}

function getStatusBadge(status: PRDTask["status"]) {
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
    <Badge variant={variants[status]} className="gap-1">
      {getStatusIcon(status)}
      {labels[status]}
    </Badge>
  );
}

function getPriorityBadge(priority?: PRDTask["priority"]) {
  if (!priority) return null;
  const variants: Record<NonNullable<PRDTask["priority"]>, "default" | "warning" | "destructive"> = {
    low: "default",
    medium: "default",
    high: "warning",
    critical: "destructive",
  };
  return <Badge variant={variants[priority]}>{priority}</Badge>;
}
