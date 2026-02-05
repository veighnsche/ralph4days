import type { Meta, StoryObj } from "@storybook/react";
import { useEffect } from "react";
import { useLoopStore } from "@/stores/useLoopStore";
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

export const Idle: Story = {
  decorators: [
    (Story) => {
      const setStatus = useLoopStore((state) => state.setStatus);
      useEffect(() => {
        setStatus({
          state: "idle",
          current_iteration: 0,
          max_iterations: 50,
          stagnant_count: 0,
          rate_limit_retries: 0,
          last_progress_hash: null,
          project_path: null,
        });
      }, [setStatus]);
      return <Story />;
    },
  ],
};

export const Running: Story = {
  decorators: [
    (Story) => {
      const setStatus = useLoopStore((state) => state.setStatus);
      useEffect(() => {
        setStatus({
          state: "running",
          current_iteration: 15,
          max_iterations: 50,
          stagnant_count: 0,
          rate_limit_retries: 0,
          last_progress_hash: null,
          project_path: "/home/user/projects/my-app",
        });
      }, [setStatus]);
      return <Story />;
    },
  ],
};

export const Paused: Story = {
  decorators: [
    (Story) => {
      const setStatus = useLoopStore((state) => state.setStatus);
      useEffect(() => {
        setStatus({
          state: "paused",
          current_iteration: 20,
          max_iterations: 50,
          stagnant_count: 0,
          rate_limit_retries: 0,
          last_progress_hash: null,
          project_path: "/home/user/projects/my-app",
        });
      }, [setStatus]);
      return <Story />;
    },
  ],
};

export const RateLimited: Story = {
  decorators: [
    (Story) => {
      const { setStatus, setRateLimitInfo } = useLoopStore();
      useEffect(() => {
        setStatus({
          state: "rate_limited",
          current_iteration: 10,
          max_iterations: 50,
          stagnant_count: 0,
          rate_limit_retries: 2,
          last_progress_hash: null,
          project_path: "/home/user/projects/my-app",
        });
        setRateLimitInfo({
          attempt: 2,
          maxAttempts: 5,
          retryInSecs: 300,
          startTime: new Date(),
        });
      }, [setStatus, setRateLimitInfo]);
      return <Story />;
    },
  ],
};

export const Complete: Story = {
  decorators: [
    (Story) => {
      const setStatus = useLoopStore((state) => state.setStatus);
      useEffect(() => {
        setStatus({
          state: "complete",
          current_iteration: 50,
          max_iterations: 50,
          stagnant_count: 0,
          rate_limit_retries: 0,
          last_progress_hash: "abc123",
          project_path: "/home/user/projects/my-app",
        });
      }, [setStatus]);
      return <Story />;
    },
  ],
};

export const Aborted: Story = {
  decorators: [
    (Story) => {
      const setStatus = useLoopStore((state) => state.setStatus);
      useEffect(() => {
        setStatus({
          state: "aborted",
          current_iteration: 25,
          max_iterations: 50,
          stagnant_count: 3,
          rate_limit_retries: 0,
          last_progress_hash: "abc123",
          project_path: "/home/user/projects/my-app",
        });
      }, [setStatus]);
      return <Story />;
    },
  ],
};
