import type { Meta, StoryObj } from '@storybook/react'
import { useEffect } from 'react'
import { TerminalTabContent } from '@/components/workspace'
import { NOOP_TAB_LIFECYCLE, useWorkspaceStore } from '@/stores/useWorkspaceStore'
import { WorkspacePanel } from './WorkspacePanel'

const meta = {
  title: 'Components/WorkspacePanel',
  component: WorkspacePanel,
  tags: ['autodocs'],
  decorators: [
    Story => (
      <div className="h-[600px]">
        <Story />
      </div>
    )
  ]
} satisfies Meta<typeof WorkspacePanel>

export default meta
type Story = StoryObj<typeof meta>

export const Empty: Story = {
  decorators: [
    Story => {
      const closeAllExcept = useWorkspaceStore(state => state.closeAllExcept)
      useEffect(() => {
        closeAllExcept('__none__')
      }, [closeAllExcept])
      return <Story />
    }
  ]
}

export const WithTerminal: Story = {
  decorators: [
    Story => {
      const { openTab } = useWorkspaceStore()
      useEffect(() => {
        openTab({
          type: 'terminal',
          component: TerminalTabContent,
          title: 'Terminal 1',
          closeable: true,
          lifecycle: NOOP_TAB_LIFECYCLE
        })
      }, [openTab])
      return <Story />
    }
  ]
}

export const MultipleTerminals: Story = {
  decorators: [
    Story => {
      const { openTab } = useWorkspaceStore()
      useEffect(() => {
        openTab({
          type: 'terminal',
          component: TerminalTabContent,
          title: 'Terminal 1',
          closeable: true,
          lifecycle: NOOP_TAB_LIFECYCLE
        })
        openTab({
          type: 'terminal',
          component: TerminalTabContent,
          title: 'Terminal 2',
          closeable: true,
          lifecycle: NOOP_TAB_LIFECYCLE
        })
        openTab({
          type: 'terminal',
          component: TerminalTabContent,
          title: 'Terminal 3',
          closeable: true,
          lifecycle: NOOP_TAB_LIFECYCLE
        })
      }, [openTab])
      return <Story />
    }
  ]
}
