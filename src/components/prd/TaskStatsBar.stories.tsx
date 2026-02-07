import type { Meta, StoryObj } from '@storybook/react'
import { TaskStatsBar } from './TaskStatsBar'

const meta = {
  title: 'PRD/TaskStatsBar',
  component: TaskStatsBar,
  parameters: {
    layout: 'padded'
  },
  tags: ['autodocs']
} satisfies Meta<typeof TaskStatsBar>

export default meta
type Story = StoryObj<typeof meta>

export const Empty: Story = {
  args: {
    totalTasks: 0,
    doneTasks: 0,
    progressPercent: 0
  }
}

export const JustStarted: Story = {
  args: {
    totalTasks: 20,
    doneTasks: 2,
    progressPercent: 10
  }
}

export const HalfwayDone: Story = {
  args: {
    totalTasks: 50,
    doneTasks: 25,
    progressPercent: 50
  }
}

export const AlmostComplete: Story = {
  args: {
    totalTasks: 30,
    doneTasks: 28,
    progressPercent: 93
  }
}

export const Complete: Story = {
  args: {
    totalTasks: 15,
    doneTasks: 15,
    progressPercent: 100
  }
}

export const LargeProject: Story = {
  args: {
    totalTasks: 247,
    doneTasks: 189,
    progressPercent: 77
  }
}
