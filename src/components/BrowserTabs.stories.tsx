import type { Meta, StoryObj } from "@storybook/react";
import { useState } from "react";
import { type BrowserTab, BrowserTabs } from "./BrowserTabs";

const meta = {
  title: "Components/BrowserTabs",
  component: BrowserTabs,
  parameters: {
    layout: "fullscreen",
  },
  tags: ["autodocs"],
} satisfies Meta<typeof BrowserTabs>;

export default meta;
type Story = StoryObj<typeof meta>;

function BrowserTabsDemo() {
  const [tabs, setTabs] = useState<BrowserTab[]>([
    { id: "output", title: "Output" },
    { id: "task-1", title: "Create Authentication" },
    { id: "task-2", title: "Database Migration" },
  ]);
  const [activeTabId, setActiveTabId] = useState("output");

  const handleTabClose = (tabId: string) => {
    setTabs(tabs.filter((t) => t.id !== tabId));
    if (activeTabId === tabId) {
      setActiveTabId(tabs[0]?.id || "");
    }
  };

  return (
    <div className="h-screen flex flex-col">
      <BrowserTabs tabs={tabs} activeTabId={activeTabId} onTabChange={setActiveTabId} onTabClose={handleTabClose} />
      <div className="flex-1 bg-background p-4">
        <p className="text-muted-foreground">Active tab: {activeTabId}</p>
      </div>
    </div>
  );
}

export const Default: Story = {
  args: {
    tabs: [],
    activeTabId: "",
    onTabChange: () => {},
  },
  render: () => <BrowserTabsDemo />,
};

export const SingleTab: Story = {
  args: {
    tabs: [{ id: "output", title: "Output" }],
    activeTabId: "output",
    onTabChange: () => {},
    onTabClose: () => {},
  },
};

export const ManyTabs: Story = {
  args: {
    tabs: [
      { id: "1", title: "Output" },
      { id: "2", title: "Create User Authentication Module" },
      { id: "3", title: "Database Schema Migration" },
      { id: "4", title: "API Endpoint Implementation" },
      { id: "5", title: "Frontend Components" },
    ],
    activeTabId: "2",
    onTabChange: () => {},
    onTabClose: () => {},
  },
};

export const LongTitles: Story = {
  args: {
    tabs: [
      { id: "1", title: "This is a very long tab title that should be truncated" },
      { id: "2", title: "Another extremely long title for testing purposes" },
    ],
    activeTabId: "1",
    onTabChange: () => {},
    onTabClose: () => {},
  },
};
