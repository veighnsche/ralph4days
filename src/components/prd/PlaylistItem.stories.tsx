import type { Meta, StoryObj } from '@storybook/react'
import type { Task } from '@/types/generated'
import { PlaylistItem } from './PlaylistItem'

const meta = {
  title: 'PRD/PlaylistItem',
  component: PlaylistItem,
  parameters: {
    layout: 'padded'
  },
  tags: ['autodocs'],
  decorators: [
    Story => (
      <div className="max-w-4xl">
        <Story />
      </div>
    )
  ]
} satisfies Meta<typeof PlaylistItem>

export default meta
type Story = StoryObj<typeof meta>

/** Discipline defaults for story mocks */
const DISCIPLINE_DEFAULTS: Record<string, { displayName: string; acronym: string; icon: string; color: string }> = {
  frontend: { displayName: 'Frontend', acronym: 'FRNT', icon: 'code', color: '#3B82F6' },
  backend: { displayName: 'Backend', acronym: 'BKND', icon: 'server', color: '#8B5CF6' },
  database: { displayName: 'Database', acronym: 'DATA', icon: 'database', color: '#F59E0B' },
  testing: { displayName: 'Testing', acronym: 'TEST', icon: 'flask-conical', color: '#10B981' },
  infra: { displayName: 'Infrastructure', acronym: 'INFR', icon: 'cloud', color: '#6366F1' },
  security: { displayName: 'Security', acronym: 'SECU', icon: 'shield', color: '#EF4444' },
  docs: { displayName: 'Documentation', acronym: 'DOCS', icon: 'book-open', color: '#64748B' },
  design: { displayName: 'Design', acronym: 'DSGN', icon: 'palette', color: '#EC4899' },
  promo: { displayName: 'Promotion', acronym: 'PRMO', icon: 'megaphone', color: '#F97316' },
  api: { displayName: 'API', acronym: 'API', icon: 'plug', color: '#14B8A6' }
}

function featureDisplayName(feature: string): string {
  return feature
    .split('-')
    .map(w => w.charAt(0).toUpperCase() + w.slice(1))
    .join(' ')
}

function featureAcronym(feature: string): string {
  return feature.replace(/-/g, '').slice(0, 4).toUpperCase()
}

function enrichFields(feature: string, discipline: string) {
  const disc = DISCIPLINE_DEFAULTS[discipline] ?? {
    displayName: discipline.charAt(0).toUpperCase() + discipline.slice(1),
    acronym: discipline.slice(0, 4).toUpperCase(),
    icon: 'circle',
    color: '#6B7280'
  }
  return {
    featureDisplayName: featureDisplayName(feature),
    featureAcronym: featureAcronym(feature),
    disciplineDisplayName: disc.displayName,
    disciplineAcronym: disc.acronym,
    disciplineIcon: disc.icon,
    disciplineColor: disc.color
  }
}

const baseTask: Task = {
  id: 1,
  feature: 'ui',
  discipline: 'frontend',
  title: 'Design main dashboard layout',
  description: 'Create responsive dashboard with sidebar and main content area',
  status: 'pending',
  priority: 'high',
  tags: ['design', 'layout'],
  dependsOn: [],
  acceptanceCriteria: ['Responsive on mobile, tablet, desktop', 'Sidebar collapses on mobile', 'Dark mode support'],
  contextFiles: [],
  outputArtifacts: [],
  comments: [],
  created: '2026-02-01',
  updated: '2026-02-05',
  ...enrichFields('ui', 'frontend')
}

export const Pending: Story = {
  args: {
    task: baseTask,
    onClick: () => console.log('Task clicked')
  }
}

export const InProgress: Story = {
  args: {
    task: {
      ...baseTask,
      id: 2,
      feature: 'api',
      discipline: 'backend',
      title: 'Implement task list component',
      status: 'in_progress',
      ...enrichFields('api', 'backend')
    },
    isNowPlaying: true,
    onClick: () => console.log('Task clicked')
  }
}

export const Done: Story = {
  args: {
    task: {
      ...baseTask,
      id: 3,
      feature: 'data',
      discipline: 'database',
      title: 'Setup REST API endpoints',
      status: 'done',
      completed: '2026-02-03',
      ...enrichFields('data', 'database')
    },
    onClick: () => console.log('Task clicked')
  }
}

export const Blocked: Story = {
  args: {
    task: {
      ...baseTask,
      id: 4,
      feature: 'tests',
      discipline: 'testing',
      title: 'Add authentication middleware',
      status: 'blocked',
      blockedBy: 'Waiting for security review',
      ...enrichFields('tests', 'testing')
    },
    onClick: () => console.log('Task clicked')
  }
}

export const Skipped: Story = {
  args: {
    task: {
      ...baseTask,
      id: 5,
      feature: 'deploy',
      discipline: 'infra',
      title: 'Deploy to staging environment',
      status: 'skipped',
      ...enrichFields('deploy', 'infra')
    },
    onClick: () => console.log('Task clicked')
  }
}

export const LowPriority: Story = {
  args: {
    task: {
      ...baseTask,
      title: 'Update documentation',
      priority: 'low'
    },
    onClick: () => console.log('Task clicked')
  }
}

export const CriticalPriority: Story = {
  args: {
    task: {
      ...baseTask,
      id: 6,
      feature: 'sec',
      discipline: 'security',
      title: 'Fix critical security vulnerability',
      priority: 'critical',
      status: 'in_progress',
      ...enrichFields('sec', 'security')
    },
    isNowPlaying: true,
    onClick: () => console.log('Task clicked')
  }
}

export const WithDependencies: Story = {
  args: {
    task: {
      ...baseTask,
      title: 'Add task detail sidebar',
      dependsOn: [2, 3],
      tags: ['component', 'ui', 'depends-on-others']
    },
    onClick: () => console.log('Task clicked')
  }
}

export const NoDescription: Story = {
  args: {
    task: {
      id: 7,
      feature: 'docs',
      discipline: 'docs',
      title: 'Write API documentation',
      status: 'pending' as const,
      priority: 'medium' as const,
      tags: [],
      dependsOn: [],
      acceptanceCriteria: [],
      contextFiles: [],
      outputArtifacts: [],
      comments: [],
      ...enrichFields('docs', 'docs')
    },
    onClick: () => console.log('Task clicked')
  }
}

export const AllDisciplines: Story = {
  args: {
    task: baseTask,
    onClick: () => {}
  },
  render: () => (
    <div className="flex flex-col gap-2">
      <PlaylistItem
        task={{
          ...baseTask,
          id: 1,
          feature: 'ui',
          discipline: 'frontend',
          title: 'Frontend Task',
          ...enrichFields('ui', 'frontend')
        }}
        onClick={() => {}}
      />
      <PlaylistItem
        task={{
          ...baseTask,
          id: 2,
          feature: 'api',
          discipline: 'backend',
          title: 'Backend Task',
          ...enrichFields('api', 'backend')
        }}
        onClick={() => {}}
      />
      <PlaylistItem
        task={{
          ...baseTask,
          id: 3,
          feature: 'data',
          discipline: 'database',
          title: 'Database Task',
          ...enrichFields('data', 'database')
        }}
        onClick={() => {}}
      />
      <PlaylistItem
        task={{
          ...baseTask,
          id: 4,
          feature: 'tests',
          discipline: 'testing',
          title: 'Testing Task',
          ...enrichFields('tests', 'testing')
        }}
        onClick={() => {}}
      />
      <PlaylistItem
        task={{
          ...baseTask,
          id: 5,
          feature: 'deploy',
          discipline: 'infra',
          title: 'Infrastructure Task',
          ...enrichFields('deploy', 'infra')
        }}
        onClick={() => {}}
      />
      <PlaylistItem
        task={{
          ...baseTask,
          id: 6,
          feature: 'sec',
          discipline: 'security',
          title: 'Security Task',
          ...enrichFields('sec', 'security')
        }}
        onClick={() => {}}
      />
      <PlaylistItem
        task={{
          ...baseTask,
          id: 7,
          feature: 'docs',
          discipline: 'docs',
          title: 'Documentation Task',
          ...enrichFields('docs', 'docs')
        }}
        onClick={() => {}}
      />
      <PlaylistItem
        task={{
          ...baseTask,
          id: 8,
          feature: 'ui',
          discipline: 'design',
          title: 'Design Task',
          ...enrichFields('ui', 'design')
        }}
        onClick={() => {}}
      />
      <PlaylistItem
        task={{
          ...baseTask,
          id: 9,
          feature: 'campaign',
          discipline: 'promo',
          title: 'Marketing Task',
          ...enrichFields('campaign', 'promo')
        }}
        onClick={() => {}}
      />
      <PlaylistItem
        task={{
          ...baseTask,
          id: 10,
          feature: 'rest',
          discipline: 'api',
          title: 'API Task',
          ...enrichFields('rest', 'api')
        }}
        onClick={() => {}}
      />
    </div>
  )
}
