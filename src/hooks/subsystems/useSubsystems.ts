import { useMemo } from 'react'
import { useInvoke } from '@/hooks/api'
import type { SubsystemData } from '@/types/generated'

export interface SubsystemConfig {
  name: string
  displayName: string
  acronym: string
}

export function useSubsystems() {
  const { data, error } = useInvoke<SubsystemData[]>('get_subsystems', undefined, {
    staleTime: 5 * 60 * 1000
  })

  const subsystems = useMemo(
    () => data?.map(f => ({ name: f.name, displayName: f.displayName, acronym: f.acronym })) ?? [],
    [data]
  )

  return {
    subsystems,
    error: error ? String(error) : null
  }
}
