import {
  clickByTestId,
  deleteAllProfiles,
  listProfileIds,
  saveScreenshot,
  setInputByTestId,
  switchToWebViewContext,
  waitForProfileCount,
  waitForTestId
} from './remote-ssh.harness.js'

describe('iOS remote SSH Appium harness', () => {
  before(async () => {
    await switchToWebViewContext()
    await waitForTestId('ssh-connections-panel', 120000)
  })

  it('00_drives profile CRUD flow and captures screenshots', async () => {
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
