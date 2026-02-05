import type { Meta, StoryObj } from "@storybook/react";
import { useState } from "react";
import { DisciplineSelect } from "./DisciplineSelect";

const meta = {
  title: "PRD/DisciplineSelect",
  component: DisciplineSelect,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  decorators: [
    (Story) => (
      <div className="w-[400px]">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof DisciplineSelect>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: () => {
    const [value, setValue] = useState("");
    return <DisciplineSelect value={value} onChange={setValue} />;
  },
};

export const FrontendSelected: Story = {
  render: () => {
    const [value, setValue] = useState("frontend");
    return <DisciplineSelect value={value} onChange={setValue} />;
  },
};

export const BackendSelected: Story = {
  render: () => {
    const [value, setValue] = useState("backend");
    return <DisciplineSelect value={value} onChange={setValue} />;
  },
};

export const DatabaseSelected: Story = {
  render: () => {
    const [value, setValue] = useState("database");
    return <DisciplineSelect value={value} onChange={setValue} />;
  },
};
