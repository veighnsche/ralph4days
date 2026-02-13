async function switchToMainWindow() {
  const handles = await browser.getWindowHandles()
  for (const handle of handles) {
    await browser.switchToWindow(handle)
    const hasMainRoot = await browser.execute(() => document.getElementById('root') !== null)
    if (hasMainRoot) return true
  }

  return false
}

async function waitForMainWindow() {
  await browser.waitUntil(
    async () => {
      const matched = await switchToMainWindow()
      if (!matched) return false

      const hasWorkspaceMarkup = await $('[data-testid="workspace-new-terminal"]').isExisting()
      return hasWorkspaceMarkup
    },
    {
      timeout: 30000,
      interval: 250,
      timeoutMsg: 'Main workspace window did not initialize within timeout.'
    }
  )
}

describe('Terminal e2e flow', () => {
  it('opens terminal UI in the native Tauri window', async () => {
    await waitForMainWindow()

    const isTauriRuntime = await browser.execute(() => {
      return typeof window.__TAURI__ !== 'undefined' || typeof window.__TAURI_IPC__ !== 'undefined'
    })
    expect(isTauriRuntime).toBe(true)

    const plusButton = await $('[data-testid="workspace-new-terminal"]')
    await plusButton.waitForDisplayed({ timeout: 30000 })
    await plusButton.click()

    const terminalHost = await $('[data-testid="workspace-terminal-host"]')
    await terminalHost.waitForDisplayed({ timeout: 30000 })
    expect(await terminalHost.isDisplayed()).toBe(true)
  })
})
