import { X } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import type { FilterSetters, FilterState } from '@/hooks/usePRDFilters'

interface ActiveFiltersProps {
  filters: FilterState
  setters: FilterSetters
}

export function ActiveFilters({ filters, setters }: ActiveFiltersProps) {
  const activeFilters: Array<{ key: string; label: string; onRemove: () => void }> = []

  if (filters.searchQuery) {
    activeFilters.push({
      key: 'search',
      label: `Search: "${filters.searchQuery}"`,
      onRemove: () => setters.setSearchQuery('')
    })
  }

  if (filters.statusFilter !== 'all') {
    activeFilters.push({
      key: 'status',
      label: `Status: ${formatFilterValue(filters.statusFilter)}`,
      onRemove: () => setters.setStatusFilter('all')
    })
  }

  if (filters.priorityFilter !== 'all') {
    activeFilters.push({
      key: 'priority',
      label: `Priority: ${formatFilterValue(filters.priorityFilter)}`,
      onRemove: () => setters.setPriorityFilter('all')
    })
  }

  if (filters.tagFilter !== 'all') {
    activeFilters.push({
      key: 'tag',
      label: `Tag: ${filters.tagFilter}`,
      onRemove: () => setters.setTagFilter('all')
    })
  }

  if (activeFilters.length === 0) {
    return null
  }

  return (
    <div className="flex flex-wrap gap-1.5">
      {activeFilters.map(filter => (
        <Badge key={filter.key} variant="secondary" className="gap-1 pr-1">
          <span className="text-[10px]">{filter.label}</span>
          <button
            type="button"
            onClick={filter.onRemove}
            className="rounded-full hover:bg-muted-foreground/20 p-0.5 transition-colors"
            aria-label={`Remove ${filter.key} filter`}>
            <X className="h-2.5 w-2.5" />
          </button>
        </Badge>
      ))}
    </div>
  )
}

function formatFilterValue(value: string): string {
  return value
    .split('_')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ')
}
