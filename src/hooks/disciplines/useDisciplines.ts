import type { LucideIcon } from 'lucide-react'
import { useInvoke } from '@/hooks/api'
import { resolveIcon } from '@/lib/iconRegistry'
import type { DisciplineConfig as DisciplineConfigWire, DisciplineCropsData } from '@/types/generated'

export interface DisciplineConfig {
  name: string
  displayName: string
  acronym: string
  icon: LucideIcon
  color: string
  bgColor: string
  stackId?: number
  imagePath?: string
  crops?: DisciplineCropsData
}

function resolveDisciplines(raw: DisciplineConfigWire[]): DisciplineConfig[] {
  return raw.map(d => ({
    name: d.name,
    displayName: d.displayName,
    acronym: d.acronym,
    icon: resolveIcon(d.icon),
    color: d.color,
    bgColor: `color-mix(in oklch, ${d.color} 15%, transparent)`,
    stackId: d.stackId,
    imagePath: d.imagePath,
    crops: d.crops
  }))
}

export function useDisciplines() {
  const { data, error } = useInvoke<DisciplineConfigWire[], DisciplineConfig[]>('get_disciplines_config', undefined, {
    staleTime: 5 * 60 * 1000,
    select: resolveDisciplines
  })

  return {
    disciplines: data ?? [],
    error: error ? String(error) : null
  }
}

export function useDisciplinesRaw() {
  const { data, error } = useInvoke<DisciplineConfigWire[]>('get_disciplines_config', undefined, {
    staleTime: 5 * 60 * 1000
  })

  return {
    disciplines: data ?? [],
    error: error ? String(error) : null
  }
}
