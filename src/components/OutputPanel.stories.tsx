import type { Meta, StoryObj } from "@storybook/react";
import { useEffect } from "react";
import { useLoopStore } from "@/stores/useLoopStore";
import { OutputPanel } from "./OutputPanel";

const meta = {
  title: "Components/OutputPanel",
  component: OutputPanel,
  tags: ["autodocs"],
  decorators: [
    (Story) => (
      <div className="h-[600px]">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof OutputPanel>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Empty: Story = {
  decorators: [
    (Story) => {
      const clearOutput = useLoopStore((state) => state.clearOutput);
      useEffect(() => {
        clearOutput();
      }, [clearOutput]);
      return <Story />;
    },
  ],
};

export const WithOutput: Story = {
  decorators: [
    (Story) => {
      const { addOutput, clearOutput } = useLoopStore();
      useEffect(() => {
        clearOutput();
        addOutput("Starting loop...", "info");
        addOutput("Running iteration 1", "output");
        addOutput("Task #1 completed successfully", "success");
        addOutput("Warning: API rate limit approaching", "info");
        addOutput("Running iteration 2", "output");
      }, [addOutput, clearOutput]);
      return <Story />;
    },
  ],
};

export const WithError: Story = {
  decorators: [
    (Story) => {
      const { addOutput, clearOutput } = useLoopStore();
      useEffect(() => {
        clearOutput();
        addOutput("Starting loop...", "info");
        addOutput("Running iteration 1", "output");
        addOutput("Error: Failed to connect to API", "error");
        addOutput("Retrying...", "info");
      }, [addOutput, clearOutput]);
      return <Story />;
    },
  ],
};

export const RateLimited: Story = {
  decorators: [
    (Story) => {
      const { addOutput, clearOutput, setRateLimitInfo } = useLoopStore();
      useEffect(() => {
        clearOutput();
        addOutput("Starting loop...", "info");
        addOutput("Running iteration 1", "output");
        setRateLimitInfo({
          attempt: 2,
          maxAttempts: 5,
          retryInSecs: 300,
          startTime: new Date(),
        });
        addOutput("Rate limited by API. Waiting to retry...", "info");
      }, [addOutput, clearOutput, setRateLimitInfo]);
      return <Story />;
    },
  ],
};
