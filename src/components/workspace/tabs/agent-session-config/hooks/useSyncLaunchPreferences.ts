import { useEffect } from 'react'
import { useAgentSessionLaunchPreferences } from '@/hooks/preferences'
import { useAgentSessionConfigLaunchState } from './useAgentSessionConfigTabState'

export function useSyncLaunchPreferences() {
  const saveLaunchConfig = useAgentSessionLaunchPreferences(s => s.setLaunchConfig)
  const { agent, model, effort, thinking, permissionLevel } = useAgentSessionConfigLaunchState()

  useEffect(() => {
    saveLaunchConfig({ agent, model, effort, thinking, permissionLevel })
  }, [agent, effort, model, permissionLevel, saveLaunchConfig, thinking])
}
