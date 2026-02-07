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

  const features = data ?? []
  const configMap = new Map<string, FeatureConfig>()
  for (const f of features) {
    configMap.set(f.name, f)
  }

  return {
    features,
    configMap,
    error: error ? String(error) : null
  }
}
