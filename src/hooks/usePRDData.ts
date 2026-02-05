import yaml from "js-yaml";
import { useCallback, useEffect, useState } from "react";
import { isTauriEnvironment, universalInvoke } from "@/services/mockBackend";
import type { PRDData } from "@/types/prd";

export function usePRDData() {
  const [prdData, setPrdData] = useState<PRDData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [usingMockData, setUsingMockData] = useState(false);

  const loadPRD = useCallback(() => {
    setLoading(true);
    setUsingMockData(!isTauriEnvironment());

    universalInvoke<string>("get_prd_content")
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
        setError(
          typeof err === "string" ? err : err.message || "Failed to load PRD data. Make sure the backend is running."
        );
        setLoading(false);
      });
  }, []);

  useEffect(() => {
    loadPRD();
  }, [loadPRD]);

  return { prdData, loading, error, refresh: loadPRD, usingMockData };
}
