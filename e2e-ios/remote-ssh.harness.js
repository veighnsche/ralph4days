import { mkdirSync } from 'fs'
import path from 'path'

export const screenshotDir = process.env.RALPH_IOS_E2E_SCREENSHOT_DIR?.trim() || '/tmp/ralph-ios-e2e'

export async function saveScreenshot(name) {
  mkdirSync(screenshotDir, { recursive: true })
  const target = path.join(screenshotDir, `${name}.png`)
  await browser.saveScreenshot(target)
}

export async function switchToWebViewContext() {
  await browser.waitUntil(
    async () => {
      const contexts = await browser.getContexts()
      return contexts.some(context => context.startsWith('WEBVIEW_'))
    },
    {
      timeout: 90000,
      interval: 500,
      timeoutMsg: 'No WEBVIEW context detected for iOS app session'
    }
  )

  const contexts = await browser.getContexts()
  const webviewContext = contexts.find(context => context.startsWith('WEBVIEW_'))
  if (!webviewContext) {
    throw new Error(`Expected WEBVIEW context, received: ${JSON.stringify(contexts)}`)
  }
  await browser.switchContext(webviewContext)
}

export async function waitForTestId(testId, timeout = 30000) {
  const element = await $(`[data-testid="${testId}"]`)
  await element.waitForDisplayed({
    timeout,
    timeoutMsg: `Expected element with data-testid='${testId}'`
  })
  return element
}

export async function clickByTestId(testId) {
  await waitForTestId(testId)
  await browser.execute(id => {
    const target = document.querySelector(`[data-testid="${id}"]`)
    if (!(target instanceof HTMLElement)) {
      throw new Error(`Missing clickable element for '${id}'`)
    }
    target.scrollIntoView({ block: 'center', inline: 'nearest' })
    target.click()
  }, testId)
}

export async function setInputByTestId(testId, value) {
  const element = await waitForTestId(testId)
  await element.clearValue()
  await element.setValue(value)
}

export async function readTextByTestId(testId) {
  return browser.execute(id => {
    const target = document.querySelector(`[data-testid="${id}"]`)
    if (!(target instanceof HTMLElement)) {
      throw new Error(`Missing element for '${id}'`)
    }
    return target.textContent ?? ''
  }, testId)
}

export async function listProfileIds() {
  return browser.execute(() =>
    Array.from(document.querySelectorAll('[data-testid^="ssh-profile-card-"]'))
      .map(node => node.getAttribute('data-testid') ?? '')
      .map(value => value.replace('ssh-profile-card-', ''))
      .filter(Boolean)
  )
}

export async function waitForProfileCount(expectedCount, timeout = 30000) {
  await browser.waitUntil(
    async () =>
      browser.execute(count => document.querySelectorAll('[data-testid^="ssh-profile-card-"]').length === count, expectedCount),
    {
      timeout,
      interval: 200,
      timeoutMsg: `Expected ${expectedCount} SSH profile rows`
    }
  )
}

export async function deleteAllProfiles() {
  const ids = await listProfileIds()
  for (const id of ids) {
    await clickByTestId(`ssh-profile-delete-${id}`)
    await waitForTestId('ssh-delete-dialog')
    await clickByTestId('ssh-delete-confirm-button')
  }
  await waitForProfileCount(0)
}
