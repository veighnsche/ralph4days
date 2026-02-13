export async function switchToMainWindow() {
  const handles = await browser.getWindowHandles()
  for (const handle of handles) {
    await browser.switchToWindow(handle)
    const hasMainRoot = await browser.execute(() => document.getElementById('root') !== null)
    if (hasMainRoot) return true
  }

  return false
}

export async function waitForMainWindow() {
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

export async function collectStartupDiagnostics() {
  const handles = await browser.getWindowHandles()
  const windows = []

  for (const handle of handles) {
    await browser.switchToWindow(handle)
    const snapshot = await browser.execute(() => {
      const hasRoot = document.getElementById('root') !== null
      const hasWorkspaceNewTerminal = document.querySelector('[data-testid="workspace-new-terminal"]') !== null
      const hasTerminalHost = document.querySelector('[data-testid="workspace-terminal-host"]') !== null
      const hasAddRalphButton = Array.from(document.querySelectorAll('button')).some(
        button => button.textContent?.trim() === 'Add Ralph'
      )
      const pagePreview = (document.body?.innerText ?? '').replace(/\s+/g, ' ').trim().slice(0, 180)

      return {
        href: window.location.href,
        title: document.title,
        hasRoot,
        hasWorkspaceNewTerminal,
        hasTerminalHost,
        hasAddRalphButton,
        pagePreview
      }
    })

    windows.push({ handle, ...snapshot })
  }

  return { handleCount: handles.length, windows }
}

export async function ensureWorkspaceReady() {
  try {
    await waitForMainWindow()
  } catch (error) {
    const diagnostics = await collectStartupDiagnostics()
    throw new Error(`Main workspace did not initialize in time. Diagnostics: ${JSON.stringify(diagnostics)}`, {
      cause: error
    })
  }
}

export async function clickElementWithDomClick(selector) {
  const element = await $(selector)
  await element.waitForDisplayed({ timeout: 30000 })
  await browser.execute(sel => {
    const target = document.querySelector(sel)
    if (!target) {
      throw new Error(`Missing element for selector: ${sel}`)
    }
    target.click()
  }, selector)
}

export async function clickButtonByText(text) {
  await browser.waitUntil(
    async () =>
      browser.execute(label => {
        const normalize = value => value.replace(/\s+/g, ' ').trim()
        return Array.from(document.querySelectorAll('button')).some(button => normalize(button.textContent ?? '') === label)
      }, text),
    {
      timeout: 30000,
      interval: 250,
      timeoutMsg: `Button not found with text: ${text}`
    }
  )

  await browser.execute(label => {
    const normalize = value => value.replace(/\s+/g, ' ').trim()
    const target = Array.from(document.querySelectorAll('button')).find(button => normalize(button.textContent ?? '') === label)
    if (!target) {
      throw new Error(`Missing button with text: ${label}`)
    }
    target.click()
  }, text)
}

export async function clickRoleElementByText(role, text) {
  await browser.waitUntil(
    async () =>
      browser.execute(
        input => {
          const normalize = value => value.replace(/\s+/g, ' ').trim()
          return Array.from(document.querySelectorAll(`[role="${input.role}"]`)).some(
            element => normalize(element.textContent ?? '').includes(input.text)
          )
        },
        { role, text }
      ),
    {
      timeout: 30000,
      interval: 250,
      timeoutMsg: `Role element not found. role=${role} text=${text}`
    }
  )

  await browser.execute(
    input => {
      const normalize = value => value.replace(/\s+/g, ' ').trim()
      const target = Array.from(document.querySelectorAll(`[role="${input.role}"]`)).find(element =>
        normalize(element.textContent ?? '').includes(input.text)
      )
      if (!target) {
        throw new Error(`Missing role element. role=${input.role} text=${input.text}`)
      }
      target.click()
    },
    { role, text }
  )
}

export async function clickButtonByAccessibleName(name) {
  await browser.waitUntil(
    async () =>
      browser.execute(label => {
        const normalize = value => value.replace(/\s+/g, ' ').trim()
        return Array.from(document.querySelectorAll('button')).some(button => {
          const aria = normalize(button.getAttribute('aria-label') ?? '')
          const srOnlyText = normalize(button.querySelector('.sr-only')?.textContent ?? '')
          const text = normalize(button.textContent ?? '')
          return aria === label || srOnlyText === label || text === label
        })
      }, name),
    {
      timeout: 30000,
      interval: 250,
      timeoutMsg: `Button not found by accessible name: ${name}`
    }
  )

  await browser.execute(label => {
    const normalize = value => value.replace(/\s+/g, ' ').trim()
    const target = Array.from(document.querySelectorAll('button')).find(button => {
      const aria = normalize(button.getAttribute('aria-label') ?? '')
      const srOnlyText = normalize(button.querySelector('.sr-only')?.textContent ?? '')
      const text = normalize(button.textContent ?? '')
      return aria === label || srOnlyText === label || text === label
    })
    if (!target) {
      throw new Error(`Missing button by accessible name: ${label}`)
    }
    target.click()
  }, name)
}

export async function clickEnabledButtonByText(text) {
  await browser.waitUntil(
    async () =>
      browser.execute(label => {
        const normalize = value => value.replace(/\s+/g, ' ').trim()
        return Array.from(document.querySelectorAll('button')).some(button => {
          if (button.disabled) return false
          return normalize(button.textContent ?? '') === label
        })
      }, text),
    {
      timeout: 30000,
      interval: 250,
      timeoutMsg: `Enabled button not found with text: ${text}`
    }
  )

  await browser.execute(label => {
    const normalize = value => value.replace(/\s+/g, ' ').trim()
    const target = Array.from(document.querySelectorAll('button')).find(button => {
      if (button.disabled) return false
      return normalize(button.textContent ?? '') === label
    })
    if (!target) {
      throw new Error(`Missing enabled button with text: ${label}`)
    }
    target.click()
  }, text)
}

export async function getWorkspaceTabs() {
  return browser.execute(() =>
    Array.from(document.querySelectorAll('[role="tab"]')).map(tab => {
      const idAttr = tab.getAttribute('id') ?? ''
      const id = idAttr.startsWith('tab-') ? idAttr.slice(4) : idAttr
      const title = tab.querySelector('span')?.textContent?.trim() ?? ''
      const selected = tab.getAttribute('aria-selected') === 'true'
      return { id, title, selected }
    })
  )
}

export async function getActiveTabId() {
  return browser.execute(() => {
    const active = document.querySelector('[role="tab"][aria-selected="true"]')
    if (!active) return ''
    const idAttr = active.getAttribute('id') ?? ''
    return idAttr.startsWith('tab-') ? idAttr.slice(4) : idAttr
  })
}

export async function waitForTabCount(expectedCount, timeout = 30000) {
  await browser.waitUntil(
    async () => {
      const tabs = await getWorkspaceTabs()
      return tabs.length === expectedCount
    },
    {
      timeout,
      interval: 200,
      timeoutMsg: `Expected ${expectedCount} workspace tabs`
    }
  )
}

export async function waitForActiveTabId(expectedTabId, timeout = 30000) {
  await browser.waitUntil(
    async () => {
      const activeTabId = await getActiveTabId()
      return activeTabId === expectedTabId
    },
    {
      timeout,
      interval: 200,
      timeoutMsg: `Expected active tab: ${expectedTabId}`
    }
  )
}

export async function clickTab(tabId) {
  await clickElementWithDomClick(`#tab-${tabId}`)
}

export async function clickTabCloseButton(tabId) {
  await browser.execute(id => {
    const tab = document.getElementById(`tab-${id}`)
    if (!tab) {
      throw new Error(`Missing tab: ${id}`)
    }

    const closeButton = tab.querySelector('button[aria-label^="Close "]')
    if (!closeButton) {
      throw new Error(`Missing close button for tab: ${id}`)
    }

    closeButton.click()
  }, tabId)
}

export async function openTabContextMenu(tabId) {
  await browser.execute(id => {
    const tab = document.getElementById(`tab-${id}`)
    if (!tab) {
      throw new Error(`Missing tab for context menu: ${id}`)
    }

    const rect = tab.getBoundingClientRect()
    const event = new MouseEvent('contextmenu', {
      bubbles: true,
      cancelable: true,
      button: 2,
      clientX: Math.round(rect.left + Math.min(8, Math.max(1, rect.width - 1))),
      clientY: Math.round(rect.top + Math.min(8, Math.max(1, rect.height - 1)))
    })

    tab.dispatchEvent(event)
  }, tabId)
}

export async function clickContextMenuItem(label) {
  await browser.waitUntil(
    async () =>
      browser.execute(itemLabel => {
        const normalize = value => value.replace(/\s+/g, ' ').trim()
        const openMenu = document.querySelector('[data-slot="context-menu-content"][data-state="open"]')
        if (!openMenu) return false
        return Array.from(openMenu.querySelectorAll('[data-slot="context-menu-item"]')).some(
          item => normalize(item.textContent ?? '') === itemLabel
        )
      }, label),
    {
      timeout: 30000,
      interval: 200,
      timeoutMsg: `Context menu item not found: ${label}`
    }
  )

  await browser.execute(itemLabel => {
    const normalize = value => value.replace(/\s+/g, ' ').trim()
    const openMenu = document.querySelector('[data-slot="context-menu-content"][data-state="open"]')
    if (!openMenu) {
      throw new Error('Missing open context menu')
    }

    const target = Array.from(openMenu.querySelectorAll('[data-slot="context-menu-item"]')).find(
      item => normalize(item.textContent ?? '') === itemLabel
    )

    if (!target) {
      throw new Error(`Missing context menu item: ${itemLabel}`)
    }

    target.click()
  }, label)
}

export async function getOpenContextMenuItems() {
  return browser.execute(() => {
    const normalize = value => value.replace(/\s+/g, ' ').trim()
    const openMenu = document.querySelector('[data-slot="context-menu-content"][data-state="open"]')
    if (!openMenu) {
      throw new Error('Missing open context menu')
    }

    return Array.from(openMenu.querySelectorAll('[data-slot="context-menu-item"]')).map(item =>
      normalize(item.textContent ?? '')
    )
  })
}

export async function middleClickTab(tabId) {
  await browser.execute(id => {
    const tab = document.getElementById(`tab-${id}`)
    if (!tab) {
      throw new Error(`Missing tab for middle click: ${id}`)
    }

    tab.dispatchEvent(
      new MouseEvent('auxclick', {
        bubbles: true,
        cancelable: true,
        button: 1
      })
    )
  }, tabId)
}

export async function sendKeyToTab(tabId, key) {
  await browser.execute(
    input => {
      const tab = document.getElementById(`tab-${input.tabId}`)
      if (!tab) {
        throw new Error(`Missing tab for key event: ${input.tabId}`)
      }

      tab.focus()
      const event = new KeyboardEvent('keydown', {
        key: input.key,
        bubbles: true,
        cancelable: true
      })
      tab.dispatchEvent(event)
    },
    { tabId, key }
  )
}

export async function isEmptyWorkspaceVisible() {
  return browser.execute(() => (document.body?.innerText ?? '').includes('No workspace tabs open'))
}

export async function waitForVisibleTerminalHost(timeout = 30000) {
  await browser.waitUntil(
    async () =>
      browser.execute(() => {
        const isElementVisiblyRendered = element => {
          if (!(element instanceof HTMLElement)) return false
          const style = window.getComputedStyle(element)
          if (style.display === 'none' || style.visibility === 'hidden') return false
          const rect = element.getBoundingClientRect()
          if (rect.width <= 1 || rect.height <= 1) return false
          return true
        }
        const hosts = Array.from(document.querySelectorAll('[data-testid="workspace-terminal-host"]'))
        return hosts.some(host => isElementVisiblyRendered(host))
      }),
    {
      timeout,
      interval: 250,
      timeoutMsg: 'No visible terminal host was rendered.'
    }
  )
}

export async function resetTerminalDiagnostics() {
  await browser.execute(() => {
    const maybeWindow = window
    if (!maybeWindow.__RALPH_TERMINAL_DIAG__) return
    maybeWindow.__RALPH_TERMINAL_DIAG__.sessions = {}
  })
}

export async function getTerminalSessionDiagnostics(sessionId) {
  return browser.execute(id => {
    return window.__RALPH_TERMINAL_DIAG__?.sessions?.[id] ?? null
  }, sessionId)
}

function isTerminalPipelineReady(diag) {
  if (!diag) return false
  if (diag.bridgeChunks < 1) return false
  if (diag.bridgeBytes < 16) return false
  if (diag.writeCalls < 1) return false
  if (diag.writeBytes < 16) return false
  if (diag.renderEvents < 1) return false
  if (diag.nonEmptyRenderEvents < 1) return false
  if (diag.maxVisibleNonEmptyLines < 1) return false
  return true
}

export async function waitForTerminalPipelineReady(sessionId, timeout = 45000) {
  let lastDiag = null
  try {
    await browser.waitUntil(
      async () => {
        lastDiag = await getTerminalSessionDiagnostics(sessionId)
        return isTerminalPipelineReady(lastDiag)
      },
      {
        timeout,
        interval: 300,
        timeoutMsg: `Terminal pipeline did not become ready for session: ${sessionId}`
      }
    )
  } catch (error) {
    throw new Error(
      `Terminal pipeline stayed incomplete for session ${sessionId}. Diagnostics: ${JSON.stringify(lastDiag)}`,
      {
        cause: error
      }
    )
  }
}

function hasAnyPatternMatch(text, patterns) {
  if (typeof text !== 'string' || text.length === 0) return false
  const lowered = text.toLowerCase()
  return patterns.some(pattern => lowered.includes(pattern.toLowerCase()))
}

export async function waitForTerminalDiagnosticPattern(sessionId, patterns, timeout = 45000) {
  if (!Array.isArray(patterns) || patterns.length === 0) {
    throw new Error('waitForTerminalDiagnosticPattern requires at least one pattern')
  }

  let lastDiag = null
  try {
    await browser.waitUntil(
      async () => {
        lastDiag = await getTerminalSessionDiagnostics(sessionId)
        if (!lastDiag) return false

        return (
          hasAnyPatternMatch(lastDiag.lastRenderPreview, patterns) ||
          hasAnyPatternMatch(lastDiag.lastPlainPreview, patterns) ||
          hasAnyPatternMatch(lastDiag.lastPreview, patterns)
        )
      },
      {
        timeout,
        interval: 300,
        timeoutMsg: `Terminal diagnostics did not contain required text patterns for session: ${sessionId}`
      }
    )
  } catch (error) {
    throw new Error(
      `Terminal diagnostics missing required text patterns for session ${sessionId}. Patterns: ${JSON.stringify(patterns)}. Diagnostics: ${JSON.stringify(lastDiag)}`,
      { cause: error }
    )
  }
}

export async function waitForTerminalCanvasInk(timeout = 45000) {
  await browser.waitUntil(
    async () =>
      browser.execute(() => {
        const isElementVisiblyRendered = element => {
          if (!(element instanceof HTMLElement)) return false
          const style = window.getComputedStyle(element)
          if (style.display === 'none' || style.visibility === 'hidden') return false
          const rect = element.getBoundingClientRect()
          if (rect.width <= 1 || rect.height <= 1) return false
          return true
        }
        const hosts = Array.from(document.querySelectorAll('[data-testid="workspace-terminal-host"]'))
        const host = hosts.find(candidate => isElementVisiblyRendered(candidate))
        if (!(host instanceof HTMLElement)) return false

        const canvases = Array.from(host.querySelectorAll('canvas')).filter(
          canvas => canvas.width > 0 && canvas.height > 0
        )
        if (canvases.length === 0) return false

        const background = [10, 10, 10]

        for (const canvas of canvases) {
          const ctx = canvas.getContext('2d', { willReadFrequently: true })
          if (!ctx) continue

          const { width, height } = canvas
          if (width === 0 || height === 0) continue

          const image = ctx.getImageData(0, 0, width, height).data
          const stepX = Math.max(1, Math.floor(width / 48))
          const stepY = Math.max(1, Math.floor(height / 24))
          let visibleInkPixels = 0

          for (let y = 0; y < height; y += stepY) {
            for (let x = 0; x < width; x += stepX) {
              const idx = (y * width + x) * 4
              const alpha = image[idx + 3]
              if (alpha === 0) continue

              const diff =
                Math.abs(image[idx] - background[0]) +
                Math.abs(image[idx + 1] - background[1]) +
                Math.abs(image[idx + 2] - background[2])

              if (diff > 24) {
                visibleInkPixels += 1
                if (visibleInkPixels >= 24) return true
              }
            }
          }
        }

        return false
      }),
    {
      timeout,
      interval: 350,
      timeoutMsg: 'Terminal canvas stayed visually blank (no visible glyph ink over terminal background).'
    }
  )
}

export async function closeAllTabsIfAny() {
  const tabs = await getWorkspaceTabs()
  if (tabs.length === 0) return

  await openTabContextMenu(tabs[0].id)
  await clickContextMenuItem('Close All Tabs')
  await waitForTabCount(0)
}
