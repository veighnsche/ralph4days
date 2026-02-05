import type { Meta, StoryObj } from "@storybook/react";
import { TaskCreateDialog } from "./TaskCreateDialog";

const meta = {
  title: "Components/PRD/TaskCreateDialog",
  component: TaskCreateDialog,
  tags: ["autodocs"],
  args: {
    onTaskCreated: () => {},
  },
} satisfies Meta<typeof TaskCreateDialog>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
