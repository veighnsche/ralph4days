import type { Meta, StoryObj } from '@storybook/react'
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
const DISCIPLINE_DEFAULTS: Record<string, { displayName: string; acronym: string; icon: string; color: string }> = {
  frontend: { displayName: 'Frontend', acronym: 'FRNT', icon: 'code', color: '#3B82F6' },
  backend: { displayName: 'Backend', acronym: 'BKND', icon: 'server', color: '#8B5CF6' },
  database: { displayName: 'Database', acronym: 'DATA', icon: 'database', color: '#F59E0B' },
  testing: { displayName: 'Testing', acronym: 'TEST', icon: 'flask-conical', color: '#10B981' },
  infra: { displayName: 'Infrastructure', acronym: 'INFR', icon: 'cloud', color: '#6366F1' },
  security: { displayName: 'Security', acronym: 'SECU', icon: 'shield', color: '#EF4444' },
  docs: { displayName: 'Documentation', acronym: 'DOCS', icon: 'book-open', color: '#64748B' },
  design: { displayName: 'Design', acronym: 'DSGN', icon: 'palette', color: '#EC4899' },
  wiring: { displayName: 'Wiring', acronym: 'WIRE', icon: 'cable', color: '#A855F7' },
  api: { displayName: 'API', acronym: 'API', icon: 'plug', color: '#14B8A6' }
}

function subsystemDisplayName(subsystem: string): string {
  return subsystem
    .split('-')
    .map(w => w.charAt(0).toUpperCase() + w.slice(1))
    .join(' ')
}

function subsystemAcronym(subsystem: string): string {
  return subsystem.replace(/-/g, '').slice(0, 4).toUpperCase()
}

const createTask = (id: number, subsystem: string, discipline: string): Task => {
  const disc = DISCIPLINE_DEFAULTS[discipline] ?? {
    displayName: discipline.charAt(0).toUpperCase() + discipline.slice(1),
    acronym: discipline.slice(0, 4).toUpperCase(),
    icon: 'circle',
    color: '#6B7280'
  }
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
    subsystemDisplayName: subsystemDisplayName(subsystem),
    signals: [],
    subsystemAcronym: subsystemAcronym(subsystem),
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
