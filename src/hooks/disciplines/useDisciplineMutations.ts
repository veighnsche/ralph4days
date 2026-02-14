import { type InvokeQueryDomain, useInvokeMutation } from '@/hooks/api'
import type { DisciplineFormData } from '@/lib/schemas'
import type { DisciplineConfig as DisciplineConfigWire } from '@/types/generated'
import { patchDisciplineInCache, removeDisciplineFromCache } from './disciplineCache'

interface CreateDisciplineParams {
  name: string
  displayName: string
  acronym: string
  icon: string
  color: string
  systemPrompt?: string
  agent?: 'claude' | 'codex'
  model?: string
  effort?: 'low' | 'medium' | 'high'
  thinking?: boolean
  skills?: string[]
  conventions?: string
  mcpServers?: Array<{
    name: string
    command: string
    args: string[]
    env: Record<string, string>
  }>
}

interface UpdateDisciplineParams {
  name: string
  displayName: string
  acronym: string
  icon: string
  color: string
  systemPrompt?: string
  agent?: 'claude' | 'codex'
  model?: string
  effort?: 'low' | 'medium' | 'high'
  thinking?: boolean
  skills?: string[]
  conventions?: string
  mcpServers?: Array<{
    name: string
    command: string
    args: string[]
    env: Record<string, string>
  }>
}

export function useDisciplineMutations(queryDomain: InvokeQueryDomain = 'app') {
  const createMutation = useInvokeMutation<CreateDisciplineParams, DisciplineConfigWire>('disciplines_create', {
    queryDomain,
    updateCache: ({ queryClient, data, queryDomain }) => patchDisciplineInCache(queryClient, data, queryDomain)
  })

  const updateMutation = useInvokeMutation<UpdateDisciplineParams, DisciplineConfigWire>('disciplines_update', {
    queryDomain,
    updateCache: ({ queryClient, data, queryDomain }) => patchDisciplineInCache(queryClient, data, queryDomain)
  })

  const deleteMutation = useInvokeMutation<{ name: string }, string>('disciplines_delete', {
    queryDomain,
    updateCache: ({ queryClient, data, queryDomain }) => removeDisciplineFromCache(queryClient, data, queryDomain)
  })

  const createDiscipline = (data: DisciplineFormData) => {
    return createMutation.mutateAsync({
      name: data.name,
      displayName: data.displayName,
      acronym: data.acronym,
      icon: data.icon,
      color: data.color,
      systemPrompt: data.systemPrompt,
      agent: data.agent,
      model: data.model?.trim() ? data.model.trim() : undefined,
      effort: data.effort,
      thinking: data.thinking,
      skills: data.skills || [],
      conventions: data.conventions,
      mcpServers: data.mcpServers || []
    })
  }

  const updateDiscipline = (data: DisciplineFormData) => {
    return updateMutation.mutateAsync({
      name: data.name,
      displayName: data.displayName,
      acronym: data.acronym,
      icon: data.icon,
      color: data.color,
      systemPrompt: data.systemPrompt,
      agent: data.agent,
      model: data.model?.trim() ? data.model.trim() : undefined,
      effort: data.effort,
      thinking: data.thinking,
      skills: data.skills || [],
      conventions: data.conventions,
      mcpServers: data.mcpServers || []
    })
  }

  const deleteDiscipline = (name: string) => {
    return deleteMutation.mutateAsync({ name })
  }

  return {
    createDiscipline,
    updateDiscipline,
    deleteDiscipline,
    isCreating: createMutation.isPending,
    isUpdating: updateMutation.isPending,
    isDeleting: deleteMutation.isPending,
    createError: createMutation.error,
    updateError: updateMutation.error,
    deleteError: deleteMutation.error
  }
}
