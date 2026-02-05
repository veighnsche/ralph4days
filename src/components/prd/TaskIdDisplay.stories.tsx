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
  },
};

export const Backend: Story = {
  args: {
    taskId: "api/backend/042",
    variant: "default",
  },
};

export const Database: Story = {
  args: {
    taskId: "data/database/3",
    variant: "default",
  },
};

export const Testing: Story = {
  args: {
    taskId: "tests/testing/015",
    variant: "default",
  },
};

export const BadgeVariant: Story = {
  args: {
    taskId: "ui/frontend/1",
    variant: "badge",
  },
};

export const AllDisciplines: Story = {
  render: () => (
    <div className="flex flex-col gap-4">
      <TaskIdDisplay taskId="ui/frontend/1" />
      <TaskIdDisplay taskId="api/backend/2" />
      <TaskIdDisplay taskId="data/database/3" />
      <TaskIdDisplay taskId="tests/testing/4" />
      <TaskIdDisplay taskId="infra/infrastructure/5" />
      <TaskIdDisplay taskId="sec/security/6" />
      <TaskIdDisplay taskId="docs/documentation/7" />
      <TaskIdDisplay taskId="design/design/8" />
      <TaskIdDisplay taskId="promo/marketing/9" />
      <TaskIdDisplay taskId="rest/api/010" />
    </div>
  ),
};
