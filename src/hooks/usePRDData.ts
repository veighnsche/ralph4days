import { invoke } from "@tauri-apps/api/core";
import yaml from "js-yaml";
import { useCallback, useEffect, useState } from "react";
import type { PRDData } from "@/types/prd";

export function usePRDData() {
  const [prdData, setPrdData] = useState<PRDData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadPRD = useCallback(() => {
    if (typeof window !== "undefined" && "__TAURI__" in window) {
      setLoading(true);
      invoke<string>("get_prd_content")
        .then((content) => {
          try {
            const parsed = yaml.load(content) as PRDData;
            setPrdData(parsed);
            setError(null);
          } catch (e) {
            setError(`Failed to parse YAML: ${e}`);
          }
          setLoading(false);
        })
        .catch((err) => {
          setError(err);
          setLoading(false);
        });
    }
  }, []);

  useEffect(() => {
    loadPRD();
  }, [loadPRD]);

  return { prdData, loading, error, refresh: loadPRD };
}
