import type { Meta, StoryObj } from "@storybook/react";
import { useEffect } from "react";
import { useLoopStore } from "@/stores/useLoopStore";
import { useWorkspaceStore } from "@/stores/useWorkspaceStore";
import { WorkspacePanel } from "./WorkspacePanel";

const meta = {
  title: "Components/WorkspacePanel",
  component: WorkspacePanel,
  tags: ["autodocs"],
  decorators: [
    (Story) => (
      <div className="h-[600px]">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof WorkspacePanel>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Empty: Story = {
  decorators: [
    (Story) => {
      const reset = useLoopStore((state) => state.reset);
      const closeAllExcept = useWorkspaceStore((state) => state.closeAllExcept);
      useEffect(() => {
        reset();
        closeAllExcept("__none__");
      }, [reset, closeAllExcept]);
      return <Story />;
    },
  ],
};

export const WithTerminal: Story = {
  decorators: [
    (Story) => {
      const reset = useLoopStore((state) => state.reset);
      const { openTab } = useWorkspaceStore();
      useEffect(() => {
        reset();
        openTab({
          type: "terminal",
          title: "Terminal 1",
          closeable: true,
        });
      }, [reset, openTab]);
      return <Story />;
    },
  ],
};

export const MultipleTerminals: Story = {
  decorators: [
    (Story) => {
      const reset = useLoopStore((state) => state.reset);
      const { openTab } = useWorkspaceStore();
      useEffect(() => {
        reset();
        openTab({ type: "terminal", title: "Terminal 1", closeable: true });
        openTab({ type: "terminal", title: "Terminal 2", closeable: true });
        openTab({ type: "terminal", title: "Terminal 3", closeable: true });
      }, [reset, openTab]);
      return <Story />;
    },
  ],
};
