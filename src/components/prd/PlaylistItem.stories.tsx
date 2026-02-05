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
  id: 1,
  feature: "ui",
  discipline: "frontend",
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
      id: 2,
      feature: "api",
      discipline: "backend",
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
      id: 3,
      feature: "data",
      discipline: "database",
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
      id: 4,
      feature: "tests",
      discipline: "testing",
      title: "Add authentication middleware",
      status: "blocked",
      blocked_by: "Waiting for security review",
    },
    onClick: () => console.log("Task clicked"),
  },
};

export const Skipped: Story = {
  args: {
    task: {
      ...baseTask,
      id: 5,
      feature: "deploy",
      discipline: "infra",
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
      id: 6,
      feature: "sec",
      discipline: "security",
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
      depends_on: [2, 3],
      tags: ["component", "ui", "depends-on-others"],
    },
    onClick: () => console.log("Task clicked"),
  },
};

export const NoDescription: Story = {
  args: {
    task: {
      id: 7,
      feature: "docs",
      discipline: "docs",
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
      <PlaylistItem
        task={{ ...baseTask, id: 1, feature: "ui", discipline: "frontend", title: "Frontend Task" }}
        onClick={() => {}}
      />
      <PlaylistItem
        task={{ ...baseTask, id: 2, feature: "api", discipline: "backend", title: "Backend Task" }}
        onClick={() => {}}
      />
      <PlaylistItem
        task={{ ...baseTask, id: 3, feature: "data", discipline: "database", title: "Database Task" }}
        onClick={() => {}}
      />
      <PlaylistItem
        task={{ ...baseTask, id: 4, feature: "tests", discipline: "testing", title: "Testing Task" }}
        onClick={() => {}}
      />
      <PlaylistItem
        task={{ ...baseTask, id: 5, feature: "deploy", discipline: "infra", title: "Infrastructure Task" }}
        onClick={() => {}}
      />
      <PlaylistItem
        task={{ ...baseTask, id: 6, feature: "sec", discipline: "security", title: "Security Task" }}
        onClick={() => {}}
      />
      <PlaylistItem
        task={{ ...baseTask, id: 7, feature: "docs", discipline: "docs", title: "Documentation Task" }}
        onClick={() => {}}
      />
      <PlaylistItem
        task={{ ...baseTask, id: 8, feature: "ui", discipline: "design", title: "Design Task" }}
        onClick={() => {}}
      />
      <PlaylistItem
        task={{ ...baseTask, id: 9, feature: "campaign", discipline: "promo", title: "Marketing Task" }}
        onClick={() => {}}
      />
      <PlaylistItem
        task={{ ...baseTask, id: 10, feature: "rest", discipline: "api", title: "API Task" }}
        onClick={() => {}}
      />
    </div>
  ),
};
