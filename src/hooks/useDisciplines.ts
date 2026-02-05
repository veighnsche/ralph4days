import { invoke } from "@tauri-apps/api/core";
import type { LucideIcon } from "lucide-react";
import { useEffect, useState } from "react";
import { resolveIcon } from "@/lib/iconRegistry";

/** Discipline config as returned by the backend */
interface DisciplineConfigRaw {
  name: string;
  display_name: string;
  icon: string;
  color: string;
}

/** Resolved discipline config with Lucide icon component */
export interface DisciplineConfig {
  name: string;
  displayName: string;
  icon: LucideIcon;
  color: string;
  bgColor: string;
}

/** Fetch discipline configs from the backend, resolve icons, and provide a lookup map */
export function useDisciplines() {
  const [disciplines, setDisciplines] = useState<DisciplineConfig[]>([]);
  const [configMap, setConfigMap] = useState<Record<string, DisciplineConfig>>({});

  useEffect(() => {
    invoke<DisciplineConfigRaw[]>("get_disciplines_config")
      .then((raw) => {
        const resolved = raw.map((d) => ({
          name: d.name,
          displayName: d.display_name,
          icon: resolveIcon(d.icon),
          color: d.color,
          bgColor: `color-mix(in oklch, ${d.color} 15%, transparent)`,
        }));
        setDisciplines(resolved);

        const map: Record<string, DisciplineConfig> = {};
        for (const d of resolved) {
          map[d.name] = d;
        }
        setConfigMap(map);
      })
      .catch((err) => {
        console.error("Failed to load discipline config:", err);
      });
  }, []);

  return { disciplines, configMap };
}
