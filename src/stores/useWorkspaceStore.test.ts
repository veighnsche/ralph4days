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

  it('keeps active tab valid after closeToRight removes the active tab', () => {
    const store = useWorkspaceStore.getState()
    const left = store.openTab({ type: 'terminal', title: 'Left', closeable: true })
    const middle = store.openTab({ type: 'terminal', title: 'Middle', closeable: true })
    const right = store.openTab({ type: 'terminal', title: 'Right', closeable: true })
    expect(useWorkspaceStore.getState().activeTabId).toBe(right)

    useWorkspaceStore.getState().closeToRight(middle)
    const nextState = useWorkspaceStore.getState()
    expect(nextState.tabs.map(tab => tab.id)).toEqual([left, middle])
    expect(nextState.activeTabId).toBe(middle)
  })

  it('does not emit transition when switching to unknown tab id', () => {
    const store = useWorkspaceStore.getState()
    store.openTab({ type: 'terminal', title: 'A', closeable: true })

    expectNoStoreTransitions(useWorkspaceStore, () => {
      useWorkspaceStore.getState().switchTab('missing-tab')
    })
  })

  it('ensures activeTabId remains valid when closeAllExcept receives unknown tab id', () => {
    const store = useWorkspaceStore.getState()
    const pinned = store.openTab({ id: 'pinned', type: 'panel', title: 'Pinned', closeable: false })
    store.openTab({ type: 'terminal', title: 'A', closeable: true })
    store.openTab({ type: 'terminal', title: 'B', closeable: true })

    useWorkspaceStore.getState().closeAllExcept('missing-tab')
    const nextState = useWorkspaceStore.getState()
    expect(nextState.tabs.map(tab => tab.id)).toEqual([pinned])
    expect(nextState.activeTabId).toBe(pinned)
  })

  it('generates unique ids for non-keyed tabs opened back-to-back', () => {
    const first = useWorkspaceStore.getState().openTab({ type: 'terminal', title: 'A', closeable: true })
    const second = useWorkspaceStore.getState().openTab({ type: 'terminal', title: 'A', closeable: true })

    expect(first).not.toBe(second)
    expect(useWorkspaceStore.getState().tabs.map(tab => tab.id)).toEqual([first, second])
  })
})
