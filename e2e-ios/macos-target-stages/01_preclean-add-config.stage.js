import { clickByTestId, listProfileIds, saveScreenshot, setInputByTestId, waitForProfileCount, waitForTestId } from '../remote-ssh.harness.js'
import {
  assertProfileCardMatches,
  removeExistingTargetProfileIfPresent,
  targetHost,
  targetName,
  targetRalphdPort,
  targetSshPort,
  targetUsername
} from './common.js'

export async function runStage01PrecleanAndAddConfig() {
  await waitForTestId('ssh-connections-panel', 120000)

  await removeExistingTargetProfileIfPresent()
  await saveScreenshot('remote-ssh-macos-01-after-preclean')

  await clickByTestId('ssh-new-profile-button')
  await waitForTestId('ssh-profile-editor')

  await setInputByTestId('ssh-profile-name-input', targetName)
  await setInputByTestId('ssh-host-input', targetHost)
  await setInputByTestId('ssh-username-input', targetUsername)
  await setInputByTestId('ssh-port-input', targetSshPort)
  await setInputByTestId('ralphd-port-input', targetRalphdPort)
  await saveScreenshot('remote-ssh-macos-02-editor-filled')

  await clickByTestId('ssh-profile-save-button')
  await waitForProfileCount(1)
  await saveScreenshot('remote-ssh-macos-03-profile-created')

  const [profileId] = await listProfileIds()
  if (!profileId) {
    throw new Error('Expected created SSH profile id for macOS target')
  }

  await assertProfileCardMatches(profileId)
}
