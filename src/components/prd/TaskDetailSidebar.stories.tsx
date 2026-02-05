import type { Meta, StoryObj } from "@storybook/react";
import type { PRDTask } from "@/types/prd";
import { TaskDetailSidebar } from "./TaskDetailSidebar";

const meta = {
  title: "Components/PRD/TaskDetailSidebar",
  component: TaskDetailSidebar,
  tags: ["autodocs"],
  args: {
    open: true,
    onClose: () => {},
    onNavigateNext: () => {},
    onNavigatePrev: () => {},
  },
} satisfies Meta<typeof TaskDetailSidebar>;

export default meta;
type Story = StoryObj<typeof meta>;

const baseTask: PRDTask = {
  id: 2,
  feature: "authentication",
  discipline: "frontend",
  title: "Build login form",
  description: "Create UI for user login with email and password fields",
  status: "in_progress",
  priority: "medium",
  tags: ["ui", "forms"],
  depends_on: [1],
  acceptance_criteria: ["Form validates input", "Displays error messages", "Redirects on success"],
  created: "2026-02-01",
  updated: "2026-02-02",
};

export const InProgress: Story = {
  args: {
    task: baseTask,
  },
};

export const Pending: Story = {
  args: {
    task: {
      ...baseTask,
      status: "pending",
      description: undefined,
    },
  },
};

export const Done: Story = {
  args: {
    task: {
      ...baseTask,
      status: "done",
      completed: "2026-02-03T14:30:00Z",
    },
  },
};

export const Blocked: Story = {
  args: {
    task: {
      ...baseTask,
      status: "blocked",
      blocked_by: "Waiting for API credentials from backend team. Expected by end of week.",
    },
  },
};

export const HighPriority: Story = {
  args: {
    task: {
      ...baseTask,
      priority: "high",
    },
  },
};

export const Critical: Story = {
  args: {
    task: {
      ...baseTask,
      priority: "critical",
      status: "blocked",
      blocked_by: "Security vulnerability found in authentication flow. Requires immediate fix.",
      tags: ["security", "critical", "auth"],
    },
  },
};

export const MinimalTask: Story = {
  args: {
    task: {
      id: 10,
      feature: "search",
      discipline: "backend",
      title: "Add search endpoint",
      status: "pending",
      tags: [],
      depends_on: [],
    },
  },
};

export const WithAllFields: Story = {
  args: {
    task: {
      id: 5,
      feature: "user-profile",
      discipline: "fullstack",
      title: "Implement profile editing",
      description:
        "Allow users to edit their profile information including name, email, avatar, bio, and preferences. Changes should be persisted to database and reflected immediately in UI.",
      status: "in_progress",
      priority: "high",
      tags: ["profile", "forms", "validation", "database"],
      depends_on: [3, 4],
      acceptance_criteria: [
        "User can edit all profile fields",
        "Changes are validated before submission",
        "Successful changes show confirmation message",
        "Failed changes show clear error messages",
        "Avatar upload supports common image formats",
        "Changes persist across sessions",
      ],
      created: "2026-01-28",
      updated: "2026-02-03",
    },
  },
};

export const NoNavigation: Story = {
  args: {
    task: baseTask,
    onNavigateNext: undefined,
    onNavigatePrev: undefined,
  },
};
