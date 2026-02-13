import {
  clickButtonByText,
  clickContextMenuItem,
  clickElementWithDomClick,
  clickTab,
  clickTabCloseButton,
  closeAllTabsIfAny,
  ensureWorkspaceReady,
  getActiveTabId,
  getOpenContextMenuItems,
  getWorkspaceTabs,
  isEmptyWorkspaceVisible,
  middleClickTab,
  openTabContextMenu,
  sendKeyToTab,
  waitForActiveTabId,
  waitForTabCount
} from './workspace.harness.js'

const NEW_TERMINAL_SELECTOR = '[data-testid="workspace-new-terminal"]'

async function assertTauriRuntime() {
  const isTauriRuntime = await browser.execute(() => {
    return typeof window.__TAURI__ !== 'undefined' || typeof window.__TAURI_IPC__ !== 'undefined'
  })
  expect(isTauriRuntime).toBe(true)
}

async function createTerminalTabs(count) {
  const initial = await getWorkspaceTabs()
  const expectedCount = initial.length + count

  for (let i = 0; i < count; i += 1) {
    await clickElementWithDomClick(NEW_TERMINAL_SELECTOR)
  }

  await waitForTabCount(expectedCount)
}

async function createTerminalTabsAndCollectIds(count) {
  const tabIds = []
  for (let i = 0; i < count; i += 1) {
    await clickElementWithDomClick(NEW_TERMINAL_SELECTOR)
    await waitForTabCount(Math.min(i + 1, 10))
    tabIds.push(await getActiveTabId())
  }
  return tabIds
}

describe('Workspace tabs e2e logic', () => {
  beforeEach(async () => {
    await ensureWorkspaceReady()
    await assertTauriRuntime()
    await closeAllTabsIfAny()
    await waitForTabCount(0)
  })

  it('supports context menu actions for workspace tabs', async () => {
    expect(await isEmptyWorkspaceVisible()).toBe(true)

    await createTerminalTabs(3)
    await waitForTabCount(3)

    let tabs = await getWorkspaceTabs()
    expect(tabs).toHaveLength(3)
    expect(new Set(tabs.map(tab => tab.id)).size).toBe(3)

    const anchorTabId = tabs[0].id

    await openTabContextMenu(anchorTabId)
    await clickContextMenuItem('New Tab to the Right')
    await waitForTabCount(4)

    tabs = await getWorkspaceTabs()
    expect(tabs[1]?.title).toBe('New Terminal')

    await openTabContextMenu(tabs[1].id)
    await clickContextMenuItem('Close Tabs to the Right')
    await waitForTabCount(2)

    tabs = await getWorkspaceTabs()
    expect(tabs).toHaveLength(2)
    expect(tabs[0].id).toBe(anchorTabId)

    await createTerminalTabs(1)
    tabs = await getWorkspaceTabs()
    const keepTabId = tabs[1].id

    await openTabContextMenu(keepTabId)
    await clickContextMenuItem('Close Others')
    await waitForTabCount(1)
    await waitForActiveTabId(keepTabId)

    tabs = await getWorkspaceTabs()
    expect(tabs[0].id).toBe(keepTabId)

    await openTabContextMenu(keepTabId)
    await clickContextMenuItem('Close All Tabs')
    await waitForTabCount(0)
    expect(await isEmptyWorkspaceVisible()).toBe(true)
  })

  it('shows context menu items based on tab position and count', async () => {
    await createTerminalTabs(1)

    let tabs = await getWorkspaceTabs()
    await openTabContextMenu(tabs[0].id)
    let menuItems = await getOpenContextMenuItems()

    expect(menuItems).toContain('New Tab to the Right')
    expect(menuItems).toContain('Close')
    expect(menuItems).not.toContain('Close Others')
    expect(menuItems).not.toContain('Close Tabs to the Right')

    await clickContextMenuItem('Close All Tabs')
    await waitForTabCount(0)

    await createTerminalTabs(3)
    tabs = await getWorkspaceTabs()
    await openTabContextMenu(tabs[0].id)
    menuItems = await getOpenContextMenuItems()

    expect(menuItems).toContain('Close Tabs to the Right')
    expect(menuItems).toContain('Close Others')

    await clickContextMenuItem('Close All Tabs')
    await waitForTabCount(0)

    await createTerminalTabs(3)
    tabs = await getWorkspaceTabs()
    await openTabContextMenu(tabs[2].id)
    menuItems = await getOpenContextMenuItems()
    expect(menuItems).toContain('Close Others')
    expect(menuItems).not.toContain('Close Tabs to the Right')

    await clickContextMenuItem('Close All Tabs')
    await waitForTabCount(0)
  })

  it('supports close via context-menu item and middle-click', async () => {
    await createTerminalTabs(3)
    const tabs = await getWorkspaceTabs()
    const firstTabId = tabs[0].id
    const secondTabId = tabs[1].id
    const thirdTabId = tabs[2].id

    await clickTab(secondTabId)
    await waitForActiveTabId(secondTabId)

    await openTabContextMenu(firstTabId)
    await clickContextMenuItem('Close')
    await waitForTabCount(2)
    await waitForActiveTabId(secondTabId)

    let remainingTabs = await getWorkspaceTabs()
    expect(remainingTabs.map(tab => tab.id)).toEqual([secondTabId, thirdTabId])

    await middleClickTab(secondTabId)
    await waitForTabCount(1)
    await waitForActiveTabId(thirdTabId)

    remainingTabs = await getWorkspaceTabs()
    expect(remainingTabs.map(tab => tab.id)).toEqual([thirdTabId])
  })

  it('supports wrap-around keyboard navigation across tabs', async () => {
    await createTerminalTabs(3)
    const tabs = await getWorkspaceTabs()
    const firstTabId = tabs[0].id
    const thirdTabId = tabs[2].id

    await clickTab(firstTabId)
    await waitForActiveTabId(firstTabId)

    await sendKeyToTab(firstTabId, 'ArrowLeft')
    await waitForActiveTabId(thirdTabId)

    await sendKeyToTab(thirdTabId, 'ArrowRight')
    await waitForActiveTabId(firstTabId)

    await sendKeyToTab(firstTabId, 'ArrowUp')
    await waitForActiveTabId(thirdTabId)

    await sendKeyToTab(thirdTabId, 'ArrowDown')
    await waitForActiveTabId(firstTabId)
  })

  it('close-tabs-right keeps ordering and rehomes active tab when needed', async () => {
    await createTerminalTabs(4)
    const tabs = await getWorkspaceTabs()
    const firstTabId = tabs[0].id
    const secondTabId = tabs[1].id
    const fourthTabId = tabs[3].id

    await clickTab(fourthTabId)
    await waitForActiveTabId(fourthTabId)

    await openTabContextMenu(secondTabId)
    await clickContextMenuItem('Close Tabs to the Right')
    await waitForTabCount(2)
    await waitForActiveTabId(secondTabId)

    const remainingTabs = await getWorkspaceTabs()
    expect(remainingTabs.map(tab => tab.id)).toEqual([firstTabId, secondTabId])
  })

  it('caps workspace tabs at max count and evicts oldest tabs', async () => {
    const createdTabIds = await createTerminalTabsAndCollectIds(12)
    const tabs = await getWorkspaceTabs()
    const finalTabIds = tabs.map(tab => tab.id)
    const expectedIds = createdTabIds.slice(-10)

    expect(tabs).toHaveLength(10)
    expect(finalTabIds).toEqual(expectedIds)
    await waitForActiveTabId(expectedIds[expectedIds.length - 1])
  })

  it('supports switching, keyboard navigation, and close flows', async () => {
    await createTerminalTabs(3)
    await waitForTabCount(3)

    const tabs = await getWorkspaceTabs()
    const firstTabId = tabs[0].id
    const secondTabId = tabs[1].id
    const thirdTabId = tabs[2].id

    await clickTab(firstTabId)
    await waitForActiveTabId(firstTabId)

    await sendKeyToTab(firstTabId, 'ArrowRight')
    await waitForActiveTabId(secondTabId)

    await sendKeyToTab(secondTabId, 'ArrowLeft')
    await waitForActiveTabId(firstTabId)

    await sendKeyToTab(firstTabId, 'End')
    await waitForActiveTabId(thirdTabId)

    await sendKeyToTab(thirdTabId, 'Home')
    await waitForActiveTabId(firstTabId)

    await sendKeyToTab(firstTabId, 'Delete')
    await waitForTabCount(2)
    await waitForActiveTabId(secondTabId)

    let remainingTabs = await getWorkspaceTabs()
    expect(remainingTabs.map(tab => tab.id)).toEqual([secondTabId, thirdTabId])

    const activeBeforeClose = await getActiveTabId()
    await clickTabCloseButton(activeBeforeClose)
    await waitForTabCount(1)

    remainingTabs = await getWorkspaceTabs()
    expect(remainingTabs).toHaveLength(1)
    expect(remainingTabs[0].id).not.toBe(activeBeforeClose)

    await clickTabCloseButton(remainingTabs[0].id)
    await waitForTabCount(0)
    expect(await isEmptyWorkspaceVisible()).toBe(true)

    await clickButtonByText('Create terminal tab')
    await waitForTabCount(1)

    const terminalHost = await $('[data-testid="workspace-terminal-host"]')
    await terminalHost.waitForDisplayed({ timeout: 30000 })
    expect(await terminalHost.isDisplayed()).toBe(true)
  })
})
