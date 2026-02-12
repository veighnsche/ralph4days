import { describe, expect, it } from 'vitest'
import { expectNoStoreTransitions, expectStoreTransitions } from '@/test/zustand-store-test-utils'
import { createAgentSessionConfigStore } from './store'

describe('agent session config tab store', () => {
  it('does not emit redundant transitions for same-value writes', () => {
    const store = createAgentSessionConfigStore({
      agent: 'claude',
      model: 'claude-sonnet-4',
      effort: 'medium',
      thinking: true,
      permissionLevel: 'balanced'
    })

    const state = store.getState()
    expectNoStoreTransitions(store, () => {
      state.setAgent('claude')
      state.setModel('claude-sonnet-4')
      state.setEffort('medium')
      state.setThinking(true)
      state.setPermissionLevel('balanced')
    })

    expectStoreTransitions(store, () => state.setAgent('codex'), 1)
    expectNoStoreTransitions(store, () => state.setAgent('codex'))
    expect(store.getState().agent).toBe('codex')
  })
})
