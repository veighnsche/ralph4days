import type { Meta, StoryObj } from '@storybook/react'
import { type Acronym, acronym } from '@/types/acronym'
import type { Task } from '@/types/generated'
import { TaskIdDisplay } from './TaskIdDisplay'

const meta = {
  title: 'PRD/TaskIdDisplay',
  component: TaskIdDisplay,
  parameters: {
    layout: 'centered'
  },
  tags: ['autodocs']
} satisfies Meta<typeof TaskIdDisplay>

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
  wiring: { displayName: 'Wiring', acronym: acronym('WIRE'), icon: 'cable', color: '#A855F7' },
  api: { displayName: 'API', acronym: acronym('APIS'), icon: 'plug', color: '#14B8A6' }
}

const SUBSYSTEM_DEFAULTS = {
  ui: { displayName: 'UI', acronym: acronym('UIUX') },
  api: { displayName: 'API', acronym: acronym('APIS') },
  data: { displayName: 'Data', acronym: acronym('DATA') },
  tests: { displayName: 'Tests', acronym: acronym('TEST') },
  authentication: { displayName: 'Authentication', acronym: acronym('AUTH') },
  'user-profile': { displayName: 'User Profile', acronym: acronym('USER') },
  payments: { displayName: 'Payments', acronym: acronym('PAYM') },
  search: { displayName: 'Search', acronym: acronym('SRCH') },
  notifications: { displayName: 'Notifications', acronym: acronym('NOTI') },
  deploy: { displayName: 'Deploy', acronym: acronym('DEPL') },
  sec: { displayName: 'Security', acronym: acronym('SECU') },
  docs: { displayName: 'Docs', acronym: acronym('DOCS') },
  campaign: { displayName: 'Campaign', acronym: acronym('CMGN') },
  rest: { displayName: 'REST', acronym: acronym('REST') }
} as const

type StorySubsystem = keyof typeof SUBSYSTEM_DEFAULTS
type StoryDiscipline = keyof typeof DISCIPLINE_DEFAULTS

const createTask = (id: number, subsystem: StorySubsystem, discipline: StoryDiscipline): Task => {
  const subsystemDefaults = SUBSYSTEM_DEFAULTS[subsystem]
  const disc = DISCIPLINE_DEFAULTS[discipline]
  return {
    id,
    subsystem,
    discipline,
    title: 'Example task',
    status: 'pending',
    tags: [],
    dependsOn: [],
    acceptanceCriteria: [],
    contextFiles: [],
    outputArtifacts: [],
    subsystemDisplayName: subsystemDefaults.displayName,
    signals: [],
    subsystemAcronym: subsystemDefaults.acronym,
    disciplineDisplayName: disc.displayName,
    disciplineAcronym: disc.acronym,
    disciplineIcon: disc.icon,
    disciplineColor: disc.color
  }
}

export const Frontend: Story = {
  args: {
    task: createTask(1, 'ui', 'frontend'),
    variant: 'default'
  }
}

export const Backend: Story = {
  args: {
    task: createTask(42, 'api', 'backend'),
    variant: 'default'
  }
}

export const Database: Story = {
  args: {
    task: createTask(3, 'data', 'database'),
    variant: 'default'
  }
}

export const Testing: Story = {
  args: {
    task: createTask(15, 'tests', 'testing'),
    variant: 'default'
  }
}

export const BadgeVariant: Story = {
  args: {
    task: createTask(1, 'ui', 'frontend'),
    variant: 'badge'
  }
}

export const HighId: Story = {
  args: {
    task: createTask(999, 'authentication', 'backend'),
    variant: 'default'
  }
}

export const VeryHighId: Story = {
  args: {
    task: createTask(1000, 'user-profile', 'frontend'),
    variant: 'default'
  }
}

export const MassiveId: Story = {
  args: {
    task: createTask(12345, 'payments', 'backend'),
    variant: 'default'
  }
}

export const IdProgression: Story = {
  args: {
    task: createTask(1, 'authentication', 'backend')
  },
  render: () => (
    <div className="flex flex-col gap-4">
      <div className="flex flex-col gap-1">
        <span className="text-xs text-muted-foreground">ID #001 (1-999 with # prefix)</span>
        <TaskIdDisplay task={createTask(1, 'authentication', 'backend')} />
      </div>
      <div className="flex flex-col gap-1">
        <span className="text-xs text-muted-foreground">ID #042</span>
        <TaskIdDisplay task={createTask(42, 'user-profile', 'frontend')} />
      </div>
      <div className="flex flex-col gap-1">
        <span className="text-xs text-muted-foreground">ID #999 (max with #)</span>
        <TaskIdDisplay task={createTask(999, 'payments', 'backend')} />
      </div>
      <div className="flex flex-col gap-1">
        <span className="text-xs text-muted-foreground">ID 1000 (no # for 1000+)</span>
        <TaskIdDisplay task={createTask(1000, 'search', 'backend')} />
      </div>
      <div className="flex flex-col gap-1">
        <span className="text-xs text-muted-foreground">ID 12345</span>
        <TaskIdDisplay task={createTask(12345, 'notifications', 'backend')} />
      </div>
    </div>
  )
}

export const AllDisciplines: Story = {
  args: {
    task: createTask(1, 'ui', 'frontend')
  },
  render: () => (
    <div className="flex flex-col gap-4">
      <TaskIdDisplay task={createTask(1, 'ui', 'frontend')} />
      <TaskIdDisplay task={createTask(2, 'api', 'backend')} />
      <TaskIdDisplay task={createTask(3, 'data', 'database')} />
      <TaskIdDisplay task={createTask(4, 'tests', 'testing')} />
      <TaskIdDisplay task={createTask(5, 'deploy', 'infra')} />
      <TaskIdDisplay task={createTask(6, 'sec', 'security')} />
      <TaskIdDisplay task={createTask(7, 'docs', 'docs')} />
      <TaskIdDisplay task={createTask(8, 'ui', 'design')} />
      <TaskIdDisplay task={createTask(9, 'campaign', 'wiring')} />
      <TaskIdDisplay task={createTask(10, 'rest', 'api')} />
    </div>
  )
}
