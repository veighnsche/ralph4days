import { act, fireEvent, render, screen } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { Task } from '@/types/generated'
import { TaskCardContent } from './TaskCardContent'

const { openTabAfterMock, useInvokeMock, useInvokeMutationMock, updateTaskMutateMock } = vi.hoisted(() => ({
  openTabAfterMock: vi.fn(),
  useInvokeMock: vi.fn(),
  useInvokeMutationMock: vi.fn(),
  updateTaskMutateMock: vi.fn()
}))

vi.mock('@/stores/useWorkspaceStore', () => ({
  useWorkspaceStore: (selector: (state: { openTabAfter: typeof openTabAfterMock }) => unknown) =>
    selector({ openTabAfter: openTabAfterMock })
}))

vi.mock('@/hooks/api', () => ({
  useInvoke: (...args: unknown[]) => useInvokeMock(...args),
  useInvokeMutation: (...args: unknown[]) => useInvokeMutationMock(...args)
}))

vi.mock('@/hooks/disciplines', () => ({
  useDisciplines: () => ({
    disciplines: [{ id: 7, name: 'frontend' }]
  })
}))

vi.mock('@/components/workspace/tabs', async importOriginal => {
  const actual = await importOriginal<typeof import('@/components/workspace/tabs')>()
  return {
    ...actual,
    useWorkspaceTabContext: () => ({ id: 'task-detail-current' })
  }
})

function createTask(overrides: Partial<Task> = {}): Task {
  return {
    id: 101,
    subsystem: 'core-platform',
    discipline: 'frontend',
    title: 'Render task details',
    status: 'pending',
    tags: [],
    dependsOn: [],
    acceptanceCriteria: [],
    contextFiles: [],
    outputArtifacts: [],
    signals: [],
    subsystemDisplayName: 'Core Platform',
    subsystemAcronym: 'CP',
    disciplineDisplayName: 'Frontend',
    disciplineAcronym: 'FE',
    disciplineIcon: 'code',
    disciplineColor: '#43a047',
    ...overrides
  }
}

describe('TaskCardContent', () => {
  beforeEach(() => {
    vi.useRealTimers()
    vi.clearAllMocks()
    useInvokeMutationMock.mockReturnValue({
      mutate: updateTaskMutateMock,
      isPending: false,
      error: null,
      reset: vi.fn()
    })
    useInvokeMock.mockReturnValue({
      data: [
        {
          id: 42,
          name: 'core-platform',
          displayName: 'Core Platform',
          acronym: 'CP',
          status: 'active',
          comments: []
        }
      ]
    })
  })

  it('single-click on criterion text toggles checkbox state', () => {
    vi.useFakeTimers()

    render(
      <TaskCardContent
        task={createTask({
          acceptanceCriteria: ['[ ] Confirm workspace update']
        })}
      />
    )

    fireEvent.click(screen.getByDisplayValue('Confirm workspace update'))
    act(() => {
      vi.advanceTimersByTime(200)
    })

    expect(updateTaskMutateMock).toHaveBeenCalledWith({
      params: expect.objectContaining({
        id: 101,
        acceptance_criteria: ['[x] Confirm workspace update']
      })
    })
  })

  it('double-click edits criterion text and saves without triggering checkbox toggle', () => {
    vi.useFakeTimers()

    render(
      <TaskCardContent
        task={createTask({
          acceptanceCriteria: ['[ ] Confirm workspace update']
        })}
      />
    )

    fireEvent.doubleClick(screen.getByDisplayValue('Confirm workspace update'))

    const editingInput = screen.getByDisplayValue('Confirm workspace update')
    fireEvent.change(editingInput, { target: { value: 'Updated criterion text' } })
    fireEvent.click(screen.getByRole('button', { name: 'Save acceptance criterion 1 edit' }))

    expect(updateTaskMutateMock).toHaveBeenCalledWith({
      params: expect.objectContaining({
        id: 101,
        acceptance_criteria: ['[ ] Updated criterion text']
      })
    })
    expect(updateTaskMutateMock).toHaveBeenCalledTimes(1)
  })

  it('renders criterion text as read-only input by default', () => {
    render(
      <TaskCardContent
        task={createTask({
          acceptanceCriteria: ['Cache updates must avoid full task list refetches']
        })}
      />
    )

    const criterionInput = screen.getByDisplayValue('Cache updates must avoid full task list refetches')
    expect(criterionInput).toHaveAttribute('readonly')
    expect(screen.queryByRole('button', { name: 'Save acceptance criterion 1 edit' })).not.toBeInTheDocument()
  })
})
