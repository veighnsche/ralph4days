import type { Meta, StoryObj } from "@storybook/react";
import type { Task } from "@/types/prd";
import { PRDBody } from "./PRDBody";

const meta = {
  title: "Components/PRD/PRDBody",
  component: PRDBody,
  tags: ["autodocs"],
  args: {
    totalTasks: 0,
    onTaskClick: () => {},
    onClearFilters: () => {},
    onBraindump: () => {},
    onYap: () => {},
  },
} satisfies Meta<typeof PRDBody>;

export default meta;
type Story = StoryObj<typeof meta>;

const mockTasks: Task[] = [
  {
    id: 1,
    feature: "authentication",
    discipline: "backend",
    title: "Implement login API",
    description: "Create REST API endpoints for user authentication",
    status: "done",
    inferredStatus: "done",
    priority: "high",
    tags: ["api", "security"],
    dependsOn: [],
    created: "2026-02-01",
    featureDisplayName: "Authentication",
    featureAcronym: "AUTH",
    disciplineDisplayName: "Backend",
    disciplineAcronym: "BKND",
    disciplineIcon: "server",
    disciplineColor: "#8B5CF6",
  },
  {
    id: 2,
    feature: "authentication",
    discipline: "frontend",
    title: "Build login form",
    description: "Create UI for user login",
    status: "in_progress",
    inferredStatus: "in_progress",
    priority: "medium",
    tags: ["ui"],
    dependsOn: [1],
    featureDisplayName: "Authentication",
    featureAcronym: "AUTH",
    disciplineDisplayName: "Frontend",
    disciplineAcronym: "FRNT",
    disciplineIcon: "code",
    disciplineColor: "#3B82F6",
  },
  {
    id: 3,
    feature: "user-profile",
    discipline: "backend",
    title: "Profile API endpoints",
    status: "pending",
    inferredStatus: "ready",
    priority: "low",
    tags: [],
    dependsOn: [],
    featureDisplayName: "User Profile",
    featureAcronym: "USER",
    disciplineDisplayName: "Backend",
    disciplineAcronym: "BKND",
    disciplineIcon: "server",
    disciplineColor: "#8B5CF6",
  },
];

export const WithTasks: Story = {
  args: {
    filteredTasks: mockTasks,
    totalTasks: mockTasks.length,
  },
};

export const Empty: Story = {
  args: {
    filteredTasks: [],
  },
};
