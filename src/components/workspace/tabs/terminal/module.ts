import { defineWorkspaceTabModule } from '../contracts'
import { TerminalTabContent } from './content'
import { createTerminalTab } from './factory'
import { parseTerminalTabParams } from './schema'

export const terminalTabModule = defineWorkspaceTabModule({
  type: 'terminal',
  component: TerminalTabContent,
  parseParams: parseTerminalTabParams,
  createTab: createTerminalTab,
  keepAliveOnDeactivate: true
})
