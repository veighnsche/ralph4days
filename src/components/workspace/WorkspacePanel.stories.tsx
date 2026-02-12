import type { Meta, StoryObj } from '@storybook/react'
import { useEffect } from 'react'
import { createTerminalTab } from '@/components/workspace/tabs'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'
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
        openTab(createTerminalTab({ title: 'Terminal 1' }))
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
        openTab(createTerminalTab({ title: 'Terminal 1' }))
        openTab(createTerminalTab({ title: 'Terminal 2' }))
        openTab(createTerminalTab({ title: 'Terminal 3' }))
      }, [openTab])
      return <Story />
    }
  ]
}
