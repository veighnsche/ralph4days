import { mkdirSync } from 'fs'
import path from 'path'

const screenshotDir = process.env.RALPH_IOS_E2E_SCREENSHOT_DIR?.trim() || '/tmp/ralph-ios-e2e'

async function saveScreenshot(name) {
  mkdirSync(screenshotDir, { recursive: true })
  const target = path.join(screenshotDir, `${name}.png`)
  await browser.saveScreenshot(target)
}

async function switchToWebViewContext() {
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

async function waitForTestId(testId, timeout = 30000) {
  const element = await $(`[data-testid="${testId}"]`)
  await element.waitForDisplayed({
    timeout,
    timeoutMsg: `Expected element with data-testid='${testId}'`
  })
  return element
}

async function clickByTestId(testId) {
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

async function setInputByTestId(testId, value) {
  const element = await waitForTestId(testId)
  await element.clearValue()
  await element.setValue(value)
}

async function listProfileIds() {
  return browser.execute(() =>
    Array.from(document.querySelectorAll('[data-testid^="ssh-profile-card-"]'))
      .map(node => node.getAttribute('data-testid') ?? '')
      .map(value => value.replace('ssh-profile-card-', ''))
      .filter(Boolean)
  )
}

async function waitForProfileCount(expectedCount, timeout = 30000) {
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

async function deleteAllProfiles() {
  const ids = await listProfileIds()
  for (const id of ids) {
    await clickByTestId(`ssh-profile-delete-${id}`)
    await waitForTestId('ssh-delete-dialog')
    await clickByTestId('ssh-delete-confirm-button')
  }
  await waitForProfileCount(0)
}

describe('iOS remote SSH Appium harness', () => {
  before(async () => {
    await switchToWebViewContext()
    await waitForTestId('ssh-connections-panel', 120000)
  })

  it('drives profile CRUD flow and captures screenshots', async () => {
    await deleteAllProfiles()
    await saveScreenshot('remote-ssh-00-empty')

    await clickByTestId('ssh-new-profile-button')
    await waitForTestId('ssh-profile-editor')
    await setInputByTestId('ssh-profile-name-input', 'E2E Host')
    await setInputByTestId('ssh-host-input', '127.0.0.1')
    await setInputByTestId('ssh-username-input', 'ralph')
    await saveScreenshot('remote-ssh-01-editor-filled')

    await clickByTestId('ssh-profile-save-button')
    await waitForProfileCount(1)
    await saveScreenshot('remote-ssh-02-profile-created')

    const [profileId] = await listProfileIds()
    if (!profileId) {
      throw new Error('Expected created SSH profile id')
    }

    await clickByTestId(`ssh-profile-connect-${profileId}`)
    await waitForTestId('ssh-connect-dialog')
    await saveScreenshot('remote-ssh-03-connect-dialog')
    await clickByTestId('ssh-connect-cancel-button')

    await clickByTestId(`ssh-profile-delete-${profileId}`)
    await waitForTestId('ssh-delete-dialog')
    await saveScreenshot('remote-ssh-04-delete-dialog')
    await clickByTestId('ssh-delete-confirm-button')
    await waitForProfileCount(0)
    await saveScreenshot('remote-ssh-05-empty-after-delete')
  })
})
