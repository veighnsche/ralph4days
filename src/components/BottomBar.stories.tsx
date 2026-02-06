import type { Meta, StoryObj } from "@storybook/react";
import { BottomBar } from "./BottomBar";

const meta = {
  title: "Components/BottomBar",
  component: BottomBar,
  tags: ["autodocs"],
  args: {
    lockedProject: "/home/user/projects/my-app",
    currentPage: "tasks" as const,
    onPageChange: () => {},
  },
} satisfies Meta<typeof BottomBar>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
