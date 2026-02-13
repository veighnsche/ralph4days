import { useState } from 'react'
import type { Task } from '@/types/generated'

export type StatusFilter =
  | 'all'
  | 'draft'
  | 'pending'
  | 'in_progress'
  | 'blocked'
  | 'done'
  | 'skipped'
  | 'ready'
  | 'waiting_on_deps'
export type PriorityFilter = 'all' | 'low' | 'medium' | 'high' | 'critical'

export interface FilterState {
  searchQuery: string
  statusFilter: StatusFilter
  priorityFilter: PriorityFilter
  tagFilter: string
}

export interface FilterSetters {
  setSearchQuery: (query: string) => void
  setStatusFilter: (filter: StatusFilter) => void
  setPriorityFilter: (filter: PriorityFilter) => void
  setTagFilter: (tag: string) => void
}

export function usePRDFilters(tasks: Task[] | null, allTags: string[]) {
  const [searchQuery, setSearchQuery] = useState('')
  const [statusFilter, setStatusFilter] = useState<StatusFilter>('all')
  const [priorityFilter, setPriorityFilter] = useState<PriorityFilter>('all')
  const [tagFilter, setTagFilter] = useState<string>('all')

  const filteredTasks = (() => {
    if (!tasks) return []

    let filtered = [...tasks]

    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase()
      filtered = filtered.filter(
        task =>
          task.title.toLowerCase().includes(query) ||
          task.description?.toLowerCase().includes(query) ||
          task.id.toString().includes(query) ||
          task.subsystem.toLowerCase().includes(query) ||
          task.discipline.toLowerCase().includes(query) ||
          task.tags?.some(tag => tag.toLowerCase().includes(query))
      )
    }

    if (statusFilter !== 'all') {
      filtered = filtered.filter(task => task.status === statusFilter)
    }

    if (priorityFilter !== 'all') {
      filtered = filtered.filter(task => task.priority === priorityFilter)
    }

    if (tagFilter !== 'all') {
      filtered = filtered.filter(task => task.tags?.includes(tagFilter))
    }

    return filtered
  })()

  const clearFilters = () => {
    setSearchQuery('')
    setStatusFilter('all')
    setPriorityFilter('all')
    setTagFilter('all')
  }

  const filters: FilterState = {
    searchQuery,
    statusFilter,
    priorityFilter,
    tagFilter
  }

  const setters: FilterSetters = {
    setSearchQuery,
    setStatusFilter,
    setPriorityFilter,
    setTagFilter
  }

  return {
    filters,
    setters,
    filteredTasks,
    allTags,
    clearFilters
  }
}
