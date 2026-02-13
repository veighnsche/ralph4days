import {
  closeAllTabsIfAny,
  clickButtonByAccessibleName,
  clickElementWithDomClick,
  clickEnabledButtonByText,
  clickRoleElementByText,
  ensureWorkspaceReady,
  getActiveTabId,
  resetTerminalDiagnostics,
  waitForTabCount,
  waitForTerminalPipelineReady,
  waitForVisibleTerminalHost,
  waitForTerminalCanvasInk
} from './workspace.harness.js'

describe('Terminal e2e flow', () => {
  beforeEach(async () => {
    await ensureWorkspaceReady()
    await closeAllTabsIfAny()
    await waitForTabCount(0)
    await resetTerminalDiagnostics()
  })

  it('opens terminal UI in the native Tauri window', async () => {
    const isTauriRuntime = await browser.execute(() => {
      return typeof window.__TAURI__ !== 'undefined' || typeof window.__TAURI_IPC__ !== 'undefined'
    })
    expect(isTauriRuntime).toBe(true)

    await clickElementWithDomClick('[data-testid="workspace-new-terminal"]')

    const terminalHost = await $('[data-testid="workspace-terminal-host"]')
    await terminalHost.waitForDisplayed({ timeout: 30000 })
    expect(await terminalHost.isDisplayed()).toBe(true)
  })

  it('renders codex cli content to the terminal screen (not blank)', async () => {
    await clickButtonByAccessibleName('Open run form')
    await clickRoleElementByText('radio', 'Codex')
    await clickEnabledButtonByText('Run')

    const activeTabId = await getActiveTabId()
    expect(activeTabId).not.toBe('')

    await waitForTerminalPipelineReady(activeTabId)
    await waitForVisibleTerminalHost()
    await waitForTerminalCanvasInk()
  })
})
