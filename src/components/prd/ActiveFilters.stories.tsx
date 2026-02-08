import type { Meta, StoryObj } from '@storybook/react'
import type { FilterSetters } from '@/hooks/tasks'
import { ActiveFilters } from './ActiveFilters'

const meta = {
  title: 'Components/PRD/ActiveFilters',
  component: ActiveFilters,
  tags: ['autodocs']
} satisfies Meta<typeof ActiveFilters>

export default meta
type Story = StoryObj<typeof meta>

const mockSetters: FilterSetters = {
  setSearchQuery: () => {},
  setStatusFilter: () => {},
  setPriorityFilter: () => {},
  setTagFilter: () => {}
}

export const NoFilters: Story = {
  args: {
    filters: {
      searchQuery: '',
      statusFilter: 'all',
      priorityFilter: 'all',
      tagFilter: 'all'
    },
    setters: mockSetters
  }
}

export const SearchOnly: Story = {
  args: {
    filters: {
      searchQuery: 'authentication',
      statusFilter: 'all',
      priorityFilter: 'all',
      tagFilter: 'all'
    },
    setters: mockSetters
  }
}

export const StatusFilter: Story = {
  args: {
    filters: {
      searchQuery: '',
      statusFilter: 'in_progress',
      priorityFilter: 'all',
      tagFilter: 'all'
    },
    setters: mockSetters
  }
}

export const PriorityFilter: Story = {
  args: {
    filters: {
      searchQuery: '',
      statusFilter: 'all',
      priorityFilter: 'high',
      tagFilter: 'all'
    },
    setters: mockSetters
  }
}

export const TagFilter: Story = {
  args: {
    filters: {
      searchQuery: '',
      statusFilter: 'all',
      priorityFilter: 'all',
      tagFilter: 'security'
    },
    setters: mockSetters
  }
}

export const AllFilters: Story = {
  args: {
    filters: {
      searchQuery: 'login',
      statusFilter: 'in_progress',
      priorityFilter: 'high',
      tagFilter: 'security'
    },
    setters: mockSetters
  }
}

export const MultipleFilters: Story = {
  args: {
    filters: {
      searchQuery: 'user profile',
      statusFilter: 'pending',
      priorityFilter: 'all',
      tagFilter: 'ui'
    },
    setters: mockSetters
  }
}
