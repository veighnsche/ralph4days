import { listProfileIds, saveScreenshot, waitForTestId } from '../remote-ssh.harness.js'
import {
  addProfile,
  assertProfileCardMatches,
  removeAllProfilesBeforeSetup,
  targetHost,
  targetName,
  targetPassword,
  targetRalphdPort,
  targetSshPort,
  targetUsername,
  waitForProfiles
} from './common.js'

export async function runStage01APrecleanAndAddPasswordConfig() {
  await waitForTestId('ssh-connections-panel', 120000)

  await removeAllProfilesBeforeSetup()
  await saveScreenshot('remote-ssh-macos-01A-after-preclean-all')

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
  await saveScreenshot('remote-ssh-macos-01A-password-profile-created')

  const [profileId] = await listProfileIds()
  if (!profileId) {
    throw new Error('Expected created SSH profile id for stage 01A')
  }

  await assertProfileCardMatches(profileId)
}
