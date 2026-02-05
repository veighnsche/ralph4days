import { useEffect, useState, useCallback, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Skeleton } from "@/components/ui/skeleton";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { LayoutList, LayoutGrid, Search, Filter } from "lucide-react";
import yaml from "js-yaml";
import { TaskListView } from "./TaskListView";
import { KanbanView } from "./KanbanView";
import { TaskDetailSidebar } from "./TaskDetailSidebar";

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

type ViewMode = "list" | "kanban";
type GroupBy = "status" | "priority" | "none";
type StatusFilter = "all" | "pending" | "in_progress" | "done" | "blocked" | "skipped";
type PriorityFilter = "all" | "low" | "medium" | "high" | "critical";

export function PRDViewer() {
  const [prdData, setPrdData] = useState<PRDData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // View state
  const [viewMode, setViewMode] = useState<ViewMode>("list");
  const [groupBy, setGroupBy] = useState<GroupBy>("status");

  // Filter state
  const [searchQuery, setSearchQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState<StatusFilter>("all");
  const [priorityFilter, setPriorityFilter] = useState<PriorityFilter>("all");
  const [tagFilter, setTagFilter] = useState<string>("all");

  // Task detail sidebar
  const [selectedTask, setSelectedTask] = useState<PRDTask | null>(null);
  const [sidebarOpen, setSidebarOpen] = useState(false);

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

  // Memoize filtered tasks to avoid recalculation on every render
  const filteredTasks = useMemo(() => {
    if (!prdData) return [];

    let filtered = [...prdData.tasks];

    // Search filter
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(
        (task) =>
          task.title.toLowerCase().includes(query) ||
          task.description?.toLowerCase().includes(query) ||
          task.id.toLowerCase().includes(query) ||
          task.tags?.some((tag) => tag.toLowerCase().includes(query))
      );
    }

    // Status filter
    if (statusFilter !== "all") {
      filtered = filtered.filter((task) => task.status === statusFilter);
    }

    // Priority filter
    if (priorityFilter !== "all") {
      filtered = filtered.filter((task) => task.priority === priorityFilter);
    }

    // Tag filter
    if (tagFilter !== "all") {
      filtered = filtered.filter((task) => task.tags?.includes(tagFilter));
    }

    return filtered;
  }, [prdData, searchQuery, statusFilter, priorityFilter, tagFilter]);

  const allTags = useMemo(() => {
    if (!prdData) return [];
    const tags = new Set<string>();
    prdData.tasks.forEach((task) => {
      task.tags?.forEach((tag) => tags.add(tag));
    });
    return Array.from(tags).sort();
  }, [prdData]);

  const doneTasks = useMemo(() =>
    prdData ? prdData.tasks.filter((t) => t.status === "done") : [],
    [prdData]
  );

  const handleTaskClick = useCallback((task: PRDTask) => {
    setSelectedTask(task);
    setSidebarOpen(true);
  }, []);

  const handleNavigateNext = useCallback(() => {
    if (!selectedTask) return;
    const currentIndex = filteredTasks.findIndex(t => t.id === selectedTask.id);
    if (currentIndex < filteredTasks.length - 1) {
      setSelectedTask(filteredTasks[currentIndex + 1]);
    }
  }, [selectedTask, filteredTasks]);

  const handleNavigatePrev = useCallback(() => {
    if (!selectedTask) return;
    const currentIndex = filteredTasks.findIndex(t => t.id === selectedTask.id);
    if (currentIndex > 0) {
      setSelectedTask(filteredTasks[currentIndex - 1]);
    }
  }, [selectedTask, filteredTasks]);

  // Keyboard navigation
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!sidebarOpen || !selectedTask) return;

      if (e.key === "ArrowUp" || e.key === "k") {
        e.preventDefault();
        handleNavigatePrev();
      } else if (e.key === "ArrowDown" || e.key === "j") {
        e.preventDefault();
        handleNavigateNext();
      } else if (e.key === "Escape") {
        e.preventDefault();
        setSidebarOpen(false);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [sidebarOpen, selectedTask, handleNavigateNext, handleNavigatePrev]);

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

  const totalTasks = prdData.tasks.length;
  const progressPercent = totalTasks > 0 ? Math.round((doneTasks.length / totalTasks) * 100) : 0;

  return (
    <div className="space-y-4">
      {/* Project Header */}
      <Card>
        <CardHeader>
          <div className="flex items-start justify-between">
            <div className="flex-1">
              <CardTitle>{prdData.project.title}</CardTitle>
              {prdData.project.description && (
                <CardDescription className="mt-1">{prdData.project.description}</CardDescription>
              )}
            </div>
            <div className="text-right">
              <div className="text-2xl font-bold">{progressPercent}%</div>
              <div className="text-xs text-[hsl(var(--muted-foreground))]">Complete</div>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="flex gap-4 text-sm">
            <div className="flex items-center gap-2">
              <span className="text-[hsl(var(--muted-foreground))]">Total:</span>
              <Badge variant="outline">{totalTasks}</Badge>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-[hsl(var(--muted-foreground))]">Done:</span>
              <Badge variant="success">{doneTasks.length}</Badge>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-[hsl(var(--muted-foreground))]">Remaining:</span>
              <Badge variant="outline">{totalTasks - doneTasks.length}</Badge>
            </div>
            {prdData.project.created && (
              <div className="ml-auto text-[hsl(var(--muted-foreground))]">
                Created: {prdData.project.created}
              </div>
            )}
          </div>
          {/* Progress Bar */}
          <div className="mt-4 w-full bg-[hsl(var(--muted))] rounded-full h-2">
            <div
              className="bg-[hsl(var(--primary))] h-2 rounded-full transition-all"
              style={{ width: `${progressPercent}%` }}
            />
          </div>
        </CardContent>
      </Card>

      {/* Controls Bar */}
      <Card className="p-4">
        <div className="flex flex-wrap gap-3">
          {/* Search */}
          <div className="flex-1 min-w-[200px]">
            <div className="relative">
              <Search className="absolute left-2 top-2.5 h-4 w-4 text-[hsl(var(--muted-foreground))]" />
              <Input
                placeholder="Search tasks..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="pl-8"
              />
            </div>
          </div>

          {/* View Mode Toggle */}
          <div className="flex gap-1 border rounded-md">
            <Button
              variant={viewMode === "list" ? "default" : "ghost"}
              size="sm"
              onClick={() => setViewMode("list")}
            >
              <LayoutList className="h-4 w-4" />
            </Button>
            <Button
              variant={viewMode === "kanban" ? "default" : "ghost"}
              size="sm"
              onClick={() => setViewMode("kanban")}
            >
              <LayoutGrid className="h-4 w-4" />
            </Button>
          </div>

          {/* Status Filter */}
          <Select value={statusFilter} onValueChange={(v) => setStatusFilter(v as StatusFilter)}>
            <SelectTrigger className="w-[150px]">
              <Filter className="h-4 w-4 mr-2" />
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Status</SelectItem>
              <SelectItem value="blocked">Blocked</SelectItem>
              <SelectItem value="in_progress">In Progress</SelectItem>
              <SelectItem value="pending">Pending</SelectItem>
              <SelectItem value="done">Done</SelectItem>
              <SelectItem value="skipped">Skipped</SelectItem>
            </SelectContent>
          </Select>

          {/* Priority Filter */}
          <Select value={priorityFilter} onValueChange={(v) => setPriorityFilter(v as PriorityFilter)}>
            <SelectTrigger className="w-[150px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Priority</SelectItem>
              <SelectItem value="critical">Critical</SelectItem>
              <SelectItem value="high">High</SelectItem>
              <SelectItem value="medium">Medium</SelectItem>
              <SelectItem value="low">Low</SelectItem>
            </SelectContent>
          </Select>

          {/* Tag Filter */}
          {allTags.length > 0 && (
            <Select value={tagFilter} onValueChange={setTagFilter}>
              <SelectTrigger className="w-[150px]">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Tags</SelectItem>
                {allTags.map((tag) => (
                  <SelectItem key={tag} value={tag}>
                    {tag}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          )}

          {/* Group By (List view only) */}
          {viewMode === "list" && (
            <Select value={groupBy} onValueChange={(v) => setGroupBy(v as GroupBy)}>
              <SelectTrigger className="w-[150px]">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="status">Group by Status</SelectItem>
                <SelectItem value="priority">Group by Priority</SelectItem>
                <SelectItem value="none">No Grouping</SelectItem>
              </SelectContent>
            </Select>
          )}
        </div>

        {/* Active Filters Summary */}
        {(searchQuery || statusFilter !== "all" || priorityFilter !== "all" || tagFilter !== "all") && (
          <div className="mt-3 flex items-center gap-2 text-sm">
            <span className="text-[hsl(var(--muted-foreground))]">Active filters:</span>
            {searchQuery && <Badge variant="outline">Search: "{searchQuery}"</Badge>}
            {statusFilter !== "all" && <Badge variant="outline">Status: {statusFilter}</Badge>}
            {priorityFilter !== "all" && <Badge variant="outline">Priority: {priorityFilter}</Badge>}
            {tagFilter !== "all" && <Badge variant="outline">Tag: {tagFilter}</Badge>}
            <Button
              variant="ghost"
              size="sm"
              onClick={() => {
                setSearchQuery("");
                setStatusFilter("all");
                setPriorityFilter("all");
                setTagFilter("all");
              }}
              className="ml-auto"
            >
              Clear all
            </Button>
          </div>
        )}

        <div className="mt-2 text-xs text-[hsl(var(--muted-foreground))]">
          Showing {filteredTasks.length} of {totalTasks} tasks
        </div>
      </Card>

      {/* Task Views */}
      {filteredTasks.length === 0 ? (
        <Alert>
          <AlertDescription>
            No tasks match your filters. Try adjusting your search criteria.
          </AlertDescription>
        </Alert>
      ) : viewMode === "list" ? (
        <TaskListView tasks={filteredTasks} groupBy={groupBy} onTaskClick={handleTaskClick} />
      ) : (
        <KanbanView tasks={filteredTasks} onTaskClick={handleTaskClick} />
      )}

      {/* Task Detail Sidebar */}
      <TaskDetailSidebar
        task={selectedTask}
        open={sidebarOpen}
        onClose={() => setSidebarOpen(false)}
        onNavigateNext={handleNavigateNext}
        onNavigatePrev={handleNavigatePrev}
      />
    </div>
  );
}
