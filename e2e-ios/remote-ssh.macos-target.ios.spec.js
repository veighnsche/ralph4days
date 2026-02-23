import { runStage00SshScreen } from './macos-target-stages/00_ssh-screen.stage.js'
import { runStage01APrecleanAndAddPasswordConfig } from './macos-target-stages/01A_preclean-add-password-config.stage.js'
import { runStage01BPrecleanAndAddProfileVariants } from './macos-target-stages/01B_preclean-add-profile-variants.stage.js'
import { runStage02ConnectToWelcome } from './macos-target-stages/02_connect-welcome.stage.js'
import { switchToWebViewContext } from './remote-ssh.harness.js'

describe('iOS remote SSH macOS target setup harness', () => {
  before(async () => {
    await switchToWebViewContext()
  })

  it('00_shows the SSH configuration screen first', async () => {
    await runStage00SshScreen()
  })

  it('01A_removes all profiles then adds one password profile', async () => {
    await runStage01APrecleanAndAddPasswordConfig()
  })

  it('01B_removes all profiles then adds profile variants for UI inspection', async () => {
    await runStage01BPrecleanAndAddProfileVariants()
  })

  it('02_starts with SSH already configured and lands on welcome project-select screen', async () => {
    await runStage02ConnectToWelcome()
  })
})
