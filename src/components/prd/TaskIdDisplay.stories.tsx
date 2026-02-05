import type { Meta, StoryObj } from "@storybook/react";
import type { PRDTask } from "@/types/prd";
import { TaskIdDisplay } from "./TaskIdDisplay";

const meta = {
  title: "PRD/TaskIdDisplay",
  component: TaskIdDisplay,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
} satisfies Meta<typeof TaskIdDisplay>;

export default meta;
type Story = StoryObj<typeof meta>;

const createTask = (id: number, feature: string, discipline: PRDTask["discipline"]): PRDTask => ({
  id,
  feature,
  discipline,
  title: "Example task",
  status: "pending",
});

export const Frontend: Story = {
  args: {
    task: createTask(1, "ui", "frontend"),
    variant: "default",
  },
};

export const Backend: Story = {
  args: {
    task: createTask(42, "api", "backend"),
    variant: "default",
  },
};

export const Database: Story = {
  args: {
    task: createTask(3, "data", "database"),
    variant: "default",
  },
};

export const Testing: Story = {
  args: {
    task: createTask(15, "tests", "testing"),
    variant: "default",
  },
};

export const BadgeVariant: Story = {
  args: {
    task: createTask(1, "ui", "frontend"),
    variant: "badge",
  },
};

export const HighId: Story = {
  args: {
    task: createTask(999, "authentication", "backend"),
    variant: "default",
  },
};

export const VeryHighId: Story = {
  args: {
    task: createTask(1000, "user-profile", "frontend"),
    variant: "default",
  },
};

export const MassiveId: Story = {
  args: {
    task: createTask(12345, "payments", "backend"),
    variant: "default",
  },
};

export const IdProgression: Story = {
  args: {
    task: createTask(1, "authentication", "backend"),
  },
  render: () => (
    <div className="flex flex-col gap-4">
      <div className="flex flex-col gap-1">
        <span className="text-xs text-muted-foreground">ID #001 (1-999 with # prefix)</span>
        <TaskIdDisplay task={createTask(1, "authentication", "backend")} />
      </div>
      <div className="flex flex-col gap-1">
        <span className="text-xs text-muted-foreground">ID #042</span>
        <TaskIdDisplay task={createTask(42, "user-profile", "frontend")} />
      </div>
      <div className="flex flex-col gap-1">
        <span className="text-xs text-muted-foreground">ID #999 (max with #)</span>
        <TaskIdDisplay task={createTask(999, "payments", "backend")} />
      </div>
      <div className="flex flex-col gap-1">
        <span className="text-xs text-muted-foreground">ID 1000 (no # for 1000+)</span>
        <TaskIdDisplay task={createTask(1000, "search", "backend")} />
      </div>
      <div className="flex flex-col gap-1">
        <span className="text-xs text-muted-foreground">ID 12345</span>
        <TaskIdDisplay task={createTask(12345, "notifications", "backend")} />
      </div>
    </div>
  ),
};

export const AllDisciplines: Story = {
  args: {
    task: createTask(1, "ui", "frontend"),
  },
  render: () => (
    <div className="flex flex-col gap-4">
      <TaskIdDisplay task={createTask(1, "ui", "frontend")} />
      <TaskIdDisplay task={createTask(2, "api", "backend")} />
      <TaskIdDisplay task={createTask(3, "data", "database")} />
      <TaskIdDisplay task={createTask(4, "tests", "testing")} />
      <TaskIdDisplay task={createTask(5, "deploy", "infra")} />
      <TaskIdDisplay task={createTask(6, "sec", "security")} />
      <TaskIdDisplay task={createTask(7, "docs", "docs")} />
      <TaskIdDisplay task={createTask(8, "ui", "design")} />
      <TaskIdDisplay task={createTask(9, "campaign", "wiring")} />
      <TaskIdDisplay task={createTask(10, "rest", "api")} />
    </div>
  ),
};
