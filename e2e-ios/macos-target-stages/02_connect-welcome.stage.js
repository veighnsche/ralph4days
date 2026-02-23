import { clickByTestId, saveScreenshot, waitForTestId } from '../remote-ssh.harness.js'
import { findMatchingProfileId, waitForText } from './common.js'

const targetFixture = process.env.RALPH_IOS_E2E_TARGET_FIXTURE?.trim() || '04-desktop-dev'

async function clickDiscoveredProjectFixture(fixtureName) {
  const clicked = await browser.execute(target => {
    const sectionHeaders = Array.from(document.querySelectorAll('h2'))
    const discoveredHeader = sectionHeaders.find(header => header.textContent?.includes('Discovered Projects'))
    if (!(discoveredHeader instanceof HTMLElement)) {
      throw new Error('Missing "Discovered Projects" section')
    }

    let sectionRoot = discoveredHeader.parentElement
    while (sectionRoot && sectionRoot.tagName !== 'DIV') {
      sectionRoot = sectionRoot.parentElement
    }
    if (!(sectionRoot instanceof HTMLElement)) {
      throw new Error('Failed to resolve Discovered Projects section container')
    }

    const projectButtons = Array.from(sectionRoot.querySelectorAll('button'))
    const targetButton = projectButtons.find(button => button.textContent?.includes(`/${target}`))
    if (!(targetButton instanceof HTMLElement)) {
      return false
    }

    targetButton.scrollIntoView({ block: 'center', inline: 'nearest' })
    targetButton.click()
    return true
  }, fixtureName)

  if (!clicked) {
    throw new Error(`Expected fixture '/${fixtureName}' inside Discovered Projects section`)
  }
}

export async function runStage02ConnectToWelcome() {
  await waitForTestId('ssh-connections-panel', 120000)
  const existingProfileId = await findMatchingProfileId()
  if (!existingProfileId) {
    throw new Error(
      "Missing prerequisite: macOS SSH profile is not present for stage 02. Run stage 01B ('01B_preclean-add-profile-variants') first."
    )
  }

  await saveScreenshot('remote-ssh-macos-04-existing-config-present')
  await clickByTestId(`ssh-profile-connect-${existingProfileId}`)
  await waitForTestId('ssh-connect-dialog')
  await saveScreenshot('remote-ssh-macos-05-connect-dialog-existing-config')
  await clickByTestId('ssh-connect-now-button')

  await waitForText('Ralph4days', 120000)
  await waitForText('Recent Projects', 120000)
  await saveScreenshot('remote-ssh-macos-06-welcome-project-select')

  await clickDiscoveredProjectFixture(targetFixture)

  await browser.waitUntil(
    async () => browser.execute(() => document.body?.textContent?.includes('Recent Projects') !== true),
    {
      timeout: 120000,
      interval: 250,
      timeoutMsg: 'Expected to leave project selector after selecting discovered fixture project'
    }
  )
  await saveScreenshot('remote-ssh-macos-07-project-opened-from-discovered')
}
