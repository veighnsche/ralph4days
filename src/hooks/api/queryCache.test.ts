import { QueryClient } from '@tanstack/react-query'
import { describe, expect, it } from 'vitest'
import { replaceQueryDataInCache, replaceQueryDataInCacheOptimistically } from './queryCache'
import { buildInvokeQueryKey } from './useInvoke'

describe('queryCache helpers', () => {
  it('replaces cached query data for a parameterized invoke query', () => {
    const queryClient = new QueryClient()
    const queryKey = buildInvokeQueryKey('get_task', { id: 123 }, 'workspace')

    queryClient.setQueryData(queryKey, { id: 123, title: 'before' })

    replaceQueryDataInCache({
      queryClient,
      queryDomain: 'workspace',
      command: 'get_task',
      args: { id: 123 },
      data: { id: 123, title: 'after' }
    })

    expect(queryClient.getQueryData(queryKey)).toEqual({ id: 123, title: 'after' })
  })

  it('fails loudly when the target query cache is missing', () => {
    const queryClient = new QueryClient()

    expect(() =>
      replaceQueryDataInCache({
        queryClient,
        queryDomain: 'workspace',
        command: 'get_task',
        args: { id: 123 },
        data: { id: 123, title: 'after' }
      })
    ).toThrowError('[query-cache] get_task cache is missing for workspace domain')
  })

  it('optimistically patches and rolls back a cached invoke query', () => {
    const queryClient = new QueryClient()
    const queryKey = buildInvokeQueryKey('get_task', { id: 123 }, 'workspace')

    queryClient.setQueryData(queryKey, { id: 123, title: 'before' })

    const rollback = replaceQueryDataInCacheOptimistically({
      queryClient,
      queryDomain: 'workspace',
      command: 'get_task',
      args: { id: 123 },
      data: { id: 123, title: 'optimistic' }
    })

    expect(queryClient.getQueryData(queryKey)).toEqual({ id: 123, title: 'optimistic' })

    rollback()

    expect(queryClient.getQueryData(queryKey)).toEqual({ id: 123, title: 'before' })
  })
})
