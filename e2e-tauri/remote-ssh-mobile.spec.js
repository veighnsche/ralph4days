import {
  clearInputByTestId,
  clickElementByTestId,
  ensureRemoteSshPanelReady,
  fillInputByTestId,
  listProfileIds,
  saveE2EScreenshot,
  waitForProfileCount,
  waitForTestId
} from './workspace.harness.js'

async function deleteAllProfiles() {
  const ids = await listProfileIds()
  for (const id of ids) {
    await clickElementByTestId(`ssh-profile-delete-${id}`)
    await waitForTestId('ssh-delete-dialog')
    await clickElementByTestId('ssh-delete-confirm-button')
  }
  await waitForProfileCount(0)
}

describe('Remote SSH mobile UI harness', () => {
  beforeEach(async () => {
    await ensureRemoteSshPanelReady()
    await deleteAllProfiles()
  })

  it('drives profile actions and captures state screenshots', async () => {
    await saveE2EScreenshot('remote-ssh-00-empty')

    await clickElementByTestId('ssh-new-profile-button')
    await waitForTestId('ssh-profile-editor')

    await fillInputByTestId('ssh-profile-name-input', 'E2E Remote Host')
    await fillInputByTestId('ssh-host-input', '127.0.0.1')
    await fillInputByTestId('ssh-username-input', 'ralph')
    await fillInputByTestId('ssh-port-input', '22')
    await fillInputByTestId('ralphd-port-input', '9944')

    await saveE2EScreenshot('remote-ssh-01-editor-filled')
    await clickElementByTestId('ssh-profile-save-button')
    await waitForProfileCount(1)
    await saveE2EScreenshot('remote-ssh-02-profile-created')

    const [profileId] = await listProfileIds()
    expect(profileId).toBeTruthy()

    await clickElementByTestId(`ssh-profile-edit-${profileId}`)
    await waitForTestId('ssh-profile-editor')
    await saveE2EScreenshot('remote-ssh-03-edit-dialog')
    await clickElementByTestId('ssh-profile-cancel-button')

    await waitForTestId('ssh-search-input')
    await fillInputByTestId('ssh-search-input', 'missing-profile')
    await saveE2EScreenshot('remote-ssh-04-search-empty')
    await clearInputByTestId('ssh-search-input')

    await clickElementByTestId('ssh-quick-connect-button')
    await waitForTestId('ssh-connect-dialog')
    await saveE2EScreenshot('remote-ssh-05-connect-dialog')
    await clickElementByTestId('ssh-connect-cancel-button')

    await clickElementByTestId(`ssh-profile-delete-${profileId}`)
    await waitForTestId('ssh-delete-dialog')
    await saveE2EScreenshot('remote-ssh-06-delete-dialog')
    await clickElementByTestId('ssh-delete-confirm-button')

    await waitForProfileCount(0)
    await saveE2EScreenshot('remote-ssh-07-empty-after-delete')
  })
})
