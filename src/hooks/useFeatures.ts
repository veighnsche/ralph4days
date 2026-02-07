import { useInvoke } from './useInvoke'

export interface FeatureConfig {
  name: string
  displayName: string
  acronym: string
}

export function useFeatures() {
  const { data, error } = useInvoke<FeatureConfig[]>('get_features_config', undefined, {
    staleTime: 5 * 60 * 1000
  })

  return {
    features: data ?? [],
    error: error ? String(error) : null
  }
}
