import type { Meta, StoryObj } from '@storybook/react'
import type { TaskListItem } from '@/types/generated'
import { PlaylistView } from './PlaylistView'

const meta = {
  title: 'Components/PRD/PlaylistView',
  component: PlaylistView,
  tags: ['autodocs'],
  args: {
    tasks: [],
    cropsStore: new Map()
  }
} satisfies Meta<typeof PlaylistView>

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
  },
  {
    id: 4,
    subsystem: 'user-profile',
    discipline: 'frontend',
    title: 'Profile page UI',
    status: 'pending',
    priority: 'medium',
    tags: ['ui'],
    dependsOn: [3],
    acceptanceCriteriaCount: 0,
    signalCount: 0,
    subsystemDisplayName: 'User Profile',
    subsystemAcronym: 'USER',
    disciplineDisplayName: 'Frontend',
    disciplineAcronym: 'FRNT',
    disciplineIcon: 'code',
    disciplineColor: '#3B82F6'
  }
]

export const Default: Story = {
  args: {
    tasks: mockTasks
  }
}

export const WithBlockedTasks: Story = {
  args: {
    tasks: [
      ...mockTasks,
      {
        id: 5,
        subsystem: 'payments',
        discipline: 'backend',
        title: 'Integrate payment gateway',
        status: 'blocked',
        priority: 'critical',
        tags: ['payments'],
        dependsOn: [],
        acceptanceCriteriaCount: 0,
        signalCount: 0,
        subsystemDisplayName: 'Payments',
        subsystemAcronym: 'PAYM',
        disciplineDisplayName: 'Backend',
        disciplineAcronym: 'BKND',
        disciplineIcon: 'server',
        disciplineColor: '#8B5CF6'
      },
      {
        id: 6,
        subsystem: 'notifications',
        discipline: 'backend',
        title: 'Email notifications',
        status: 'skipped',
        priority: 'low',
        tags: ['email'],
        dependsOn: [],
        acceptanceCriteriaCount: 0,
        signalCount: 0,
        subsystemDisplayName: 'Notifications',
        subsystemAcronym: 'NOTI',
        disciplineDisplayName: 'Backend',
        disciplineAcronym: 'BKND',
        disciplineIcon: 'server',
        disciplineColor: '#8B5CF6'
      }
    ]
  }
}

export const AllDone: Story = {
  args: {
    tasks: mockTasks.map(task => ({ ...task, status: 'done' as const }))
  }
}

export const AllPending: Story = {
  args: {
    tasks: mockTasks.map(task => ({
      ...task,
      status: 'pending' as const,
      dependsOn: []
    }))
  }
}

export const Empty: Story = {
  args: {
    tasks: []
  }
}
