import { useCallback, useMemo, useState } from "react";
import type { EnrichedTask, PriorityFilter, StatusFilter } from "@/types/prd";

export interface FilterState {
  searchQuery: string;
  statusFilter: StatusFilter;
  priorityFilter: PriorityFilter;
  tagFilter: string;
}

export interface FilterSetters {
  setSearchQuery: (query: string) => void;
  setStatusFilter: (filter: StatusFilter) => void;
  setPriorityFilter: (filter: PriorityFilter) => void;
  setTagFilter: (tag: string) => void;
}

export function usePRDFilters(tasks: EnrichedTask[] | null, allTags: string[]) {
  const [searchQuery, setSearchQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState<StatusFilter>("all");
  const [priorityFilter, setPriorityFilter] = useState<PriorityFilter>("all");
  const [tagFilter, setTagFilter] = useState<string>("all");

  const filteredTasks = useMemo(() => {
    if (!tasks) return [];

    let filtered = [...tasks];

    // Search filter
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(
        (task) =>
          task.title.toLowerCase().includes(query) ||
          task.description?.toLowerCase().includes(query) ||
          task.id.toString().includes(query) ||
          task.feature.toLowerCase().includes(query) ||
          task.discipline.toLowerCase().includes(query) ||
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
  }, [tasks, searchQuery, statusFilter, priorityFilter, tagFilter]);

  const clearFilters = useCallback(() => {
    setSearchQuery("");
    setStatusFilter("all");
    setPriorityFilter("all");
    setTagFilter("all");
  }, []);

  const filters: FilterState = {
    searchQuery,
    statusFilter,
    priorityFilter,
    tagFilter,
  };

  const setters: FilterSetters = {
    setSearchQuery,
    setStatusFilter,
    setPriorityFilter,
    setTagFilter,
  };

  return {
    filters,
    setters,
    filteredTasks,
    allTags,
    clearFilters,
  };
}
