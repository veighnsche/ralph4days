import { render } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { Task } from '@/types/generated'
import { TaskSidebar } from './TaskSidebar'

const { useInvokeMutationMock, useResolvedTaskLaunchMock } = vi.hoisted(() => ({
  useInvokeMutationMock: vi.fn(),
  useResolvedTaskLaunchMock: vi.fn()
}))

vi.mock('@/hooks/api', () => ({
  useInvokeMutation: (...args: unknown[]) => useInvokeMutationMock(...args)
}))

vi.mock('../hooks/useResolvedTaskLaunch', () => ({
  useResolvedTaskLaunch: (...args: unknown[]) => useResolvedTaskLaunchMock(...args)
}))

vi.mock('./DisciplineSelect', () => ({
  DisciplineSelect: ({ value }: { value: string }) => <span data-testid="discipline-select">{value}</span>
}))

function createTask(overrides: Partial<Task> = {}): Task {
  return {
    id: 101,
    subsystem: 'core-platform',
    discipline: 'frontend',
    title: 'Render task details',
    status: 'pending',
    tags: ['ui', 'sidebar'],
    dependsOn: [],
    created: '2026-02-12T15:32:00.000Z',
    updated: '2026-02-12T16:18:00.000Z',
    acceptanceCriteria: [],
    contextFiles: [],
    outputArtifacts: [],
    signals: [
      {
        id: 1,
        author: 'agent',
        body: 'Need API contract clarification',
        session_id: 'session-1',
        signal_verb: 'ask'
      }
    ],
    subsystemDisplayName: 'Core Platform',
    subsystemAcronym: 'CP',
    disciplineDisplayName: 'Frontend',
    disciplineAcronym: 'FE',
    disciplineIcon: 'bot',
    disciplineColor: '#43a047',
    ...overrides
  }
}

describe('TaskSidebar', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    useInvokeMutationMock.mockReturnValue({
      mutate: vi.fn(),
      isPending: false,
      error: null,
      reset: vi.fn()
    })
    useResolvedTaskLaunchMock.mockReturnValue({
      resolvedAgent: 'codex',
      resolvedModel: 'gpt-5.3-codex',
      resolvedEffort: 'medium',
      resolvedThinking: true,
      resolvedModelSupportsEffort: true,
      agentSource: 'task',
      modelSource: 'task',
      effortSource: 'task',
      thinkingSource: 'task'
    })
  })

  it('does not render adjacent separators when signals and tags are present', () => {
    const { container } = render(<TaskSidebar task={createTask()} inferredStatus="ready" />)
    const separators = Array.from(container.querySelectorAll('[data-slot="separator"].my-2'))
    const hasAdjacentSeparators = separators.some(
      separator => separator.nextElementSibling?.getAttribute('data-slot') === 'separator'
    )

    expect(separators.length).toBe(3)
    expect(hasAdjacentSeparators).toBe(false)
  })
})
