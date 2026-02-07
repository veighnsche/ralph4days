import type { Meta, StoryObj } from '@storybook/react'
import { ProjectSelector } from './ProjectSelector'

const meta = {
  title: 'Components/ProjectSelector',
  component: ProjectSelector,
  tags: ['autodocs'],
  args: {
    onProjectSelected: () => {}
  }
} satisfies Meta<typeof ProjectSelector>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const WithMockProjects: Story = {
  parameters: {
    mockData: [
      {
        command: 'scan_for_ralph_projects',
        response: [
          { name: 'my-app', path: '/home/user/projects/my-app' },
          { name: 'backend-api', path: '/home/user/projects/backend-api' },
          { name: 'mobile-client', path: '/home/user/projects/mobile-client' }
        ]
      }
    ]
  }
}

export const NoProjectsFound: Story = {
  parameters: {
    mockData: [
      {
        command: 'scan_for_ralph_projects',
        response: []
      }
    ]
  }
}
