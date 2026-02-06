import { Filter } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import type { FilterSetters, FilterState } from "@/hooks/usePRDFilters";

interface FiltersModalProps {
  filters: FilterState;
  setters: FilterSetters;
  allTags: string[];
  onClearFilters: () => void;
}

export function FiltersModal({ filters, setters, allTags, onClearFilters }: FiltersModalProps) {
  const [open, setOpen] = useState(false);

  const hasActiveFilters =
    filters.searchQuery ||
    filters.statusFilter !== "all" ||
    filters.priorityFilter !== "all" ||
    filters.tagFilter !== "all";

  const handleClearFilters = () => {
    onClearFilters();
    setOpen(false);
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button variant={hasActiveFilters ? "default" : "outline"} size="sm" className="h-8 gap-1.5">
          <Filter className="h-3.5 w-3.5" />
          Filters
          {hasActiveFilters && (
            <span className="ml-1 rounded-full bg-secondary-foreground text-secondary px-1.5 py-0.5 text-[10px] font-semibold">
              •
            </span>
          )}
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-[380px]">
        <DialogHeader>
          <DialogTitle>Filter Tasks</DialogTitle>
          <DialogDescription>Set filters to narrow down your task list</DialogDescription>
        </DialogHeader>
        <div className="grid gap-3 py-3">
          {/* Search */}
          <div className="grid gap-2">
            <Label htmlFor="search">Search</Label>
            <Input
              id="search"
              placeholder="Search tasks..."
              value={filters.searchQuery}
              onChange={(e) => setters.setSearchQuery(e.target.value)}
            />
          </div>

          {/* Status */}
          <div className="grid gap-2">
            <Label htmlFor="status">Status</Label>
            <Select value={filters.statusFilter} onValueChange={setters.setStatusFilter}>
              <SelectTrigger id="status">
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
          </div>

          {/* Priority */}
          <div className="grid gap-2">
            <Label htmlFor="priority">Priority</Label>
            <Select value={filters.priorityFilter} onValueChange={setters.setPriorityFilter}>
              <SelectTrigger id="priority">
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
          </div>

          {/* Tags */}
          {allTags.length > 0 && (
            <div className="grid gap-2">
              <Label htmlFor="tag">Tag</Label>
              <Select value={filters.tagFilter} onValueChange={setters.setTagFilter}>
                <SelectTrigger id="tag">
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
            </div>
          )}
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={handleClearFilters} disabled={!hasActiveFilters}>
            Clear All
          </Button>
          <Button onClick={() => setOpen(false)}>Done</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
