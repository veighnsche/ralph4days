import { useMemo } from 'react'
import type { FeatureData } from '@/types/generated'
import { useInvoke } from './useInvoke'

export interface FeatureConfig {
  name: string
  displayName: string
  acronym: string
}

export function useFeatures() {
  const { data, error } = useInvoke<FeatureData[]>('get_features', undefined, {
    staleTime: 5 * 60 * 1000
  })

  const features = useMemo(
    () => data?.map(f => ({ name: f.name, displayName: f.displayName, acronym: f.acronym })) ?? [],
    [data]
  )

  return {
    features,
    error: error ? String(error) : null
  }
}
