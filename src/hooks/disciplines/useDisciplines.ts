import type { LucideIcon } from 'lucide-react'
import { type InvokeQueryDomain, useInvoke } from '@/hooks/api'
import { resolveIcon } from '@/lib/iconRegistry'
import { type Acronym, toAcronym } from '@/types/acronym'
import type { DisciplineConfig as DisciplineConfigWire, DisciplineCropsData } from '@/types/generated'

export interface DisciplineConfig {
  id: number
  name: string
  displayName: string
  acronym: Acronym
  icon: LucideIcon
  color: string
  bgColor: string
  agent?: string
  model?: string
  effort?: string
  thinking?: boolean
  stackId?: number
  imagePath?: string
  crops?: DisciplineCropsData
}

function resolveDisciplines(raw: DisciplineConfigWire[]): DisciplineConfig[] {
  return raw.map(d => ({
    id: d.id,
    name: d.name,
    displayName: d.displayName,
    acronym: toAcronym(d.acronym),
    icon: resolveIcon(d.icon),
    color: d.color,
    bgColor: `color-mix(in oklch, ${d.color} 15%, transparent)`,
    agent: d.agent,
    model: d.model,
    effort: d.effort,
    thinking: d.thinking,
    stackId: d.stackId,
    imagePath: d.imagePath,
    crops: d.crops
  }))
}

const EMPTY_DISCIPLINES: DisciplineConfig[] = []

export function useDisciplines(queryDomain: InvokeQueryDomain = 'app') {
  const { data, error, isLoading } = useInvoke<DisciplineConfigWire[], DisciplineConfig[]>(
    'disciplines_list',
    undefined,
    {
      queryDomain,
      staleTime: 5 * 60 * 1000,
      select: resolveDisciplines
    }
  )

  return {
    disciplines: data ?? EMPTY_DISCIPLINES,
    error: error ? String(error) : null,
    isLoading
  }
}
