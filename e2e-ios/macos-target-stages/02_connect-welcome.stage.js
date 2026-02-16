import { clickByTestId, saveScreenshot, waitForTestId } from '../remote-ssh.harness.js'
import { findMatchingProfileId, waitForText } from './common.js'

export async function runStage02ConnectToWelcome() {
  await waitForTestId('ssh-connections-panel', 120000)
  const existingProfileId = await findMatchingProfileId()
  if (!existingProfileId) {
    throw new Error('Expected existing macOS SSH profile from previous stage')
  }

  await saveScreenshot('remote-ssh-macos-04-existing-config-present')
  await clickByTestId(`ssh-profile-connect-${existingProfileId}`)
  await waitForTestId('ssh-connect-dialog')
  await saveScreenshot('remote-ssh-macos-05-connect-dialog-existing-config')
  await clickByTestId('ssh-connect-now-button')

  await waitForText('Ralph4days', 120000)
  await waitForText('Recent Projects', 120000)
  await saveScreenshot('remote-ssh-macos-06-welcome-project-select')
}
