import { AlertCircle, CheckCircle2 } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { FullBleedSeparator } from "@/components/ui/full-bleed-separator";
import { ScrollArea } from "@/components/ui/scroll-area";
import { INFERRED_STATUS_CONFIG, PRIORITY_CONFIG, STATUS_CONFIG } from "@/constants/prd";
import { useTabMeta } from "@/hooks/useTabMeta";
import { resolveIcon } from "@/lib/iconRegistry";
import { shouldShowInferredStatus } from "@/lib/taskStatus";
import type { WorkspaceTab } from "@/stores/useWorkspaceStore";
import type { EnrichedTask } from "@/types/prd";
import { TaskIdDisplay } from "../prd/TaskIdDisplay";

export function TaskDetailTabContent({ tab }: { tab: WorkspaceTab }) {
  const task = tab.data?.entity as EnrichedTask | undefined;
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
  const DisciplineIcon = resolveIcon(task.disciplineIcon);

  return (
    <div
      className="h-full px-3 overflow-hidden relative"
      style={{
        background: `repeating-linear-gradient(
        45deg,
        transparent,
        transparent 10px,
        ${statusConfig.color}15 10px,
        ${statusConfig.color}15 20px
      )`,
      }}
    >
      {/* Card Wrapper */}
      <Card className="shadow-lg flex flex-row py-0 my-3">
        {/* ── Main Content ── */}
        <ScrollArea className="flex-1 min-w-0">
          <div className="px-6 py-4 space-y-5">
            {/* Breadcrumb + Title */}
            <div className="space-y-2">
              <TaskIdDisplay task={task} variant="full" />
              <h1 className="text-xl font-semibold leading-tight">{task.title}</h1>
            </div>

            {/* Blocked Alert */}
            {task.blockedBy && (
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
                  {task.blockedBy}
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
            {task.acceptanceCriteria && task.acceptanceCriteria.length > 0 && (
              <div className="space-y-2">
                <h2 className="text-sm font-medium text-muted-foreground">Acceptance Criteria</h2>
                <ul className="space-y-1.5">
                  {task.acceptanceCriteria.map((criterion) => (
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
        <div className="w-56 flex-shrink-0 border-l">
          <div className="px-4 py-4 space-y-0.5 overflow-y-auto h-full">
            {/* Status - consolidated with inferred status and dependencies */}
            <PropertyRow label="Status">
              <div className="flex flex-col gap-1.5">
                {/* Actual Status */}
                <div className="flex items-center gap-1.5">
                  <StatusIcon className="h-3.5 w-3.5" style={{ color: statusConfig.color }} />
                  <span className="text-sm" style={{ color: statusConfig.color }}>
                    {statusConfig.label}
                  </span>
                </div>

                {/* Inferred Status (only if different from actual) */}
                {shouldShowInferredStatus(task.status, task.inferredStatus) &&
                  (() => {
                    const inferredConfig = INFERRED_STATUS_CONFIG[task.inferredStatus];
                    const InferredIcon = inferredConfig.icon;
                    const hasDeps = task.dependsOn && task.dependsOn.length > 0;

                    return (
                      <div className="flex items-start gap-1.5 pl-5">
                        <span className="text-xs text-muted-foreground mt-0.5">→</span>
                        <div className="flex flex-col gap-1">
                          <div className="flex items-center gap-1.5">
                            <InferredIcon className="h-3 w-3" style={{ color: inferredConfig.color }} />
                            <span className="text-xs font-medium" style={{ color: inferredConfig.color }}>
                              {inferredConfig.label}
                            </span>
                          </div>
                          {hasDeps && (
                            <div className="flex flex-wrap gap-1 items-center">
                              <span className="text-xs text-muted-foreground">Depends on:</span>
                              {task.dependsOn?.map((depId) => (
                                <Badge key={depId} variant="outline" className="text-xs font-mono px-1.5 py-0 h-4">
                                  #{depId.toString().padStart(3, "0")}
                                </Badge>
                              ))}
                            </div>
                          )}
                        </div>
                      </div>
                    );
                  })()}

                {/* Show dependencies even when not waiting (if task has no inferred status difference) */}
                {!shouldShowInferredStatus(task.status, task.inferredStatus) &&
                  task.dependsOn &&
                  task.dependsOn.length > 0 && (
                    <div className="flex flex-wrap gap-1 items-center pl-5">
                      <span className="text-xs text-muted-foreground">Depends on:</span>
                      {task.dependsOn.map((depId) => (
                        <Badge key={depId} variant="outline" className="text-xs font-mono px-1.5 py-0 h-4">
                          #{depId.toString().padStart(3, "0")}
                        </Badge>
                      ))}
                    </div>
                  )}
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

            <FullBleedSeparator className="my-2" />

            {/* Feature */}
            <PropertyRow label="Feature">
              <span className="text-sm">{task.featureDisplayName}</span>
            </PropertyRow>

            {/* Discipline */}
            <PropertyRow label="Discipline">
              <div className="flex items-center gap-1.5">
                <DisciplineIcon className="h-3.5 w-3.5" style={{ color: task.disciplineColor }} />
                <span className="text-sm" style={{ color: task.disciplineColor }}>
                  {task.disciplineDisplayName}
                </span>
              </div>
            </PropertyRow>

            {/* Tags */}
            {task.tags && task.tags.length > 0 && (
              <>
                <FullBleedSeparator className="my-2" />
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

            {/* Timeline */}
            <FullBleedSeparator className="my-2" />
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
      </Card>
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
