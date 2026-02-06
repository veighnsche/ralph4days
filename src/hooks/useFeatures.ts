import { useInvoke } from "./useInvoke";

/** Feature config as returned by the backend */
interface FeatureConfigRaw {
  name: string;
  display_name: string;
  acronym: string;
}

/** Feature config */
export interface FeatureConfig {
  name: string;
  displayName: string;
  acronym: string;
}

function resolveFeatures(raw: FeatureConfigRaw[]): FeatureConfig[] {
  return raw.map((f) => ({
    name: f.name,
    displayName: f.display_name,
    acronym: f.acronym,
  }));
}

/** Fetch feature configs from the backend and provide a lookup map */
export function useFeatures() {
  const { data, error } = useInvoke<FeatureConfigRaw[], FeatureConfig[]>("get_features_config", undefined, {
    staleTime: 5 * 60 * 1000,
    select: resolveFeatures,
  });

  const features = data ?? [];
  const configMap = new Map<string, FeatureConfig>();
  for (const f of features) {
    configMap.set(f.name, f);
  }

  return {
    features,
    configMap,
    error: error ? String(error) : null,
  };
}
