import { saveScreenshot, waitForTestId } from '../remote-ssh.harness.js'

export async function runStage00SshScreen() {
  await waitForTestId('ssh-connections-panel', 120000)
  await saveScreenshot('remote-ssh-macos-00-first-screen')
}
