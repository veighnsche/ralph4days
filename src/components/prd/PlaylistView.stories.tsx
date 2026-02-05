import type { Meta, StoryObj } from "@storybook/react";
import type { PRDTask } from "@/types/prd";
import { PlaylistView } from "./PlaylistView";

const meta = {
  title: "Components/PRD/PlaylistView",
  component: PlaylistView,
  tags: ["autodocs"],
  args: {
    onTaskClick: () => {},
  },
} satisfies Meta<typeof PlaylistView>;

export default meta;
type Story = StoryObj<typeof meta>;

const mockTasks: PRDTask[] = [
  {
    id: 1,
    feature: "authentication",
    discipline: "backend",
    title: "Implement login API",
    description: "Create REST API endpoints for user authentication",
    status: "done",
    priority: "high",
    tags: ["api", "security"],
    depends_on: [],
    created: "2026-02-01",
  },
  {
    id: 2,
    feature: "authentication",
    discipline: "frontend",
    title: "Build login form",
    description: "Create UI for user login",
    status: "in_progress",
    priority: "medium",
    tags: ["ui"],
    depends_on: [1],
  },
  {
    id: 3,
    feature: "user-profile",
    discipline: "backend",
    title: "Profile API endpoints",
    status: "pending",
    priority: "low",
    tags: [],
    depends_on: [],
  },
  {
    id: 4,
    feature: "user-profile",
    discipline: "frontend",
    title: "Profile page UI",
    status: "pending",
    priority: "medium",
    tags: ["ui"],
    depends_on: [3],
  },
];

export const Default: Story = {
  args: {
    tasks: mockTasks,
  },
};

export const WithBlockedTasks: Story = {
  args: {
    tasks: [
      ...mockTasks,
      {
        id: 5,
        feature: "payments",
        discipline: "backend",
        title: "Integrate payment gateway",
        status: "blocked",
        blocked_by: "Waiting for API credentials from payment provider",
        priority: "critical",
        tags: ["payments"],
        depends_on: [],
      },
      {
        id: 6,
        feature: "notifications",
        discipline: "backend",
        title: "Email notifications",
        status: "skipped",
        priority: "low",
        tags: ["email"],
        depends_on: [],
      },
    ],
  },
};

export const AllDone: Story = {
  args: {
    tasks: mockTasks.map((task) => ({ ...task, status: "done" as const })),
  },
};

export const AllPending: Story = {
  args: {
    tasks: mockTasks.map((task) => ({ ...task, status: "pending" as const })),
  },
};

export const Empty: Story = {
  args: {
    tasks: [],
  },
};
