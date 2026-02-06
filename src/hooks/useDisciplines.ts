import type { LucideIcon } from "lucide-react";
import { resolveIcon } from "@/lib/iconRegistry";
import { useInvoke } from "./useInvoke";

/** Discipline config as returned by the backend (now camelCase) */
interface DisciplineConfigRaw {
  name: string;
  displayName: string;
  icon: string;
  color: string;
  acronym: string;
}

/** Resolved discipline config with Lucide icon component */
export interface DisciplineConfig {
  name: string;
  displayName: string;
  acronym: string;
  icon: LucideIcon;
  color: string;
  bgColor: string;
}

function resolveDisciplines(raw: DisciplineConfigRaw[]): DisciplineConfig[] {
  return raw.map((d) => ({
    name: d.name,
    displayName: d.displayName,
    acronym: d.acronym,
    icon: resolveIcon(d.icon),
    color: d.color,
    bgColor: `color-mix(in oklch, ${d.color} 15%, transparent)`,
  }));
}

/** Fetch discipline configs from the backend, resolve icons, and provide a lookup map */
export function useDisciplines() {
  const { data, error } = useInvoke<DisciplineConfigRaw[], DisciplineConfig[]>("get_disciplines_config", undefined, {
    staleTime: 5 * 60 * 1000,
    select: resolveDisciplines,
  });

  const disciplines = data ?? [];
  const configMap: Record<string, DisciplineConfig> = {};
  for (const d of disciplines) {
    configMap[d.name] = d;
  }

  return {
    disciplines,
    configMap,
    error: error ? String(error) : null,
  };
}
