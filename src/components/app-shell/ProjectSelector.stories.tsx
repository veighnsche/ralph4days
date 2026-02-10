import type { Meta, StoryObj } from '@storybook/react'
import { ProjectSelector } from './ProjectSelector'

const meta = {
  title: 'Components/ProjectSelector',
  component: ProjectSelector,
  tags: ['autodocs'],
  args: {
    onProjectSelected: () => {}
  },
  decorators: [
    Story => (
      <div className="h-screen w-screen">
        <Story />
      </div>
    )
  ]
} satisfies Meta<typeof ProjectSelector>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const BothLists: Story = {
  parameters: {
    mockData: [
      {
        command: 'get_recent_projects',
        response: [
          { name: 'my-app', path: '/home/user/projects/my-app', lastOpened: '2026-02-10T15:30:00Z' },
          { name: 'backend-api', path: '/home/user/projects/backend-api', lastOpened: '2026-02-09T12:00:00Z' }
        ]
      },
      {
        command: 'scan_for_ralph_projects',
        response: [
          { name: 'my-app', path: '/home/user/projects/my-app' },
          { name: 'backend-api', path: '/home/user/projects/backend-api' },
          { name: 'mobile-client', path: '/home/user/projects/mobile-client' },
          { name: 'data-pipeline', path: '/home/user/projects/data-pipeline' }
        ]
      }
    ]
  }
}

export const NoRecentWithDiscovered: Story = {
  parameters: {
    mockData: [
      {
        command: 'get_recent_projects',
        response: []
      },
      {
        command: 'scan_for_ralph_projects',
        response: Array.from({ length: 8 }, (_, i) => ({
          name: `project-${i + 1}`,
          path: `/home/user/projects/project-${i + 1}`
        }))
      }
    ]
  }
}

export const NoProjectsFound: Story = {
  parameters: {
    mockData: [
      {
        command: 'get_recent_projects',
        response: []
      },
      {
        command: 'scan_for_ralph_projects',
        response: []
      }
    ]
  }
}
