import type { QueryClient } from '@tanstack/react-query'
import { type InvokeQueryDomain, replaceListItemInQueryCache } from '@/hooks/api'
import type { SubsystemData } from '@/types/generated'

export function patchSubsystemInCache(
  queryClient: QueryClient,
  subsystem: SubsystemData,
  queryDomain: InvokeQueryDomain
): void {
  replaceListItemInQueryCache({
    queryClient,
    queryDomain,
    command: 'subsystems_list',
    item: subsystem,
    getKey: item => item.name,
    entityLabel: 'Subsystem'
  })
}
