import { AlertCircle, CheckCircle2, Circle } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { PRIORITY_CONFIG, STATUS_CONFIG } from "@/constants/prd";
import { useDisciplines } from "@/hooks/useDisciplines";
import { useFeatures } from "@/hooks/useFeatures";
import { useTabMeta } from "@/hooks/useTabMeta";
import type { WorkspaceTab } from "@/stores/useWorkspaceStore";
import type { PRDTask } from "@/types/prd";
import { TaskIdDisplay } from "../prd/TaskIdDisplay";

export function TaskDetailTabContent({ tab }: { tab: WorkspaceTab }) {
  const task = tab.data?.entity as PRDTask | undefined;
  const { configMap: disciplineMap } = useDisciplines();
  const { configMap: featureMap } = useFeatures();
  useTabMeta(tab.id, task?.title ?? "Task Detail", CheckCircle2);

  if (!task) {
    return (
      <div className="h-full flex items-center justify-center text-muted-foreground">
        <span>Task not found</span>
      </div>
    );
  }

  const statusConfig = STATUS_CONFIG[task.status];
  const StatusIcon = statusConfig.icon;
  const priorityConfig = task.priority ? PRIORITY_CONFIG[task.priority] : null;
  const disciplineConfig = disciplineMap[task.discipline];
  const featureConfig = featureMap.get(task.feature);
  const DisciplineIcon = disciplineConfig?.icon || Circle;

  return (
    <div className="h-full flex">
      {/* ── Main Content ── */}
      <ScrollArea className="flex-1 min-w-0">
        <div className="px-6 py-5 space-y-5">
          {/* Breadcrumb + Title */}
          <div className="space-y-2">
            <TaskIdDisplay task={task} variant="full" />
            <h1 className="text-xl font-semibold leading-tight">{task.title}</h1>
          </div>

          {/* Blocked Alert */}
          {task.blocked_by && (
            <div
              className="flex items-start gap-3 rounded-md px-3 py-2.5 text-sm"
              style={{
                backgroundColor: STATUS_CONFIG.blocked.bgColor,
                color: STATUS_CONFIG.blocked.color,
              }}
            >
              <AlertCircle className="h-4 w-4 mt-0.5 flex-shrink-0" />
              <div>
                <span className="font-medium">Blocked — </span>
                {task.blocked_by}
              </div>
            </div>
          )}

          {/* Description */}
          {task.description && (
            <div className="space-y-2">
              <h2 className="text-sm font-medium text-muted-foreground">Description</h2>
              <p className="text-sm leading-relaxed whitespace-pre-wrap">{task.description}</p>
            </div>
          )}

          {/* Acceptance Criteria */}
          {task.acceptance_criteria && task.acceptance_criteria.length > 0 && (
            <div className="space-y-2">
              <h2 className="text-sm font-medium text-muted-foreground">Acceptance Criteria</h2>
              <ul className="space-y-1.5">
                {task.acceptance_criteria.map((criterion) => (
                  <li key={criterion} className="flex items-start gap-2.5 text-sm">
                    <div
                      className="mt-1 w-4 h-4 rounded-sm border flex items-center justify-center flex-shrink-0"
                      style={{
                        borderColor: task.status === "done" ? STATUS_CONFIG.done.color : "hsl(var(--border))",
                        backgroundColor: task.status === "done" ? STATUS_CONFIG.done.bgColor : "transparent",
                      }}
                    >
                      {task.status === "done" && (
                        <CheckCircle2 className="w-3 h-3" style={{ color: STATUS_CONFIG.done.color }} />
                      )}
                    </div>
                    <span className={task.status === "done" ? "line-through text-muted-foreground" : ""}>
                      {criterion}
                    </span>
                  </li>
                ))}
              </ul>
            </div>
          )}
        </div>
      </ScrollArea>

      {/* ── Properties Sidebar ── */}
      <div className="w-56 flex-shrink-0 border-l overflow-y-auto">
        <div className="px-4 py-5 space-y-0.5">
          {/* Status */}
          <PropertyRow label="Status">
            <div className="flex items-center gap-1.5">
              <StatusIcon className="h-3.5 w-3.5" style={{ color: statusConfig.color }} />
              <span className="text-sm" style={{ color: statusConfig.color }}>
                {statusConfig.label}
              </span>
            </div>
          </PropertyRow>

          {/* Priority */}
          <PropertyRow label="Priority">
            {priorityConfig ? (
              <span className="text-sm" style={{ color: priorityConfig.color }}>
                {priorityConfig.label}
              </span>
            ) : (
              <span className="text-sm text-muted-foreground">None</span>
            )}
          </PropertyRow>

          <Separator className="my-2" />

          {/* Feature */}
          <PropertyRow label="Feature">
            <span className="text-sm">{featureConfig?.displayName || task.feature}</span>
          </PropertyRow>

          {/* Discipline */}
          <PropertyRow label="Discipline">
            <div className="flex items-center gap-1.5">
              <DisciplineIcon className="h-3.5 w-3.5" style={{ color: disciplineConfig?.color }} />
              <span className="text-sm" style={{ color: disciplineConfig?.color }}>
                {disciplineConfig?.displayName || task.discipline}
              </span>
            </div>
          </PropertyRow>

          {/* Tags */}
          {task.tags && task.tags.length > 0 && (
            <>
              <Separator className="my-2" />
              <PropertyRow label="Tags">
                <div className="flex flex-wrap gap-1">
                  {task.tags.map((tag) => (
                    <Badge key={tag} variant="secondary" className="text-xs px-1.5 py-0 h-5">
                      {tag}
                    </Badge>
                  ))}
                </div>
              </PropertyRow>
            </>
          )}

          {/* Dependencies */}
          {task.depends_on && task.depends_on.length > 0 && (
            <>
              <Separator className="my-2" />
              <PropertyRow label="Depends on">
                <div className="flex flex-wrap gap-1">
                  {task.depends_on.map((depId) => (
                    <Badge key={depId} variant="outline" className="text-xs font-mono px-1.5 py-0 h-5">
                      #{depId.toString().padStart(3, "0")}
                    </Badge>
                  ))}
                </div>
              </PropertyRow>
            </>
          )}

          {/* Timeline */}
          <Separator className="my-2" />
          {task.created && (
            <PropertyRow label="Created">
              <span className="text-xs text-muted-foreground">{formatDate(task.created)}</span>
            </PropertyRow>
          )}
          {task.updated && (
            <PropertyRow label="Updated">
              <span className="text-xs text-muted-foreground">{formatDate(task.updated)}</span>
            </PropertyRow>
          )}
          {task.completed && (
            <PropertyRow label="Completed">
              <span className="text-xs" style={{ color: STATUS_CONFIG.done.color }}>
                {formatDate(task.completed)}
              </span>
            </PropertyRow>
          )}
        </div>
      </div>
    </div>
  );
}

function PropertyRow({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex flex-col gap-1 py-1.5">
      <span className="text-xs font-medium text-muted-foreground">{label}</span>
      {children}
    </div>
  );
}

function formatDate(value: unknown): string {
  if (typeof value === "string") {
    // Try to parse and format nicely, fall back to raw string
    const d = new Date(value);
    if (!Number.isNaN(d.getTime())) {
      return d.toLocaleDateString(undefined, { month: "short", day: "numeric", year: "numeric" });
    }
    return value;
  }
  if (value instanceof Date) {
    return value.toLocaleDateString(undefined, { month: "short", day: "numeric", year: "numeric" });
  }
  return String(value);
}
