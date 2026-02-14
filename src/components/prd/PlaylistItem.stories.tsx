import type { Meta, StoryObj } from '@storybook/react'
import { type Acronym, acronym } from '@/types/acronym'
import type { TaskListItem } from '@/types/generated'
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
const DISCIPLINE_DEFAULTS: Record<string, { displayName: string; acronym: Acronym; icon: string; color: string }> = {
  frontend: { displayName: 'Frontend', acronym: acronym('FRNT'), icon: 'code', color: '#3B82F6' },
  backend: { displayName: 'Backend', acronym: acronym('BKND'), icon: 'server', color: '#8B5CF6' },
  database: { displayName: 'Database', acronym: acronym('DATA'), icon: 'database', color: '#F59E0B' },
  testing: { displayName: 'Testing', acronym: acronym('TEST'), icon: 'flask-conical', color: '#10B981' },
  infra: { displayName: 'Infrastructure', acronym: acronym('INFR'), icon: 'cloud', color: '#6366F1' },
  security: { displayName: 'Security', acronym: acronym('SECU'), icon: 'shield', color: '#EF4444' },
  docs: { displayName: 'Documentation', acronym: acronym('DOCS'), icon: 'book-open', color: '#64748B' },
  design: { displayName: 'Design', acronym: acronym('DSGN'), icon: 'palette', color: '#EC4899' },
  promo: { displayName: 'Promotion', acronym: acronym('PRMO'), icon: 'megaphone', color: '#F97316' },
  api: { displayName: 'API', acronym: acronym('APIS'), icon: 'plug', color: '#14B8A6' }
}

const SUBSYSTEM_DEFAULTS = {
  ui: { displayName: 'UI', acronym: acronym('UIUX') },
  api: { displayName: 'API', acronym: acronym('APIS') },
  data: { displayName: 'Data', acronym: acronym('DATA') },
  tests: { displayName: 'Tests', acronym: acronym('TEST') },
  deploy: { displayName: 'Deploy', acronym: acronym('DEPL') },
  sec: { displayName: 'Security', acronym: acronym('SECU') },
  docs: { displayName: 'Docs', acronym: acronym('DOCS') },
  campaign: { displayName: 'Campaign', acronym: acronym('CMGN') },
  rest: { displayName: 'REST', acronym: acronym('REST') }
} as const

type StorySubsystem = keyof typeof SUBSYSTEM_DEFAULTS
type StoryDiscipline = keyof typeof DISCIPLINE_DEFAULTS

function enrichFields(subsystem: StorySubsystem, discipline: StoryDiscipline) {
  const subsystemDefaults = SUBSYSTEM_DEFAULTS[subsystem]
  const disc = DISCIPLINE_DEFAULTS[discipline]
  return {
    subsystemDisplayName: subsystemDefaults.displayName,
    subsystemAcronym: subsystemDefaults.acronym,
    disciplineDisplayName: disc.displayName,
    disciplineAcronym: disc.acronym,
    disciplineIcon: disc.icon,
    disciplineColor: disc.color
  }
}

const baseTask: TaskListItem = {
  id: 1,
  subsystem: 'ui',
  discipline: 'frontend',
  title: 'Design main dashboard layout',
  description: 'Create responsive dashboard with sidebar and main content area',
  status: 'pending',
  priority: 'high',
  tags: ['design', 'layout'],
  dependsOn: [],
  acceptanceCriteriaCount: 3,
  signalCount: 0,
  ...enrichFields('ui', 'frontend')
}

export const Pending: Story = {
  args: {
    task: baseTask
  }
}

export const InProgress: Story = {
  args: {
    task: {
      ...baseTask,
      id: 2,
      subsystem: 'api',
      discipline: 'backend',
      title: 'Implement task list component',
      status: 'in_progress',
      ...enrichFields('api', 'backend')
    },
    isNowPlaying: true
  }
}

export const Done: Story = {
  args: {
    task: {
      ...baseTask,
      id: 3,
      subsystem: 'data',
      discipline: 'database',
      title: 'Setup REST API endpoints',
      status: 'done',
      ...enrichFields('data', 'database')
    }
  }
}

export const Blocked: Story = {
  args: {
    task: {
      ...baseTask,
      id: 4,
      subsystem: 'tests',
      discipline: 'testing',
      title: 'Add authentication middleware',
      status: 'blocked',
      ...enrichFields('tests', 'testing')
    }
  }
}

export const Skipped: Story = {
  args: {
    task: {
      ...baseTask,
      id: 5,
      subsystem: 'deploy',
      discipline: 'infra',
      title: 'Deploy to staging environment',
      status: 'skipped',
      ...enrichFields('deploy', 'infra')
    }
  }
}

export const LowPriority: Story = {
  args: {
    task: {
      ...baseTask,
      title: 'Update documentation',
      priority: 'low'
    }
  }
}

export const CriticalPriority: Story = {
  args: {
    task: {
      ...baseTask,
      id: 6,
      subsystem: 'sec',
      discipline: 'security',
      title: 'Fix critical security vulnerability',
      priority: 'critical',
      status: 'in_progress',
      ...enrichFields('sec', 'security')
    },
    isNowPlaying: true
  }
}

export const WithDependencies: Story = {
  args: {
    task: {
      ...baseTask,
      title: 'Add task detail sidebar',
      dependsOn: [2, 3],
      tags: ['component', 'ui', 'depends-on-others']
    }
  }
}

export const NoDescription: Story = {
  args: {
    task: {
      id: 7,
      subsystem: 'docs',
      discipline: 'docs',
      title: 'Write API documentation',
      status: 'pending' as const,
      priority: 'medium' as const,
      tags: [],
      dependsOn: [],
      acceptanceCriteriaCount: 0,
      signalCount: 0,
      ...enrichFields('docs', 'docs')
    }
  }
}

export const AllDisciplines: Story = {
  args: {
    task: baseTask
  },
  render: () => (
    <div className="flex flex-col gap-2">
      <PlaylistItem
        task={{
          ...baseTask,
          id: 1,
          subsystem: 'ui',
          discipline: 'frontend',
          title: 'Frontend Task',
          ...enrichFields('ui', 'frontend')
        }}
      />
      <PlaylistItem
        task={{
          ...baseTask,
          id: 2,
          subsystem: 'api',
          discipline: 'backend',
          title: 'Backend Task',
          ...enrichFields('api', 'backend')
        }}
      />
      <PlaylistItem
        task={{
          ...baseTask,
          id: 3,
          subsystem: 'data',
          discipline: 'database',
          title: 'Database Task',
          ...enrichFields('data', 'database')
        }}
      />
      <PlaylistItem
        task={{
          ...baseTask,
          id: 4,
          subsystem: 'tests',
          discipline: 'testing',
          title: 'Testing Task',
          ...enrichFields('tests', 'testing')
        }}
      />
      <PlaylistItem
        task={{
          ...baseTask,
          id: 5,
          subsystem: 'deploy',
          discipline: 'infra',
          title: 'Infrastructure Task',
          ...enrichFields('deploy', 'infra')
        }}
      />
      <PlaylistItem
        task={{
          ...baseTask,
          id: 6,
          subsystem: 'sec',
          discipline: 'security',
          title: 'Security Task',
          ...enrichFields('sec', 'security')
        }}
      />
      <PlaylistItem
        task={{
          ...baseTask,
          id: 7,
          subsystem: 'docs',
          discipline: 'docs',
          title: 'Documentation Task',
          ...enrichFields('docs', 'docs')
        }}
      />
      <PlaylistItem
        task={{
          ...baseTask,
          id: 8,
          subsystem: 'ui',
          discipline: 'design',
          title: 'Design Task',
          ...enrichFields('ui', 'design')
        }}
      />
      <PlaylistItem
        task={{
          ...baseTask,
          id: 9,
          subsystem: 'campaign',
          discipline: 'promo',
          title: 'Marketing Task',
          ...enrichFields('campaign', 'promo')
        }}
      />
      <PlaylistItem
        task={{
          ...baseTask,
          id: 10,
          subsystem: 'rest',
          discipline: 'api',
          title: 'API Task',
          ...enrichFields('rest', 'api')
        }}
      />
    </div>
  )
}
