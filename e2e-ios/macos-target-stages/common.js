import {
  clickByTestId,
  deleteAllProfiles,
  openProfileActions,
  readTextByTestId,
  setInputByTestId,
  waitForProfileCount,
  waitForTestId
} from '../remote-ssh.harness.js'

export const targetName = process.env.RALPH_IOS_E2E_TARGET_NAME?.trim() || 'macOS Target'
export const targetHost = process.env.RALPH_IOS_E2E_TARGET_HOST?.trim()
export const targetUsername = process.env.RALPH_IOS_E2E_TARGET_USERNAME?.trim()
export const targetSshPort = process.env.RALPH_IOS_E2E_TARGET_SSH_PORT?.trim() || '22'
export const targetRalphdPort = process.env.RALPH_IOS_E2E_TARGET_RALPHD_PORT?.trim() || '9944'
export const targetPassword = process.env.RALPH_IOS_E2E_TARGET_PASSWORD?.trim() || 'ralph-password'

if (!targetHost) {
  throw new Error('RALPH_IOS_E2E_TARGET_HOST is required')
}

if (!targetUsername) {
  throw new Error('RALPH_IOS_E2E_TARGET_USERNAME is required')
}

export async function findMatchingProfileId() {
  return browser.execute(
    ({ name, host, username, sshPort, ralphdPort }) => {
      const cards = Array.from(document.querySelectorAll('[data-testid^="ssh-profile-card-"]'))
      for (const card of cards) {
        const testId = card.getAttribute('data-testid')
        if (!testId) continue
        const id = testId.replace('ssh-profile-card-', '')
        if (!id) continue
        const text = (card.textContent ?? '').replace(/\s+/g, ' ')
        if (
          text.includes(name) &&
          text.includes(`${username}@${host}:${sshPort}`) &&
          text.includes(`ralphd:${ralphdPort}`)
        ) {
          return id
        }
      }
      return null
    },
    {
      name: targetName,
      host: targetHost,
      username: targetUsername,
      sshPort: targetSshPort,
      ralphdPort: targetRalphdPort
    }
  )
}

export async function removeExistingTargetProfileIfPresent() {
  const existingProfileId = await findMatchingProfileId()
  if (!existingProfileId) {
    return
  }

  await openProfileActions(existingProfileId)
  await clickByTestId(`ssh-profile-action-delete-${existingProfileId}`)
  await waitForTestId('ssh-delete-dialog')
  await clickByTestId('ssh-delete-confirm-button')

  await browser.waitUntil(
    async () =>
      browser.execute(id => document.querySelector(`[data-testid="ssh-profile-card-${id}"]`) === null, existingProfileId),
    {
      timeout: 30000,
      interval: 200,
      timeoutMsg: `Expected existing profile '${existingProfileId}' to be removed before test setup`
    }
  )
}

export async function removeAllProfilesBeforeSetup() {
  await deleteAllProfiles()
}

export async function assertProfileCardMatches(profileId) {
  const cardText = (await readTextByTestId(`ssh-profile-card-${profileId}`)).replace(/\s+/g, ' ')
  if (!cardText.includes(`${targetUsername}@${targetHost}:${targetSshPort}`)) {
    throw new Error(`Expected SSH target summary to include '${targetUsername}@${targetHost}:${targetSshPort}'`)
  }
  if (!cardText.includes(`ralphd:${targetRalphdPort}`)) {
    throw new Error(`Expected profile to include ralphd port '${targetRalphdPort}'`)
  }
}

export async function waitForText(text, timeout = 30000) {
  await browser.waitUntil(
    async () => browser.execute(value => document.body?.textContent?.includes(value) === true, text),
    {
      timeout,
      interval: 200,
      timeoutMsg: `Expected screen text '${text}'`
    }
  )
}

export async function addProfile({
  name,
  host,
  username,
  sshPort,
  ralphdPort,
  authMode = 'key',
  password,
  identityFile
}) {
  await clickByTestId('ssh-new-profile-button')
  await waitForTestId('ssh-profile-editor')

  await setInputByTestId('ssh-profile-name-input', name)
  await setInputByTestId('ssh-host-input', host)
  await setInputByTestId('ssh-username-input', username)
  await setInputByTestId('ssh-port-input', sshPort)
  await setInputByTestId('ralphd-port-input', ralphdPort)

  if (authMode === 'password') {
    await clickByTestId('ssh-auth-mode-select')
    await clickByTestId('ssh-auth-mode-password')
    if (password) {
      await setInputByTestId('ssh-password-input', password)
    }
  }

  if (authMode === 'key' && identityFile) {
    await setInputByTestId('ssh-identity-file-input', identityFile)
  }

  await clickByTestId('ssh-profile-save-button')
}

export async function waitForProfiles(expectedCount) {
  await waitForProfileCount(expectedCount)
}
