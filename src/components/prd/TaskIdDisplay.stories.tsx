import type { Meta, StoryObj } from "@storybook/react";
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

export const Frontend: Story = {
  args: {
    taskId: "ui/frontend/1",
    variant: "default",
    status: "pending",
  },
};

export const Backend: Story = {
  args: {
    taskId: "api/backend/042",
    variant: "default",
    status: "in_progress",
  },
};

export const Database: Story = {
  args: {
    taskId: "data/database/3",
    variant: "default",
    status: "done",
  },
};

export const Testing: Story = {
  args: {
    taskId: "tests/testing/015",
    variant: "default",
    status: "blocked",
  },
};

export const BadgeVariant: Story = {
  args: {
    taskId: "ui/frontend/1",
    variant: "badge",
    status: "in_progress",
  },
};

export const AllDisciplines: Story = {
  render: () => (
    <div className="flex flex-col gap-4">
      <TaskIdDisplay taskId="ui/frontend/1" status="pending" />
      <TaskIdDisplay taskId="api/backend/2" status="in_progress" />
      <TaskIdDisplay taskId="data/database/3" status="done" />
      <TaskIdDisplay taskId="tests/testing/4" status="blocked" />
      <TaskIdDisplay taskId="infra/infrastructure/5" status="skipped" />
      <TaskIdDisplay taskId="sec/security/6" status="in_progress" />
      <TaskIdDisplay taskId="docs/documentation/7" status="pending" />
      <TaskIdDisplay taskId="design/design/8" status="done" />
      <TaskIdDisplay taskId="promo/marketing/9" status="pending" />
      <TaskIdDisplay taskId="rest/api/010" status="in_progress" />
    </div>
  ),
};
