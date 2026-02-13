import type { QueryClient } from '@tanstack/react-query'
import { type InvokeQueryDomain, removeListItemFromQueryCache, replaceListItemInQueryCache } from '@/hooks/api'
import type { DisciplineConfig as DisciplineConfigWire } from '@/types/generated'

export function patchDisciplineInCache(
  queryClient: QueryClient,
  discipline: DisciplineConfigWire,
  queryDomain: InvokeQueryDomain
): void {
  replaceListItemInQueryCache({
    queryClient,
    queryDomain,
    command: 'get_disciplines_config',
    item: discipline,
    getKey: item => item.name,
    entityLabel: 'Discipline'
  })
}

export function removeDisciplineFromCache(
  queryClient: QueryClient,
  name: string,
  queryDomain: InvokeQueryDomain
): void {
  removeListItemFromQueryCache<DisciplineConfigWire, string>({
    queryClient,
    queryDomain,
    command: 'get_disciplines_config',
    key: name,
    getKey: item => item.name,
    entityLabel: 'Discipline'
  })
}
