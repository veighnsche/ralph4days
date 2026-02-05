import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

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

/** Fetch feature configs from the backend and provide a lookup map */
export function useFeatures() {
  const [features, setFeatures] = useState<FeatureConfig[]>([]);
  const [configMap, setConfigMap] = useState<Map<string, FeatureConfig>>(new Map());

  useEffect(() => {
    invoke<FeatureConfigRaw[]>("get_features_config")
      .then((raw) => {
        const resolved = raw.map((f) => ({
          name: f.name,
          displayName: f.display_name,
          acronym: f.acronym,
        }));
        setFeatures(resolved);

        const map = new Map<string, FeatureConfig>();
        for (const f of resolved) {
          map.set(f.name, f);
        }
        setConfigMap(map);
      })
      .catch((err) => {
        console.error("Failed to load feature config:", err);
      });
  }, []);

  return { features, configMap };
}
