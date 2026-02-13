import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, renderHook, waitFor } from '@testing-library/react'
import { createElement, type ReactNode } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { buildInvokeQueryKey } from './useInvoke'
import { useInvokeMutation } from './useInvokeMutation'

interface Item {
  id: number
  name: string
}

const mockInvoke = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args)
}))

function createWrapper(seedItems: Item[]) {
  const queryClient = new QueryClient({
    defaultOptions: { mutations: { retry: false } }
  })
  queryClient.setQueryData(buildInvokeQueryKey('get_items', undefined, 'workspace'), seedItems)

  return {
    queryClient,
    wrapper: ({ children }: { children: ReactNode }) =>
      createElement(QueryClientProvider, { client: queryClient }, children)
  }
}

describe('useInvokeMutation optimistic updates', () => {
  beforeEach(() => {
    mockInvoke.mockReset()
  })

  it('applies optimistic update immediately and finalizes with updateCache payload', async () => {
    let resolveInvoke: ((value: Item) => void) | null = null
    mockInvoke.mockImplementation(
      () =>
        new Promise<Item>(resolve => {
          resolveInvoke = resolve
        })
    )

    const { wrapper, queryClient } = createWrapper([
      { id: 1, name: 'original-1' },
      { id: 2, name: 'original-2' }
    ])

    const { result } = renderHook(
      () =>
        useInvokeMutation<{ nextItem: Item }, Item>('update_item', {
          queryDomain: 'workspace',
          optimisticUpdate: ({ queryClient: client, variables, queryDomain }) => {
            const queryKey = buildInvokeQueryKey('get_items', undefined, queryDomain)
            const previous = client.getQueryData<Item[]>(queryKey)
            if (!previous) {
              throw new Error('[test] missing get_items cache')
            }
            client.setQueryData<Item[]>(queryKey, [previous[0], variables.nextItem])
            return () => {
              client.setQueryData(queryKey, previous)
            }
          },
          updateCache: ({ queryClient: client, data, queryDomain }) => {
            client.setQueryData<Item[]>(buildInvokeQueryKey('get_items', undefined, queryDomain), current => {
              if (!current) {
                throw new Error('[test] missing get_items cache')
              }
              return [current[0], data]
            })
          }
        }),
      { wrapper }
    )

    act(() => {
      result.current.mutate({
        nextItem: { id: 2, name: 'optimistic' }
      })
    })

    await waitFor(() =>
      expect(queryClient.getQueryData<Item[]>(buildInvokeQueryKey('get_items', undefined, 'workspace'))).toEqual([
        { id: 1, name: 'original-1' },
        { id: 2, name: 'optimistic' }
      ])
    )

    if (resolveInvoke === null) {
      throw new Error('[test] invoke resolver was not captured')
    }
    const invokeResolve: (value: Item) => void = resolveInvoke

    act(() => {
      invokeResolve({ id: 2, name: 'server-final' })
    })

    await waitFor(() =>
      expect(queryClient.getQueryData<Item[]>(buildInvokeQueryKey('get_items', undefined, 'workspace'))).toEqual([
        { id: 1, name: 'original-1' },
        { id: 2, name: 'server-final' }
      ])
    )
  })

  it('rolls back optimistic update when mutation fails', async () => {
    let rejectInvoke: ((reason?: unknown) => void) | null = null
    mockInvoke.mockImplementation(
      () =>
        new Promise<Item>((_, reject) => {
          rejectInvoke = reject
        })
    )

    const originalItems: Item[] = [
      { id: 1, name: 'original-1' },
      { id: 2, name: 'original-2' }
    ]
    const { wrapper, queryClient } = createWrapper(originalItems)

    const { result } = renderHook(
      () =>
        useInvokeMutation<{ nextItem: Item }, Item>('update_item', {
          queryDomain: 'workspace',
          optimisticUpdate: ({ queryClient: client, variables, queryDomain }) => {
            const queryKey = buildInvokeQueryKey('get_items', undefined, queryDomain)
            const previous = client.getQueryData<Item[]>(queryKey)
            if (!previous) {
              throw new Error('[test] missing get_items cache')
            }
            client.setQueryData<Item[]>(queryKey, [previous[0], variables.nextItem])
            return () => {
              client.setQueryData(queryKey, previous)
            }
          }
        }),
      { wrapper }
    )

    act(() => {
      result.current.mutate({
        nextItem: { id: 2, name: 'optimistic' }
      })
    })

    await waitFor(() =>
      expect(queryClient.getQueryData<Item[]>(buildInvokeQueryKey('get_items', undefined, 'workspace'))).toEqual([
        { id: 1, name: 'original-1' },
        { id: 2, name: 'optimistic' }
      ])
    )

    if (rejectInvoke === null) {
      throw new Error('[test] invoke rejecter was not captured')
    }
    const invokeReject: (reason?: unknown) => void = rejectInvoke

    act(() => {
      invokeReject(new Error('mutation failed'))
    })

    await waitFor(() => expect(result.current.isError).toBe(true))
    expect(queryClient.getQueryData<Item[]>(buildInvokeQueryKey('get_items', undefined, 'workspace'))).toEqual(
      originalItems
    )
  })
})
