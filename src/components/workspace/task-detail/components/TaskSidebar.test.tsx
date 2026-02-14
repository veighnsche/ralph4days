import { render, screen } from '@testing-library/react'
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
        sessionId: 'session-1',
        signalVerb: 'ask'
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
  const approveMutate = vi.fn()
  const updateMutate = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
    useInvokeMutationMock.mockImplementation((command: string) => {
      if (command === 'tasks_set_status') {
        return {
          mutate: approveMutate,
          isPending: false,
          error: null,
          reset: vi.fn()
        }
      }
      if (command === 'tasks_update') {
        return {
          mutate: updateMutate,
          isPending: false,
          error: null,
          reset: vi.fn()
        }
      }
      throw new Error(`Unexpected command ${command}`)
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

    expect(separators.length).toBe(2)
    expect(hasAdjacentSeparators).toBe(false)
  })

  it('does not render the priority button group in the sidebar', () => {
    render(<TaskSidebar task={createTask({ priority: 'medium' })} inferredStatus="ready" />)

    expect(screen.queryByRole('button', { name: 'Low' })).toBeNull()
    expect(screen.queryByRole('button', { name: 'Medium' })).toBeNull()
    expect(screen.queryByRole('button', { name: 'High' })).toBeNull()
  })

  it('uses workspace query domain for task mutations', () => {
    render(<TaskSidebar task={createTask({ priority: 'medium' })} inferredStatus="ready" />)

    expect(useInvokeMutationMock).toHaveBeenNthCalledWith(
      1,
      'tasks_set_status',
      expect.objectContaining({ queryDomain: 'workspace' })
    )
    expect(useInvokeMutationMock).toHaveBeenNthCalledWith(
      2,
      'tasks_update',
      expect.objectContaining({ queryDomain: 'workspace' })
    )
  })
})
