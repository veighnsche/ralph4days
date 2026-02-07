import type { Meta, StoryObj } from '@storybook/react'
import { NavigationMenu } from './NavigationMenu'

const meta = {
  title: 'Components/NavigationMenu',
  component: NavigationMenu,
  tags: ['autodocs'],
  args: {
    onPageChange: () => {}
  }
} satisfies Meta<typeof NavigationMenu>

export default meta
type Story = StoryObj<typeof meta>

export const TasksPage: Story = {
  args: {
    currentPage: 'tasks'
  }
}

export const FeaturesPage: Story = {
  args: {
    currentPage: 'features'
  }
}

export const DisciplinesPage: Story = {
  args: {
    currentPage: 'disciplines'
  }
}
