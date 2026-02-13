import type { QueryClient } from '@tanstack/react-query'
import { buildInvokeQueryKey, type InvokeQueryDomain } from './useInvoke'

interface ListItemParams<TItem, TKey> {
  items: TItem[]
  item: TItem
  getKey: (item: TItem) => TKey
  entityLabel: string
}

interface ListCacheParams<TItem, TKey> {
  queryClient: QueryClient
  queryDomain: InvokeQueryDomain
  command: string
  item: TItem
  getKey: (item: TItem) => TKey
  entityLabel: string
}

interface ListCacheRemoveParams<TItem, TKey> {
  queryClient: QueryClient
  queryDomain: InvokeQueryDomain
  command: string
  key: TKey
  getKey: (item: TItem) => TKey
  entityLabel: string
}

function withCachedList<TItem>(
  queryClient: QueryClient,
  queryDomain: InvokeQueryDomain,
  command: string,
  update: (items: TItem[]) => TItem[]
): void {
  queryClient.setQueryData<TItem[]>(buildInvokeQueryKey(command, undefined, queryDomain), currentItems => {
    if (!currentItems) {
      throw new Error(`[list-cache] ${command} cache is missing for ${queryDomain} domain`)
    }
    return update(currentItems)
  })
}

export function replaceListItemInArray<TItem, TKey>({
  items,
  item,
  getKey,
  entityLabel
}: ListItemParams<TItem, TKey>): TItem[] {
  const key = getKey(item)
  const itemIndex = items.findIndex(current => getKey(current) === key)
  if (itemIndex === -1) {
    throw new Error(`[list-cache] ${entityLabel} ${String(key)} missing from cache`)
  }

  const nextItems = [...items]
  nextItems[itemIndex] = item
  return nextItems
}

export function removeListItemFromArray<TItem, TKey>(
  items: TItem[],
  key: TKey,
  getKey: (item: TItem) => TKey,
  entityLabel: string
): TItem[] {
  const itemIndex = items.findIndex(current => getKey(current) === key)
  if (itemIndex === -1) {
    throw new Error(`[list-cache] ${entityLabel} ${String(key)} missing from cache`)
  }

  const nextItems = [...items]
  nextItems.splice(itemIndex, 1)
  return nextItems
}

export function replaceListItemInQueryCache<TItem, TKey>({
  queryClient,
  queryDomain,
  command,
  item,
  getKey,
  entityLabel
}: ListCacheParams<TItem, TKey>): void {
  withCachedList<TItem>(queryClient, queryDomain, command, currentItems =>
    replaceListItemInArray({
      items: currentItems,
      item,
      getKey,
      entityLabel
    })
  )
}

export function removeListItemFromQueryCache<TItem, TKey>({
  queryClient,
  queryDomain,
  command,
  key,
  getKey,
  entityLabel
}: ListCacheRemoveParams<TItem, TKey>): void {
  withCachedList<TItem>(queryClient, queryDomain, command, currentItems =>
    removeListItemFromArray(currentItems, key, getKey, entityLabel)
  )
}
