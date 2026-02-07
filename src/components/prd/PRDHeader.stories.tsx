import type { Meta, StoryObj } from '@storybook/react'
import type { ProjectInfo } from '@/types/prd'
import { PRDHeader } from './PRDHeader'

const meta = {
  title: 'PRD/PRDHeader',
  component: PRDHeader,
  parameters: {
    layout: 'padded'
  },
  tags: ['autodocs']
} satisfies Meta<typeof PRDHeader>

export default meta
type Story = StoryObj<typeof meta>

const mockProject: ProjectInfo = {
  title: 'Ralph4days Development',
  description: 'Autonomous multi-agent build loops with Claude',
  created: '2026-02-01'
}

const mockFilters = {
  searchQuery: '',
  statusFilter: 'all' as const,
  priorityFilter: 'all' as const,
  tagFilter: 'all' as const
}

const mockSetters = {
  setSearchQuery: () => {},
  setStatusFilter: () => {},
  setPriorityFilter: () => {},
  setTagFilter: () => {}
}

export const NewProject: Story = {
  args: {
    project: mockProject,
    totalTasks: 0,
    doneTasks: 0,
    progressPercent: 0,
    filteredCount: 0,
    filters: mockFilters,
    setters: mockSetters,
    allTags: [],
    onClearFilters: () => console.log('Clear filters')
  }
}

export const JustStarted: Story = {
  args: {
    project: mockProject,
    totalTasks: 25,
    doneTasks: 3,
    progressPercent: 12,
    filteredCount: 25,
    filters: mockFilters,
    setters: mockSetters,
    allTags: ['frontend', 'backend', 'design'],
    onClearFilters: () => console.log('Clear filters')
  }
}

export const HalfwayComplete: Story = {
  args: {
    project: mockProject,
    totalTasks: 50,
    doneTasks: 25,
    progressPercent: 50,
    filteredCount: 50,
    filters: mockFilters,
    setters: mockSetters,
    allTags: ['frontend', 'backend', 'database', 'testing', 'design'],
    onClearFilters: () => console.log('Clear filters')
  }
}

export const NearlyComplete: Story = {
  args: {
    project: mockProject,
    totalTasks: 30,
    doneTasks: 28,
    progressPercent: 93,
    filteredCount: 30,
    filters: mockFilters,
    setters: mockSetters,
    allTags: ['frontend', 'backend', 'polish'],
    onClearFilters: () => console.log('Clear filters')
  }
}

export const Complete: Story = {
  args: {
    project: mockProject,
    totalTasks: 42,
    doneTasks: 42,
    progressPercent: 100,
    filteredCount: 42,
    filters: mockFilters,
    setters: mockSetters,
    allTags: ['frontend', 'backend', 'database', 'testing', 'design', 'docs'],
    onClearFilters: () => console.log('Clear filters')
  }
}

export const WithFiltersActive: Story = {
  args: {
    project: mockProject,
    totalTasks: 50,
    doneTasks: 20,
    progressPercent: 40,
    filteredCount: 8,
    filters: {
      searchQuery: 'api',
      statusFilter: 'in_progress' as const,
      priorityFilter: 'high' as const,
      tagFilter: 'backend'
    },
    setters: mockSetters,
    allTags: ['frontend', 'backend', 'database', 'testing', 'api'],
    onClearFilters: () => console.log('Clear filters')
  }
}

export const LongProjectName: Story = {
  args: {
    project: {
      title: 'Enterprise Customer Relationship Management System with Advanced Analytics',
      description:
        'A comprehensive CRM solution featuring real-time analytics, multi-tenant architecture, and AI-powered insights',
      created: '2026-01-15'
    },
    totalTasks: 247,
    doneTasks: 189,
    progressPercent: 77,
    filteredCount: 247,
    filters: mockFilters,
    setters: mockSetters,
    allTags: ['frontend', 'backend', 'database', 'api', 'analytics', 'ai', 'security'],
    onClearFilters: () => console.log('Clear filters')
  }
}
