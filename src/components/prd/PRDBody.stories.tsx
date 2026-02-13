import type { Meta, StoryObj } from '@storybook/react'
import type { Task } from '@/types/generated'
import { PRDBody } from './PRDBody'

const meta = {
  title: 'Components/PRD/PRDBody',
  component: PRDBody,
  tags: ['autodocs'],
  args: {
    totalTasks: 0,
    cropsStore: new Map(),
    onTaskClick: () => {},
    onClearFilters: () => {}
  }
} satisfies Meta<typeof PRDBody>

export default meta
type Story = StoryObj<typeof meta>

const mockTasks: Task[] = [
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
    acceptanceCriteria: [],
    contextFiles: [],
    outputArtifacts: [],
    created: '2026-02-01',
    subsystemDisplayName: 'Authentication',
    signals: [],
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
    acceptanceCriteria: [],
    contextFiles: [],
    outputArtifacts: [],
    subsystemDisplayName: 'Authentication',
    signals: [],
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
    acceptanceCriteria: [],
    contextFiles: [],
    outputArtifacts: [],
    subsystemDisplayName: 'User Profile',
    signals: [],
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
