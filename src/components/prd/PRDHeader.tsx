import { Filter, Search } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Progress } from "@/components/ui/progress";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";
import type { FilterSetters, FilterState } from "@/hooks/usePRDFilters";
import type { PRDProject } from "@/types/prd";

interface PRDHeaderProps {
  project: PRDProject;
  totalTasks: number;
  doneTasks: number;
  progressPercent: number;
  filteredCount: number;
  filters: FilterState;
  setters: FilterSetters;
  allTags: string[];
  onClearFilters: () => void;
}

export function PRDHeader({
  project,
  totalTasks,
  doneTasks,
  progressPercent,
  filteredCount,
  filters,
  setters,
  allTags,
  onClearFilters,
}: PRDHeaderProps) {
  const hasActiveFilters =
    filters.searchQuery ||
    filters.statusFilter !== "all" ||
    filters.priorityFilter !== "all" ||
    filters.tagFilter !== "all";

  return (
    <div className="flex-shrink-0 p-4 pb-0">
      <Card className="py-3">
        <CardContent className="space-y-3">
          {/* Title Row */}
          <div className="flex items-center justify-between gap-4">
            <div className="flex-1 min-w-0">
              <CardTitle className="text-base">{project.title}</CardTitle>
              {project.description && (
                <CardDescription className="text-xs mt-0.5 line-clamp-1">{project.description}</CardDescription>
              )}
            </div>
            <div className="flex items-center gap-6 text-xs">
              <div className="flex items-center gap-2">
                <span className="text-[hsl(var(--muted-foreground))]">Total:</span>
                <Badge variant="outline" className="h-5">
                  {totalTasks}
                </Badge>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-[hsl(var(--muted-foreground))]">Done:</span>
                <Badge variant="success" className="h-5">
                  {doneTasks}
                </Badge>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-[hsl(var(--muted-foreground))]">Remaining:</span>
                <Badge variant="outline" className="h-5">
                  {totalTasks - doneTasks}
                </Badge>
              </div>
              <div className="text-right min-w-[60px]">
                <div className="text-xl font-bold leading-none">{progressPercent}%</div>
                <div className="text-[10px] text-[hsl(var(--muted-foreground))] mt-0.5">Complete</div>
              </div>
            </div>
          </div>

          {/* Progress Bar */}
          <Progress value={progressPercent} className="h-1.5" />

          <Separator />

          {/* Filters Row */}
          <div className="flex items-center gap-2">
            {/* Search */}
            <div className="flex-1 max-w-xs">
              <div className="relative">
                <Search className="absolute left-2 top-2 h-3.5 w-3.5 text-[hsl(var(--muted-foreground))]" />
                <Input
                  placeholder="Search tasks..."
                  value={filters.searchQuery}
                  onChange={(e) => setters.setSearchQuery(e.target.value)}
                  className="pl-8 h-8 text-xs"
                />
              </div>
            </div>

            {/* Status Filter */}
            <Select value={filters.statusFilter} onValueChange={setters.setStatusFilter}>
              <SelectTrigger className="w-[130px] h-8 text-xs">
                <Filter className="h-3.5 w-3.5 mr-1.5" />
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
            <Select value={filters.priorityFilter} onValueChange={setters.setPriorityFilter}>
              <SelectTrigger className="w-[130px] h-8 text-xs">
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
              <Select value={filters.tagFilter} onValueChange={setters.setTagFilter}>
                <SelectTrigger className="w-[130px] h-8 text-xs">
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

            {/* Clear Filters Button */}
            {hasActiveFilters && (
              <Button variant="ghost" size="sm" onClick={onClearFilters} className="h-8 text-xs">
                Clear filters
              </Button>
            )}

            <div className="text-[10px] text-[hsl(var(--muted-foreground))] ml-auto whitespace-nowrap">
              Showing {filteredCount} of {totalTasks}
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
