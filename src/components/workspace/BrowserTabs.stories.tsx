import type { Meta, StoryObj } from '@storybook/react'
import { Terminal } from 'lucide-react'
import { useState } from 'react'
import type { BrowserTabsActions } from '@/hooks/workspace/useBrowserTabsActions'
import { type BrowserTab, BrowserTabs } from './BrowserTabs'

const meta = {
  title: 'Components/BrowserTabs',
  component: BrowserTabs,
  parameters: {
    layout: 'fullscreen'
  },
  tags: ['autodocs']
} satisfies Meta<typeof BrowserTabs>

export default meta
type Story = StoryObj<typeof meta>

function BrowserTabsDemo() {
  const [tabs, setTabs] = useState<BrowserTab[]>([
    { id: 'output', title: 'Output', icon: Terminal },
    { id: 'task-1', title: 'Create Authentication' },
    { id: 'task-2', title: 'Database Migration' },
    { id: 'task-3', title: 'API Endpoints' },
    { id: 'task-4', title: 'Frontend Components' }
  ])
  const [activeTabId, setActiveTabId] = useState('output')

  const actions: BrowserTabsActions = {
    switchTab: (tabId: string) => setActiveTabId(tabId),
    closeTab: (tabId: string) => {
      setTabs(tabs.filter(t => t.id !== tabId))
      if (activeTabId === tabId) {
        setActiveTabId(tabs[0]?.id || '')
      }
    },
    closeAll: () => {
      setTabs([])
      setActiveTabId('')
    },
    closeOthers: (tabId: string) => {
      setTabs(tabs.filter(t => t.id === tabId))
      setActiveTabId(tabId)
    },
    closeToRight: (tabId: string) => {
      const index = tabs.findIndex(t => t.id === tabId)
      if (index !== -1 && index < tabs.length - 1) {
        setTabs(tabs.slice(0, index + 1))
      }
    },
    newTabToRight: (afterTabId: string) => {
      const index = tabs.findIndex(t => t.id === afterTabId)
      const newTab: BrowserTab = {
        id: `new-${Date.now()}`,
        title: 'New Tab'
      }
      if (index !== -1) {
        const nextTabs = [...tabs.slice(0, index + 1), newTab, ...tabs.slice(index + 1)]
        setTabs(nextTabs)
        setActiveTabId(newTab.id)
      }
    },
    reorderTabs: (fromIndex: number, toIndex: number) => {
      const nextTabs = [...tabs]
      const [movedTab] = nextTabs.splice(fromIndex, 1)
      nextTabs.splice(toIndex, 0, movedTab)
      setTabs(nextTabs)
    }
  }

  return (
    <div className="h-screen flex flex-col dark bg-background text-foreground">
      <BrowserTabs tabs={tabs} activeTabId={activeTabId} actions={actions} />
      <div className="flex-1 bg-background p-4">
        <div className="space-y-2">
          <p className="text-muted-foreground text-sm">Active tab: {activeTabId}</p>
          <p className="text-muted-foreground text-xs">Right-click on a tab to see context menu options</p>
          <p className="text-muted-foreground text-xs">Drag and drop tabs to reorder them</p>
        </div>
      </div>
    </div>
  )
}

const mockActions: BrowserTabsActions = {
  switchTab: () => {},
  closeTab: () => {},
  closeAll: () => {},
  closeOthers: () => {},
  closeToRight: () => {},
  newTabToRight: () => {},
  reorderTabs: () => {}
}

export const Default: Story = {
  args: {
    tabs: [],
    activeTabId: '',
    actions: mockActions
  },
  render: () => <BrowserTabsDemo />
}

export const SingleTab: Story = {
  args: {
    tabs: [{ id: 'output', title: 'Output', icon: Terminal }],
    activeTabId: 'output',
    actions: mockActions
  }
}

export const ManyTabs: Story = {
  args: {
    tabs: [
      { id: '1', title: 'Output', icon: Terminal },
      { id: '2', title: 'Create User Authentication Module' },
      { id: '3', title: 'Database Schema Migration' },
      { id: '4', title: 'API Endpoint Implementation' },
      { id: '5', title: 'Frontend Components' }
    ],
    activeTabId: '2',
    actions: mockActions
  }
}

export const LongTitles: Story = {
  args: {
    tabs: [
      { id: '1', title: 'This is a very long tab title that should be truncated' },
      { id: '2', title: 'Another extremely long title for testing purposes' }
    ],
    activeTabId: '1',
    actions: mockActions
  }
}

export const NonCloseable: Story = {
  args: {
    tabs: [
      { id: '1', title: 'Permanent Tab', closeable: false },
      { id: '2', title: 'Closeable Tab', closeable: true }
    ],
    activeTabId: '1',
    actions: mockActions
  }
}
