import { saveScreenshot, waitForTestId } from '../remote-ssh.harness.js'
import {
  addProfile,
  findMatchingProfileId,
  removeAllProfilesBeforeSetup,
  targetHost,
  targetName,
  targetPassword,
  targetRalphdPort,
  targetSshPort,
  targetUsername,
  waitForProfiles
} from './common.js'

export async function runStage01BPrecleanAndAddProfileVariants() {
  await waitForTestId('ssh-connections-panel', 120000)

  await removeAllProfilesBeforeSetup()
  await saveScreenshot('remote-ssh-macos-01B-after-preclean-all')

  await addProfile({
    name: `${targetName} Password`,
    host: targetHost,
    username: targetUsername,
    sshPort: targetSshPort,
    ralphdPort: targetRalphdPort,
    authMode: 'password',
    password: targetPassword
  })
  await waitForProfiles(1)
  await saveScreenshot('remote-ssh-macos-01B-password-profile')

  await addProfile({
    name: targetName,
    host: targetHost,
    username: targetUsername,
    sshPort: targetSshPort,
    ralphdPort: targetRalphdPort,
    authMode: 'key',
    identityFile: `/Users/${targetUsername}/.ssh/id_ed25519`
  })
  await waitForProfiles(2)
  await saveScreenshot('remote-ssh-macos-01B-key-connectable-profile')

  await addProfile({
    name: `${targetName} Key Setup Required`,
    host: targetHost,
    username: targetUsername,
    sshPort: targetSshPort,
    ralphdPort: targetRalphdPort,
    authMode: 'key'
  })
  await waitForProfiles(3)
  await saveScreenshot('remote-ssh-macos-01B-key-setup-required-profile')

  const targetProfileId = await findMatchingProfileId()
  if (!targetProfileId) {
    throw new Error("Expected connectable target profile to exist after stage 01B (profile named targetName)")
  }
}
