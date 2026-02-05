import type { Meta, StoryObj } from "@storybook/react";
import type { PRDTask } from "@/types/prd";
import { PlaylistItem } from "./PlaylistItem";

const meta = {
  title: "PRD/PlaylistItem",
  component: PlaylistItem,
  parameters: {
    layout: "padded",
  },
  tags: ["autodocs"],
  decorators: [
    (Story) => (
      <div className="max-w-4xl">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof PlaylistItem>;

export default meta;
type Story = StoryObj<typeof meta>;

const baseTask: PRDTask = {
  id: "ui/frontend/001",
  title: "Design main dashboard layout",
  description: "Create responsive dashboard with sidebar and main content area",
  status: "pending",
  priority: "high",
  tags: ["design", "layout"],
  created: "2026-02-01",
  updated: "2026-02-05",
  acceptance_criteria: ["Responsive on mobile, tablet, desktop", "Sidebar collapses on mobile", "Dark mode support"],
};

export const Pending: Story = {
  args: {
    task: baseTask,
    onClick: () => console.log("Task clicked"),
  },
};

export const InProgress: Story = {
  args: {
    task: {
      ...baseTask,
      id: "api/backend/002",
      title: "Implement task list component",
      status: "in_progress",
    },
    isNowPlaying: true,
    onClick: () => console.log("Task clicked"),
  },
};

export const Done: Story = {
  args: {
    task: {
      ...baseTask,
      id: "data/database/003",
      title: "Setup REST API endpoints",
      status: "done",
      completed: "2026-02-03",
    },
    onClick: () => console.log("Task clicked"),
  },
};

export const Blocked: Story = {
  args: {
    task: {
      ...baseTask,
      id: "tests/testing/004",
      title: "Add authentication middleware",
      status: "blocked",
      blocked_by: "Waiting for security review",
    },
    isIssue: true,
    onClick: () => console.log("Task clicked"),
  },
};

export const Skipped: Story = {
  args: {
    task: {
      ...baseTask,
      id: "infra/infrastructure/005",
      title: "Deploy to staging environment",
      status: "skipped",
    },
    onClick: () => console.log("Task clicked"),
  },
};

export const LowPriority: Story = {
  args: {
    task: {
      ...baseTask,
      title: "Update documentation",
      priority: "low",
    },
    onClick: () => console.log("Task clicked"),
  },
};

export const CriticalPriority: Story = {
  args: {
    task: {
      ...baseTask,
      id: "sec/security/006",
      title: "Fix critical security vulnerability",
      priority: "critical",
      status: "in_progress",
    },
    isNowPlaying: true,
    onClick: () => console.log("Task clicked"),
  },
};

export const WithDependencies: Story = {
  args: {
    task: {
      ...baseTask,
      title: "Add task detail sidebar",
      depends_on: ["ui/frontend/002", "ui/frontend/003"],
      tags: ["component", "ui", "depends-on-others"],
    },
    onClick: () => console.log("Task clicked"),
  },
};

export const NoDescription: Story = {
  args: {
    task: {
      id: "docs/documentation/007",
      title: "Write API documentation",
      status: "pending",
      priority: "medium",
    },
    onClick: () => console.log("Task clicked"),
  },
};

export const AllDisciplines: Story = {
  render: () => (
    <div className="flex flex-col gap-2">
      <PlaylistItem task={{ ...baseTask, id: "ui/frontend/001", title: "Frontend Task" }} onClick={() => {}} />
      <PlaylistItem task={{ ...baseTask, id: "api/backend/002", title: "Backend Task" }} onClick={() => {}} />
      <PlaylistItem task={{ ...baseTask, id: "data/database/003", title: "Database Task" }} onClick={() => {}} />
      <PlaylistItem task={{ ...baseTask, id: "tests/testing/004", title: "Testing Task" }} onClick={() => {}} />
      <PlaylistItem
        task={{ ...baseTask, id: "infra/infrastructure/005", title: "Infrastructure Task" }}
        onClick={() => {}}
      />
      <PlaylistItem task={{ ...baseTask, id: "sec/security/006", title: "Security Task" }} onClick={() => {}} />
      <PlaylistItem
        task={{ ...baseTask, id: "docs/documentation/007", title: "Documentation Task" }}
        onClick={() => {}}
      />
      <PlaylistItem task={{ ...baseTask, id: "design/design/008", title: "Design Task" }} onClick={() => {}} />
      <PlaylistItem task={{ ...baseTask, id: "promo/marketing/009", title: "Marketing Task" }} onClick={() => {}} />
      <PlaylistItem task={{ ...baseTask, id: "rest/api/010", title: "API Task" }} onClick={() => {}} />
    </div>
  ),
};
