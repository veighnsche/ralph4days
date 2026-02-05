import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Skeleton } from "@/components/ui/skeleton";
import yaml from "js-yaml";

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

interface PRDProject {
  title: string;
  description?: string;
  created?: string;
}

interface PRDData {
  schema_version: string;
  project: PRDProject;
  tasks: PRDTask[];
}

export function PRDViewer() {
  const [prdData, setPrdData] = useState<PRDData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (typeof window !== 'undefined' && '__TAURI__' in window) {
      invoke<string>("get_prd_content")
        .then((content) => {
          try {
            const parsed = yaml.load(content) as PRDData;
            setPrdData(parsed);
            setError(null);
          } catch (e) {
            setError(`Failed to parse YAML: ${e}`);
          }
          setLoading(false);
        })
        .catch((err) => {
          setError(err);
          setLoading(false);
        });
    }
  }, []);

  if (loading) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-[100px]" />
        <Skeleton className="h-[60px]" />
        <Skeleton className="h-[60px]" />
        <Skeleton className="h-[60px]" />
      </div>
    );
  }

  if (error) {
    return (
      <Alert variant="destructive">
        <AlertDescription>{error}</AlertDescription>
      </Alert>
    );
  }

  if (!prdData) {
    return (
      <Alert>
        <AlertDescription>No PRD data available</AlertDescription>
      </Alert>
    );
  }

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
    return <Badge variant={variants[status]}>{labels[status]}</Badge>;
  };

  const getPriorityBadge = (priority?: PRDTask["priority"]) => {
    if (!priority) return null;
    const variants: Record<NonNullable<PRDTask["priority"]>, "default" | "warning" | "destructive"> = {
      low: "default",
      medium: "default",
      high: "warning",
      critical: "destructive",
    };
    return <Badge variant={variants[priority]}>{priority}</Badge>;
  };

  const pendingTasks = prdData.tasks.filter((t) => t.status === "pending");
  const inProgressTasks = prdData.tasks.filter((t) => t.status === "in_progress");
  const doneTasks = prdData.tasks.filter((t) => t.status === "done");
  const blockedTasks = prdData.tasks.filter((t) => t.status === "blocked");
  const skippedTasks = prdData.tasks.filter((t) => t.status === "skipped");

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>{prdData.project.title}</CardTitle>
          {prdData.project.description && (
            <CardDescription>{prdData.project.description}</CardDescription>
          )}
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap gap-4 text-sm text-[hsl(var(--muted-foreground))]">
            {prdData.project.created && <div>Created: {prdData.project.created}</div>}
            <div>Total: {prdData.tasks.length}</div>
            <div>Done: {doneTasks.length}</div>
            <div>In Progress: {inProgressTasks.length}</div>
            <div>Pending: {pendingTasks.length}</div>
            {blockedTasks.length > 0 && <div>Blocked: {blockedTasks.length}</div>}
            {skippedTasks.length > 0 && <div>Skipped: {skippedTasks.length}</div>}
          </div>
        </CardContent>
      </Card>

      {blockedTasks.length > 0 && (
        <div>
          <h2 className="text-lg font-semibold mb-3">Blocked Tasks ({blockedTasks.length})</h2>
          <Accordion type="single" collapsible className="space-y-2">
            {blockedTasks.map((task) => (
              <AccordionItem key={task.id} value={task.id}>
                <AccordionTrigger>
                  <div className="flex items-center gap-2 flex-1">
                    <span>{task.title}</span>
                    <div className="flex gap-2">
                      {getStatusBadge(task.status)}
                      {getPriorityBadge(task.priority)}
                    </div>
                  </div>
                </AccordionTrigger>
                <AccordionContent>
                  <div className="space-y-3 pt-2">
                    {task.description && (
                      <div>
                        <div className="font-medium text-sm mb-1">Description</div>
                        <div className="text-sm text-[hsl(var(--muted-foreground))]">{task.description}</div>
                      </div>
                    )}
                    {task.blocked_by && (
                      <div>
                        <div className="font-medium text-sm mb-1">Blocked By</div>
                        <div className="text-sm text-[hsl(var(--muted-foreground))]">{task.blocked_by}</div>
                      </div>
                    )}
                    {task.depends_on && task.depends_on.length > 0 && (
                      <div>
                        <div className="font-medium text-sm mb-1">Dependencies</div>
                        <div className="flex gap-2 flex-wrap">
                          {task.depends_on.map((depId) => (
                            <Badge key={depId} variant="outline">{depId}</Badge>
                          ))}
                        </div>
                      </div>
                    )}
                    {task.acceptance_criteria && task.acceptance_criteria.length > 0 && (
                      <div>
                        <div className="font-medium text-sm mb-1">Acceptance Criteria</div>
                        <ul className="list-disc list-inside space-y-1 text-sm text-[hsl(var(--muted-foreground))]">
                          {task.acceptance_criteria.map((criterion, idx) => (
                            <li key={idx}>{criterion}</li>
                          ))}
                        </ul>
                      </div>
                    )}
                    {task.tags && task.tags.length > 0 && (
                      <div className="flex gap-2 flex-wrap">
                        {task.tags.map((tag) => (
                          <Badge key={tag} variant="outline">{tag}</Badge>
                        ))}
                      </div>
                    )}
                    <div className="text-xs text-[hsl(var(--muted-foreground))] flex gap-3">
                      {task.created && <span>Created: {task.created}</span>}
                      {task.updated && <span>Updated: {task.updated}</span>}
                      {task.completed && <span>Completed: {task.completed}</span>}
                    </div>
                  </div>
                </AccordionContent>
              </AccordionItem>
            ))}
          </Accordion>
        </div>
      )}

      {inProgressTasks.length > 0 && (
        <div>
          <h2 className="text-lg font-semibold mb-3">In Progress ({inProgressTasks.length})</h2>
          <Accordion type="single" collapsible className="space-y-2">
            {inProgressTasks.map((task) => (
              <AccordionItem key={task.id} value={task.id}>
                <AccordionTrigger>
                  <div className="flex items-center gap-2 flex-1">
                    <span>{task.title}</span>
                    <div className="flex gap-2">
                      {getStatusBadge(task.status)}
                      {getPriorityBadge(task.priority)}
                    </div>
                  </div>
                </AccordionTrigger>
                <AccordionContent>
                  <div className="space-y-3 pt-2">
                    {task.description && (
                      <div>
                        <div className="font-medium text-sm mb-1">Description</div>
                        <div className="text-sm text-[hsl(var(--muted-foreground))]">{task.description}</div>
                      </div>
                    )}
                    {task.blocked_by && (
                      <div>
                        <div className="font-medium text-sm mb-1">Blocked By</div>
                        <div className="text-sm text-[hsl(var(--muted-foreground))]">{task.blocked_by}</div>
                      </div>
                    )}
                    {task.depends_on && task.depends_on.length > 0 && (
                      <div>
                        <div className="font-medium text-sm mb-1">Dependencies</div>
                        <div className="flex gap-2 flex-wrap">
                          {task.depends_on.map((depId) => (
                            <Badge key={depId} variant="outline">{depId}</Badge>
                          ))}
                        </div>
                      </div>
                    )}
                    {task.acceptance_criteria && task.acceptance_criteria.length > 0 && (
                      <div>
                        <div className="font-medium text-sm mb-1">Acceptance Criteria</div>
                        <ul className="list-disc list-inside space-y-1 text-sm text-[hsl(var(--muted-foreground))]">
                          {task.acceptance_criteria.map((criterion, idx) => (
                            <li key={idx}>{criterion}</li>
                          ))}
                        </ul>
                      </div>
                    )}
                    {task.tags && task.tags.length > 0 && (
                      <div className="flex gap-2 flex-wrap">
                        {task.tags.map((tag) => (
                          <Badge key={tag} variant="outline">{tag}</Badge>
                        ))}
                      </div>
                    )}
                    <div className="text-xs text-[hsl(var(--muted-foreground))] flex gap-3">
                      {task.created && <span>Created: {task.created}</span>}
                      {task.updated && <span>Updated: {task.updated}</span>}
                      {task.completed && <span>Completed: {task.completed}</span>}
                    </div>
                  </div>
                </AccordionContent>
              </AccordionItem>
            ))}
          </Accordion>
        </div>
      )}

      {pendingTasks.length > 0 && (
        <div>
          <h2 className="text-lg font-semibold mb-3">Pending ({pendingTasks.length})</h2>
          <Accordion type="single" collapsible className="space-y-2">
            {pendingTasks.map((task) => (
              <AccordionItem key={task.id} value={task.id}>
                <AccordionTrigger>
                  <div className="flex items-center gap-2 flex-1">
                    <span>{task.title}</span>
                    <div className="flex gap-2">
                      {getStatusBadge(task.status)}
                      {getPriorityBadge(task.priority)}
                    </div>
                  </div>
                </AccordionTrigger>
                <AccordionContent>
                  <div className="space-y-3 pt-2">
                    {task.description && (
                      <div>
                        <div className="font-medium text-sm mb-1">Description</div>
                        <div className="text-sm text-[hsl(var(--muted-foreground))]">{task.description}</div>
                      </div>
                    )}
                    {task.blocked_by && (
                      <div>
                        <div className="font-medium text-sm mb-1">Blocked By</div>
                        <div className="text-sm text-[hsl(var(--muted-foreground))]">{task.blocked_by}</div>
                      </div>
                    )}
                    {task.depends_on && task.depends_on.length > 0 && (
                      <div>
                        <div className="font-medium text-sm mb-1">Dependencies</div>
                        <div className="flex gap-2 flex-wrap">
                          {task.depends_on.map((depId) => (
                            <Badge key={depId} variant="outline">{depId}</Badge>
                          ))}
                        </div>
                      </div>
                    )}
                    {task.acceptance_criteria && task.acceptance_criteria.length > 0 && (
                      <div>
                        <div className="font-medium text-sm mb-1">Acceptance Criteria</div>
                        <ul className="list-disc list-inside space-y-1 text-sm text-[hsl(var(--muted-foreground))]">
                          {task.acceptance_criteria.map((criterion, idx) => (
                            <li key={idx}>{criterion}</li>
                          ))}
                        </ul>
                      </div>
                    )}
                    {task.tags && task.tags.length > 0 && (
                      <div className="flex gap-2 flex-wrap">
                        {task.tags.map((tag) => (
                          <Badge key={tag} variant="outline">{tag}</Badge>
                        ))}
                      </div>
                    )}
                    <div className="text-xs text-[hsl(var(--muted-foreground))] flex gap-3">
                      {task.created && <span>Created: {task.created}</span>}
                      {task.updated && <span>Updated: {task.updated}</span>}
                      {task.completed && <span>Completed: {task.completed}</span>}
                    </div>
                  </div>
                </AccordionContent>
              </AccordionItem>
            ))}
          </Accordion>
        </div>
      )}

      {doneTasks.length > 0 && (
        <div>
          <h2 className="text-lg font-semibold mb-3">Done ({doneTasks.length})</h2>
          <Accordion type="single" collapsible className="space-y-2">
            {doneTasks.map((task) => (
              <AccordionItem key={task.id} value={task.id}>
                <AccordionTrigger>
                  <div className="flex items-center gap-2 flex-1">
                    <span>{task.title}</span>
                    <div className="flex gap-2">
                      {getStatusBadge(task.status)}
                      {getPriorityBadge(task.priority)}
                    </div>
                  </div>
                </AccordionTrigger>
                <AccordionContent>
                  <div className="space-y-3 pt-2">
                    {task.description && (
                      <div>
                        <div className="font-medium text-sm mb-1">Description</div>
                        <div className="text-sm text-[hsl(var(--muted-foreground))]">{task.description}</div>
                      </div>
                    )}
                    {task.blocked_by && (
                      <div>
                        <div className="font-medium text-sm mb-1">Blocked By</div>
                        <div className="text-sm text-[hsl(var(--muted-foreground))]">{task.blocked_by}</div>
                      </div>
                    )}
                    {task.depends_on && task.depends_on.length > 0 && (
                      <div>
                        <div className="font-medium text-sm mb-1">Dependencies</div>
                        <div className="flex gap-2 flex-wrap">
                          {task.depends_on.map((depId) => (
                            <Badge key={depId} variant="outline">{depId}</Badge>
                          ))}
                        </div>
                      </div>
                    )}
                    {task.acceptance_criteria && task.acceptance_criteria.length > 0 && (
                      <div>
                        <div className="font-medium text-sm mb-1">Acceptance Criteria</div>
                        <ul className="list-disc list-inside space-y-1 text-sm text-[hsl(var(--muted-foreground))]">
                          {task.acceptance_criteria.map((criterion, idx) => (
                            <li key={idx}>{criterion}</li>
                          ))}
                        </ul>
                      </div>
                    )}
                    {task.tags && task.tags.length > 0 && (
                      <div className="flex gap-2 flex-wrap">
                        {task.tags.map((tag) => (
                          <Badge key={tag} variant="outline">{tag}</Badge>
                        ))}
                      </div>
                    )}
                    <div className="text-xs text-[hsl(var(--muted-foreground))] flex gap-3">
                      {task.created && <span>Created: {task.created}</span>}
                      {task.updated && <span>Updated: {task.updated}</span>}
                      {task.completed && <span>Completed: {task.completed}</span>}
                    </div>
                  </div>
                </AccordionContent>
              </AccordionItem>
            ))}
          </Accordion>
        </div>
      )}

      {skippedTasks.length > 0 && (
        <div>
          <h2 className="text-lg font-semibold mb-3">Skipped ({skippedTasks.length})</h2>
          <Accordion type="single" collapsible className="space-y-2">
            {skippedTasks.map((task) => (
              <AccordionItem key={task.id} value={task.id}>
                <AccordionTrigger>
                  <div className="flex items-center gap-2 flex-1">
                    <span>{task.title}</span>
                    <div className="flex gap-2">
                      {getStatusBadge(task.status)}
                      {getPriorityBadge(task.priority)}
                    </div>
                  </div>
                </AccordionTrigger>
                <AccordionContent>
                  <div className="space-y-3 pt-2">
                    {task.description && (
                      <div>
                        <div className="font-medium text-sm mb-1">Description</div>
                        <div className="text-sm text-[hsl(var(--muted-foreground))]">{task.description}</div>
                      </div>
                    )}
                    {task.blocked_by && (
                      <div>
                        <div className="font-medium text-sm mb-1">Blocked By</div>
                        <div className="text-sm text-[hsl(var(--muted-foreground))]">{task.blocked_by}</div>
                      </div>
                    )}
                    {task.depends_on && task.depends_on.length > 0 && (
                      <div>
                        <div className="font-medium text-sm mb-1">Dependencies</div>
                        <div className="flex gap-2 flex-wrap">
                          {task.depends_on.map((depId) => (
                            <Badge key={depId} variant="outline">{depId}</Badge>
                          ))}
                        </div>
                      </div>
                    )}
                    {task.acceptance_criteria && task.acceptance_criteria.length > 0 && (
                      <div>
                        <div className="font-medium text-sm mb-1">Acceptance Criteria</div>
                        <ul className="list-disc list-inside space-y-1 text-sm text-[hsl(var(--muted-foreground))]">
                          {task.acceptance_criteria.map((criterion, idx) => (
                            <li key={idx}>{criterion}</li>
                          ))}
                        </ul>
                      </div>
                    )}
                    {task.tags && task.tags.length > 0 && (
                      <div className="flex gap-2 flex-wrap">
                        {task.tags.map((tag) => (
                          <Badge key={tag} variant="outline">{tag}</Badge>
                        ))}
                      </div>
                    )}
                    <div className="text-xs text-[hsl(var(--muted-foreground))] flex gap-3">
                      {task.created && <span>Created: {task.created}</span>}
                      {task.updated && <span>Updated: {task.updated}</span>}
                      {task.completed && <span>Completed: {task.completed}</span>}
                    </div>
                  </div>
                </AccordionContent>
              </AccordionItem>
            ))}
          </Accordion>
        </div>
      )}
    </div>
  );
}
