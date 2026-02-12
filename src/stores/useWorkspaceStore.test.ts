import { beforeEach, describe, expect, it } from 'vitest'
import { expectNoStoreTransitions, expectStoreTransitions } from '@/test/zustand-store-test-utils'
import { useWorkspaceStore } from './useWorkspaceStore'

describe('useWorkspaceStore', () => {
  beforeEach(() => {
    useWorkspaceStore.setState({ tabs: [], activeTabId: '' })
  })

  it('does not emit transition when switching to the current active tab', () => {
    const store = useWorkspaceStore.getState()
    const tabId = store.openTab({ type: 'terminal', title: 'A', closeable: true })

    expectNoStoreTransitions(useWorkspaceStore, () => {
      useWorkspaceStore.getState().switchTab(tabId)
    })
  })

  it('does not emit transition when tab meta is unchanged', () => {
    const store = useWorkspaceStore.getState()
    const tabId = store.openTab({ type: 'terminal', title: 'A', closeable: true })

    expectNoStoreTransitions(useWorkspaceStore, () => {
      useWorkspaceStore.getState().setTabMeta(tabId, { title: 'A' })
    })

    expectStoreTransitions(
      useWorkspaceStore,
      () => {
        useWorkspaceStore.getState().setTabMeta(tabId, { title: 'B' })
      },
      1
    )

    const updated = useWorkspaceStore.getState().tabs.find(tab => tab.id === tabId)
    expect(updated?.title).toBe('B')
  })
})
