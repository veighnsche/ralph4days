import { Search } from "lucide-react";
import { Card, CardContent, CardDescription, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Progress } from "@/components/ui/progress";
import { Separator } from "@/components/ui/separator";
import type { FilterSetters, FilterState } from "@/hooks/usePRDFilters";
import type { PRDProject } from "@/types/prd";
import { ActiveFilters } from "./ActiveFilters";
import { FiltersModal } from "./FiltersModal";
import { TaskStatsBar } from "./TaskStatsBar";

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
    <div className="flex-shrink-0 p-3 pb-0">
      <Card className="py-3">
        <CardContent className="space-y-3">
          {/* Title Row */}
          <div className="flex items-center justify-between gap-3">
            <div className="flex-1 min-w-0">
              <CardTitle className="text-base">{project.title}</CardTitle>
              {project.description && (
                <CardDescription className="text-xs mt-0.5 line-clamp-1">{project.description}</CardDescription>
              )}
            </div>
            <TaskStatsBar totalTasks={totalTasks} doneTasks={doneTasks} progressPercent={progressPercent} />
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

            {/* Filters Modal */}
            <FiltersModal filters={filters} setters={setters} allTags={allTags} onClearFilters={onClearFilters} />

            <div className="text-[10px] text-[hsl(var(--muted-foreground))] ml-auto whitespace-nowrap">
              Showing {filteredCount} of {totalTasks}
            </div>
          </div>

          {/* Active Filters */}
          {hasActiveFilters && (
            <div className="flex items-center gap-2 pt-1">
              <ActiveFilters filters={filters} setters={setters} />
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
