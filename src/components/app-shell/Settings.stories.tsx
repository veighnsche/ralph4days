import type { Meta, StoryObj } from '@storybook/react'
import { Settings } from './Settings'

const meta = {
  title: 'Components/Settings',
  component: Settings,
  args: { open: true, onOpenChange: () => {} },
  tags: ['autodocs']
} satisfies Meta<typeof Settings>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
