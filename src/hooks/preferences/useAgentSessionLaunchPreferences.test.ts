import { beforeEach, describe, expect, it } from 'vitest'
import { expectNoStoreTransitions, expectStoreTransitions } from '@/test/zustand-store-test-utils'
import type { AgentSessionLaunchConfig } from './useAgentSessionLaunchPreferences'
import { useAgentSessionLaunchPreferences } from './useAgentSessionLaunchPreferences'

const BASE_CONFIG: AgentSessionLaunchConfig = {
  agent: 'claude',
  model: 'claude-sonnet-4',
  effort: 'medium',
  thinking: true,
  permissionLevel: 'balanced'
}

describe('useAgentSessionLaunchPreferences', () => {
  beforeEach(() => {
    localStorage.clear()
    useAgentSessionLaunchPreferences.getState().setLaunchConfig(BASE_CONFIG)
  })

  it('does not emit redundant transitions for same-value writes', () => {
    const store = useAgentSessionLaunchPreferences.getState()
    expectNoStoreTransitions(useAgentSessionLaunchPreferences, () => {
      store.setAgent(BASE_CONFIG.agent)
      store.setModel(BASE_CONFIG.model)
      store.setEffort(BASE_CONFIG.effort)
      store.setThinking(BASE_CONFIG.thinking)
      store.setPermissionLevel(BASE_CONFIG.permissionLevel)
      store.setLaunchConfig(BASE_CONFIG)
    })
  })

  it('emits once when launch config actually changes', () => {
    expectStoreTransitions(
      useAgentSessionLaunchPreferences,
      () => {
        useAgentSessionLaunchPreferences.getState().setLaunchConfig({
          ...BASE_CONFIG,
          agent: 'codex',
          model: 'gpt-5.3-codex'
        })
      },
      1
    )

    expect(useAgentSessionLaunchPreferences.getState()).toMatchObject({
      agent: 'codex',
      model: 'gpt-5.3-codex'
    })
  })
})
