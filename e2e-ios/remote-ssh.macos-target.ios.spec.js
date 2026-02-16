import { runStage00SshScreen } from './macos-target-stages/00_ssh-screen.stage.js'
import { runStage01PrecleanAndAddConfig } from './macos-target-stages/01_preclean-add-config.stage.js'
import { runStage02ConnectToWelcome } from './macos-target-stages/02_connect-welcome.stage.js'
import { switchToWebViewContext } from './remote-ssh.harness.js'

describe('iOS remote SSH macOS target setup harness', () => {
  before(async () => {
    await switchToWebViewContext()
  })

  it('00_shows the SSH configuration screen first', async () => {
    await runStage00SshScreen()
  })

  it('01_removes existing matching config if present, then adds macOS SSH config', async () => {
    await runStage01PrecleanAndAddConfig()
  })

  it('02_starts with SSH already configured and lands on welcome project-select screen', async () => {
    await runStage02ConnectToWelcome()
  })
})
