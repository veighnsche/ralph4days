import { QueryClient } from '@tanstack/react-query'
import { describe, expect, it } from 'vitest'
import {
  removeListItemFromArray,
  removeListItemFromQueryCache,
  replaceListItemInArray,
  replaceListItemInQueryCache
} from './listCache'
import { buildInvokeQueryKey } from './useInvoke'

interface Item {
  id: number
  name: string
}

describe('listCache', () => {
  it('replaces one entity in an array by key', () => {
    const current: Item[] = [
      { id: 1, name: 'one' },
      { id: 2, name: 'two' }
    ]

    const next = replaceListItemInArray({
      items: current,
      item: { id: 2, name: 'two-updated' },
      getKey: item => item.id,
      entityLabel: 'Item'
    })

    expect(next).toEqual([
      { id: 1, name: 'one' },
      { id: 2, name: 'two-updated' }
    ])
    expect(next[0]).toBe(current[0])
  })

  it('removes one entity from an array by key', () => {
    const current: Item[] = [
      { id: 1, name: 'one' },
      { id: 2, name: 'two' },
      { id: 3, name: 'three' }
    ]

    expect(removeListItemFromArray(current, 2, item => item.id, 'Item')).toEqual([
      { id: 1, name: 'one' },
      { id: 3, name: 'three' }
    ])
  })

  it('fails loudly when replacing an item that is missing', () => {
    expect(() =>
      replaceListItemInArray({
        items: [{ id: 1, name: 'one' }],
        item: { id: 999, name: 'missing' },
        getKey: item => item.id,
        entityLabel: 'Item'
      })
    ).toThrowError('[list-cache] Item 999 missing from cache')
  })

  it('fails loudly when cache is missing in query cache patch', () => {
    const queryClient = new QueryClient()

    expect(() =>
      replaceListItemInQueryCache({
        queryClient,
        queryDomain: 'workspace',
        command: 'get_items',
        item: { id: 1, name: 'one' },
        getKey: item => item.id,
        entityLabel: 'Item'
      })
    ).toThrowError('[list-cache] get_items cache is missing for workspace domain')
  })

  it('patches and removes list items in query cache', () => {
    const queryClient = new QueryClient()
    const queryKey = buildInvokeQueryKey('get_items', undefined, 'workspace')
    queryClient.setQueryData<Item[]>(queryKey, [
      { id: 1, name: 'one' },
      { id: 2, name: 'two' }
    ])

    replaceListItemInQueryCache({
      queryClient,
      queryDomain: 'workspace',
      command: 'get_items',
      item: { id: 2, name: 'two-updated' },
      getKey: item => item.id,
      entityLabel: 'Item'
    })

    removeListItemFromQueryCache<Item, number>({
      queryClient,
      queryDomain: 'workspace',
      command: 'get_items',
      key: 1,
      getKey: item => item.id,
      entityLabel: 'Item'
    })

    expect(queryClient.getQueryData<Item[]>(queryKey)).toEqual([{ id: 2, name: 'two-updated' }])
  })
})
