import { useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { AlertCircle, Bot, CheckCircle2, Cog, FileCode, MessageSquare, Send, User } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { INFERRED_STATUS_CONFIG, PRIORITY_CONFIG, STATUS_CONFIG } from "@/constants/prd";
import { useTabMeta } from "@/hooks/useTabMeta";
import { resolveIcon } from "@/lib/iconRegistry";
import { shouldShowInferredStatus } from "@/lib/taskStatus";
import type { WorkspaceTab } from "@/stores/useWorkspaceStore";
import type { EnrichedTask, TaskComment } from "@/types/prd";
import { TaskIdDisplay } from "../prd/TaskIdDisplay";

const PROVENANCE_CONFIG = {
  agent: { label: "Agent", icon: Bot },
  human: { label: "Human", icon: User },
  system: { label: "System", icon: Cog },
} as const;

export function TaskDetailTabContent({ tab }: { tab: WorkspaceTab }) {
  const task = tab.data?.entity as EnrichedTask | undefined;
  useTabMeta(tab.id, task?.title ?? "Task Detail", CheckCircle2);
  const queryClient = useQueryClient();
  const [comments, setComments] = useState<TaskComment[]>(task?.comments ?? []);
  const [commentInput, setCommentInput] = useState("");
  const [submitting, setSubmitting] = useState(false);

  // Sync local comments state from server data (also handles task switching)
  useEffect(() => {
    setComments(task?.comments ?? []);
  }, [task?.comments]);

  const handleAddComment = async () => {
    if (!task || !commentInput.trim()) return;
    setSubmitting(true);
    try {
      await invoke("add_task_comment", {
        taskId: task.id,
        author: "human",
        body: commentInput.trim(),
      });
      setComments((prev) => [
        ...prev,
        { author: "human" as const, body: commentInput.trim(), created: new Date().toISOString() },
      ]);
      setCommentInput("");
      queryClient.invalidateQueries({ queryKey: ["get_enriched_tasks"] });
      toast.success("Comment added");
    } catch (e) {
      toast.error(String(e));
    } finally {
      setSubmitting(false);
    }
  };

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
      <Card className="shadow-lg flex flex-row gap-0 py-0 my-3">
        {/* ── Main Content ── */}
        <ScrollArea className="flex-1 min-w-0">
          <div className="py-4 space-y-6">
            {(() => {
              const sections: React.ReactNode[] = [];
              sections.push(
                <div key="body" className="px-6 space-y-3">
                  <div className="space-y-1.5">
                    <TaskIdDisplay task={task} variant="full" />
                    <h1 className="text-xl font-semibold leading-tight">{task.title}</h1>
                  </div>

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

                  {task.description && (
                    <>
                      <h2 className="text-sm font-medium text-muted-foreground">Description</h2>
                      <p className="text-sm leading-relaxed whitespace-pre-wrap">{task.description}</p>
                    </>
                  )}
                  {task.hints && (
                    <div className="border-l-2 border-muted-foreground/20 pl-3">
                      <p className="text-sm text-muted-foreground leading-relaxed whitespace-pre-wrap">{task.hints}</p>
                    </div>
                  )}
                </div>
              );
              if (task.acceptanceCriteria && task.acceptanceCriteria.length > 0) {
                sections.push(
                  <div key="criteria" className="px-6 space-y-2">
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
                );
              }
              if (
                (task.contextFiles && task.contextFiles.length > 0) ||
                (task.outputArtifacts && task.outputArtifacts.length > 0)
              ) {
                sections.push(
                  <div key="files" className="px-6 space-y-2">
                    <h2 className="text-sm font-medium text-muted-foreground">Files</h2>
                    <div className="space-y-1.5">
                      {task.contextFiles && task.contextFiles.length > 0 && (
                        <div className="flex flex-wrap items-center gap-1.5">
                          <span className="text-xs text-muted-foreground">In:</span>
                          {task.contextFiles.map((file) => (
                            <Badge key={file} variant="outline" className="text-xs font-mono px-2 py-0.5 h-5 gap-1">
                              <FileCode className="h-3 w-3 text-muted-foreground" />
                              {file}
                            </Badge>
                          ))}
                        </div>
                      )}
                      {task.outputArtifacts && task.outputArtifacts.length > 0 && (
                        <div className="flex flex-wrap items-center gap-1.5">
                          <span className="text-xs text-muted-foreground">Out:</span>
                          {task.outputArtifacts.map((artifact) => (
                            <Badge key={artifact} variant="outline" className="text-xs font-mono px-2 py-0.5 h-5 gap-1">
                              <FileCode className="h-3 w-3 text-muted-foreground" />
                              {artifact}
                            </Badge>
                          ))}
                        </div>
                      )}
                    </div>
                  </div>
                );
              }
              sections.push(
                <div key="comments" className="px-6 space-y-3">
                  <h2 className="text-sm font-medium text-muted-foreground flex items-center gap-1.5">
                    <MessageSquare className="h-3.5 w-3.5" />
                    Comments{comments.length > 0 && ` (${comments.length})`}
                  </h2>
                  {comments.length > 0 && (
                    <div className="space-y-2">
                      {comments.map((comment, i) => (
                        <div key={i} className="flex items-start gap-2 text-sm">
                          <div className="flex items-center gap-1 flex-shrink-0 mt-0.5">
                            {comment.author === "agent" ? (
                              <Bot className="h-3.5 w-3.5 text-muted-foreground" />
                            ) : (
                              <User className="h-3.5 w-3.5 text-muted-foreground" />
                            )}
                            <Badge variant="outline" className="text-xs px-1.5 py-0 h-4">
                              {comment.author === "agent" ? `Agent #${comment.agent_task_id}` : "You"}
                            </Badge>
                          </div>
                          <div className="flex-1 min-w-0">
                            <p className="text-sm leading-relaxed whitespace-pre-wrap">{comment.body}</p>
                            {comment.created && (
                              <span className="text-xs text-muted-foreground">{formatDate(comment.created)}</span>
                            )}
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                  <div className="flex items-center gap-2">
                    <Input
                      placeholder="Add a comment..."
                      value={commentInput}
                      onChange={(e) => setCommentInput(e.target.value)}
                      onKeyDown={(e) => {
                        if (e.key === "Enter" && !submitting) handleAddComment();
                      }}
                      className="h-8 text-sm"
                    />
                    <Button
                      size="sm"
                      variant="ghost"
                      className="h-8 w-8 p-0 flex-shrink-0"
                      disabled={!commentInput.trim() || submitting}
                      onClick={handleAddComment}
                    >
                      <Send className="h-3.5 w-3.5" />
                    </Button>
                  </div>
                </div>
              );
              return sections.flatMap((section, i) =>
                i === 0 ? [section] : [<Separator key={`sep-${i}`} />, section]
              );
            })()}
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
                  {task.estimatedTurns != null && (
                    <span className="text-xs text-muted-foreground ml-1">· ~{task.estimatedTurns} turns</span>
                  )}
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

            <Separator bleed="md" className="my-2" />

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
                <Separator bleed="md" className="my-2" />
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
            <Separator bleed="md" className="my-2" />
            {task.created && (
              <PropertyRow label="Created">
                <div className="flex items-center gap-1.5">
                  <span className="text-xs text-muted-foreground">{formatDate(task.created)}</span>
                  {task.provenance &&
                    (() => {
                      const prov = PROVENANCE_CONFIG[task.provenance];
                      const ProvIcon = prov.icon;
                      return (
                        <>
                          <span className="text-xs text-muted-foreground">·</span>
                          <ProvIcon className="h-3 w-3 text-muted-foreground" />
                          <span className="text-xs text-muted-foreground">{prov.label}</span>
                        </>
                      );
                    })()}
                </div>
              </PropertyRow>
            )}
            {!task.created && task.provenance && (
              <PropertyRow label="Created by">
                {(() => {
                  const prov = PROVENANCE_CONFIG[task.provenance];
                  const ProvIcon = prov.icon;
                  return (
                    <div className="flex items-center gap-1.5">
                      <ProvIcon className="h-3 w-3 text-muted-foreground" />
                      <span className="text-xs text-muted-foreground">{prov.label}</span>
                    </div>
                  );
                })()}
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
