import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, renderHook, waitFor } from '@testing-library/react'
import { createElement, type ReactNode } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { buildInvokeQueryKey } from '@/hooks/api'
import type { SubsystemData } from '@/types/generated'
import { useSubsystemCommentMutations } from './useSubsystemCommentMutations'

const mockInvoke = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args)
}))

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { mutations: { retry: false } }
  })
  const seededSubsystem: SubsystemData = {
    id: 1,
    name: 'auth',
    displayName: 'Auth',
    acronym: 'AUTH',
    status: 'active',
    comments: []
  }
  queryClient.setQueryData(buildInvokeQueryKey('get_subsystems', undefined, 'app'), [seededSubsystem])
  return ({ children }: { children: ReactNode }) =>
    createElement(QueryClientProvider, { client: queryClient }, children)
}

describe('useSubsystemCommentMutations', () => {
  beforeEach(() => {
    mockInvoke.mockReset()
    mockInvoke.mockImplementation((command: string) => {
      if (
        command === 'add_subsystem_comment' ||
        command === 'update_subsystem_comment' ||
        command === 'delete_subsystem_comment'
      ) {
        return Promise.resolve({
          id: 1,
          name: 'auth',
          displayName: 'Auth',
          acronym: 'AUTH',
          status: 'active',
          comments: []
        } satisfies SubsystemData)
      }
      return Promise.resolve(undefined)
    })
  })

  it('addComment.mutate calls invoke with correct params', async () => {
    const { result } = renderHook(() => useSubsystemCommentMutations('auth'), {
      wrapper: createWrapper()
    })

    act(() => {
      result.current.addComment.mutate({
        subsystemName: 'auth',
        category: 'gotcha',
        body: 'Watch out for XSS',
        reason: 'OWASP top 10'
      })
    })

    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith('add_subsystem_comment', {
        subsystemName: 'auth',
        category: 'gotcha',
        body: 'Watch out for XSS',
        reason: 'OWASP top 10'
      })
    )
  })

  it('startEdit / cancelEdit state management', () => {
    const { result } = renderHook(() => useSubsystemCommentMutations('auth'), {
      wrapper: createWrapper()
    })

    expect(result.current.editingId).toBeNull()
    expect(result.current.editBody).toBe('')
    expect(result.current.editSummary).toBe('')
    expect(result.current.editReason).toBe('')

    act(() => {
      result.current.startEdit(42, 'old body', 'old summary', 'old reason')
    })

    expect(result.current.editingId).toBe(42)
    expect(result.current.editBody).toBe('old body')
    expect(result.current.editSummary).toBe('old summary')
    expect(result.current.editReason).toBe('old reason')

    act(() => {
      result.current.cancelEdit()
    })

    expect(result.current.editingId).toBeNull()
    expect(result.current.editBody).toBe('')
    expect(result.current.editSummary).toBe('')
    expect(result.current.editReason).toBe('')
  })

  it('submitEdit calls invoke with trimmed values', async () => {
    const { result } = renderHook(() => useSubsystemCommentMutations('auth'), {
      wrapper: createWrapper()
    })

    act(() => {
      result.current.startEdit(7, '  trimmed body  ', '  trimmed summary  ', '  trimmed reason  ')
    })

    act(() => {
      result.current.submitEdit()
    })

    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith('update_subsystem_comment', {
        subsystemName: 'auth',
        commentId: 7,
        body: 'trimmed body',
        summary: 'trimmed summary',
        reason: 'trimmed reason'
      })
    )
  })

  it('submitEdit no-op when editingId is null', () => {
    const { result } = renderHook(() => useSubsystemCommentMutations('auth'), {
      wrapper: createWrapper()
    })

    act(() => {
      result.current.submitEdit()
    })

    expect(mockInvoke).not.toHaveBeenCalled()
  })

  it('deleteComment calls invoke with subsystemName and commentId', async () => {
    const { result } = renderHook(() => useSubsystemCommentMutations('auth'), {
      wrapper: createWrapper()
    })

    act(() => {
      result.current.deleteComment(99)
    })

    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith('delete_subsystem_comment', { subsystemName: 'auth', commentId: 99 })
    )
  })

  it('error aggregation and reset', async () => {
    mockInvoke.mockRejectedValueOnce('Add failed')

    const { result } = renderHook(() => useSubsystemCommentMutations('auth'), {
      wrapper: createWrapper()
    })

    act(() => {
      result.current.addComment.mutate({
        subsystemName: 'auth',
        category: 'gotcha',
        body: 'test'
      })
    })

    await waitFor(() => expect(result.current.error).toBeTruthy())

    act(() => {
      result.current.resetError()
    })

    await waitFor(() => expect(result.current.error).toBeNull())
  })
})
