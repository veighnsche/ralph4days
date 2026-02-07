import type { Meta, StoryObj } from '@storybook/react'
import { Settings } from './Settings'

const meta = {
  title: 'Components/Settings',
  component: Settings,
  tags: ['autodocs']
} satisfies Meta<typeof Settings>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
