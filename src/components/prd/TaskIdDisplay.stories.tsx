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
    taskId: "ui/frontend/001",
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
    taskId: "data/database/003",
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
    taskId: "ui/frontend/001",
    variant: "badge",
  },
};

export const AllDisciplines: Story = {
  render: () => (
    <div className="flex flex-col gap-4">
      <TaskIdDisplay taskId="ui/frontend/001" />
      <TaskIdDisplay taskId="api/backend/002" />
      <TaskIdDisplay taskId="data/database/003" />
      <TaskIdDisplay taskId="tests/testing/004" />
      <TaskIdDisplay taskId="infra/infrastructure/005" />
      <TaskIdDisplay taskId="sec/security/006" />
      <TaskIdDisplay taskId="docs/documentation/007" />
      <TaskIdDisplay taskId="design/design/008" />
      <TaskIdDisplay taskId="promo/marketing/009" />
      <TaskIdDisplay taskId="rest/api/010" />
    </div>
  ),
};
