import type { Meta, StoryObj } from '@storybook/react'
import type { Task } from '@/types/generated'
import { PlaylistView } from './PlaylistView'

const meta = {
  title: 'Components/PRD/PlaylistView',
  component: PlaylistView,
  tags: ['autodocs'],
  args: {
    onTaskClick: () => {},
    cropsStore: new Map()
  }
} satisfies Meta<typeof PlaylistView>

export default meta
type Story = StoryObj<typeof meta>

const mockTasks: Task[] = [
  {
    id: 1,
    feature: 'authentication',
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
    featureDisplayName: 'Authentication',
    signals: [],
    featureAcronym: 'AUTH',
    disciplineDisplayName: 'Backend',
    disciplineAcronym: 'BKND',
    disciplineIcon: 'server',
    disciplineColor: '#8B5CF6'
  },
  {
    id: 2,
    feature: 'authentication',
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
    featureDisplayName: 'Authentication',
    signals: [],
    featureAcronym: 'AUTH',
    disciplineDisplayName: 'Frontend',
    disciplineAcronym: 'FRNT',
    disciplineIcon: 'code',
    disciplineColor: '#3B82F6'
  },
  {
    id: 3,
    feature: 'user-profile',
    discipline: 'backend',
    title: 'Profile API endpoints',
    status: 'pending',
    priority: 'low',
    tags: [],
    dependsOn: [],
    acceptanceCriteria: [],
    contextFiles: [],
    outputArtifacts: [],
    featureDisplayName: 'User Profile',
    signals: [],
    featureAcronym: 'USER',
    disciplineDisplayName: 'Backend',
    disciplineAcronym: 'BKND',
    disciplineIcon: 'server',
    disciplineColor: '#8B5CF6'
  },
  {
    id: 4,
    feature: 'user-profile',
    discipline: 'frontend',
    title: 'Profile page UI',
    status: 'pending',
    priority: 'medium',
    tags: ['ui'],
    dependsOn: [3],
    acceptanceCriteria: [],
    contextFiles: [],
    outputArtifacts: [],
    featureDisplayName: 'User Profile',
    signals: [],
    featureAcronym: 'USER',
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
        feature: 'payments',
        discipline: 'backend',
        title: 'Integrate payment gateway',
        status: 'blocked',
        priority: 'critical',
        tags: ['payments'],
        dependsOn: [],
        acceptanceCriteria: [],
        contextFiles: [],
        outputArtifacts: [],
        featureDisplayName: 'Payments',
        signals: [],
        featureAcronym: 'PAYM',
        disciplineDisplayName: 'Backend',
        disciplineAcronym: 'BKND',
        disciplineIcon: 'server',
        disciplineColor: '#8B5CF6'
      },
      {
        id: 6,
        feature: 'notifications',
        discipline: 'backend',
        title: 'Email notifications',
        status: 'skipped',
        priority: 'low',
        tags: ['email'],
        dependsOn: [],
        acceptanceCriteria: [],
        contextFiles: [],
        outputArtifacts: [],
        featureDisplayName: 'Notifications',
        signals: [],
        featureAcronym: 'NOTI',
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
