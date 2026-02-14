import { useMemo } from 'react'
import { useInvoke } from '@/hooks/api'
import { type Acronym, toAcronym } from '@/types/acronym'
import type { SubsystemData } from '@/types/generated'

export interface SubsystemConfig {
  name: string
  displayName: string
  acronym: Acronym
}

export function useSubsystems() {
  const { data, error } = useInvoke<SubsystemData[]>('subsystems_list', undefined, {
    staleTime: 5 * 60 * 1000
  })

  const subsystems = useMemo(
    () => data?.map(f => ({ name: f.name, displayName: f.displayName, acronym: toAcronym(f.acronym) })) ?? [],
    [data]
  )

  return {
    subsystems,
    error: error ? String(error) : null
  }
}
