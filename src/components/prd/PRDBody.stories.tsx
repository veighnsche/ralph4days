import type { Meta, StoryObj } from '@storybook/react'
import type { TaskListItem } from '@/types/generated'
import { PRDBody } from './PRDBody'

const meta = {
  title: 'Components/PRD/PRDBody',
  component: PRDBody,
  tags: ['autodocs'],
  args: {
    filteredTasks: [],
    totalTasks: 0,
    cropsStore: new Map(),
    onClearFilters: () => {}
  }
} satisfies Meta<typeof PRDBody>

export default meta
type Story = StoryObj<typeof meta>

const mockTasks: TaskListItem[] = [
  {
    id: 1,
    subsystem: 'authentication',
    discipline: 'backend',
    title: 'Implement login API',
    description: 'Create REST API endpoints for user authentication',
    status: 'done',
    priority: 'high',
    tags: ['api', 'security'],
    dependsOn: [],
    acceptanceCriteriaCount: 0,
    signalCount: 0,
    subsystemDisplayName: 'Authentication',
    subsystemAcronym: 'AUTH',
    disciplineDisplayName: 'Backend',
    disciplineAcronym: 'BKND',
    disciplineIcon: 'server',
    disciplineColor: '#8B5CF6'
  },
  {
    id: 2,
    subsystem: 'authentication',
    discipline: 'frontend',
    title: 'Build login form',
    description: 'Create UI for user login',
    status: 'in_progress',
    priority: 'medium',
    tags: ['ui'],
    dependsOn: [1],
    acceptanceCriteriaCount: 0,
    signalCount: 0,
    subsystemDisplayName: 'Authentication',
    subsystemAcronym: 'AUTH',
    disciplineDisplayName: 'Frontend',
    disciplineAcronym: 'FRNT',
    disciplineIcon: 'code',
    disciplineColor: '#3B82F6'
  },
  {
    id: 3,
    subsystem: 'user-profile',
    discipline: 'backend',
    title: 'Profile API endpoints',
    status: 'pending',
    priority: 'low',
    tags: [],
    dependsOn: [],
    acceptanceCriteriaCount: 0,
    signalCount: 0,
    subsystemDisplayName: 'User Profile',
    subsystemAcronym: 'USER',
    disciplineDisplayName: 'Backend',
    disciplineAcronym: 'BKND',
    disciplineIcon: 'server',
    disciplineColor: '#8B5CF6'
  }
]

export const WithTasks: Story = {
  args: {
    filteredTasks: mockTasks,
    totalTasks: mockTasks.length
  }
}

export const Empty: Story = {
  args: {
    filteredTasks: []
  }
}
