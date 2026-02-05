import type { Meta, StoryObj } from "@storybook/react";
import { useState } from "react";
import { LoopCountBadge } from "./LoopCountBadge";

const meta = {
  title: "Components/LoopCountBadge",
  component: LoopCountBadge,
  tags: ["autodocs"],
} satisfies Meta<typeof LoopCountBadge>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Idle: Story = {
  render: () => {
    const [maxIterations, setMaxIterations] = useState(50);
    return (
      <LoopCountBadge
        status={{ state: "idle", current_iteration: 0, max_iterations: 50 }}
        maxIterations={maxIterations}
        setMaxIterations={setMaxIterations}
      />
    );
  },
};

export const Running: Story = {
  render: () => {
    const [maxIterations, setMaxIterations] = useState(50);
    return (
      <LoopCountBadge
        status={{ state: "running", current_iteration: 15, max_iterations: 50 }}
        maxIterations={maxIterations}
        setMaxIterations={setMaxIterations}
      />
    );
  },
};

export const NearComplete: Story = {
  render: () => {
    const [maxIterations, setMaxIterations] = useState(50);
    return (
      <LoopCountBadge
        status={{ state: "running", current_iteration: 48, max_iterations: 50 }}
        maxIterations={maxIterations}
        setMaxIterations={setMaxIterations}
      />
    );
  },
};

export const SmallCount: Story = {
  render: () => {
    const [maxIterations, setMaxIterations] = useState(5);
    return (
      <LoopCountBadge
        status={{ state: "idle", current_iteration: 0, max_iterations: 5 }}
        maxIterations={maxIterations}
        setMaxIterations={setMaxIterations}
      />
    );
  },
};

export const LargeCount: Story = {
  render: () => {
    const [maxIterations, setMaxIterations] = useState(999);
    return (
      <LoopCountBadge
        status={{ state: "idle", current_iteration: 0, max_iterations: 999 }}
        maxIterations={maxIterations}
        setMaxIterations={setMaxIterations}
      />
    );
  },
};
