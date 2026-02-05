import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import yaml from "js-yaml";
import type { PRDData } from "@/types/prd";

export function usePRDData() {
  const [prdData, setPrdData] = useState<PRDData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (typeof window !== "undefined" && "__TAURI__" in window) {
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

  return { prdData, loading, error };
}
