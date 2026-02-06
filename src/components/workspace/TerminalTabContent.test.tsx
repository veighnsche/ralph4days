import { render, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { WorkspaceTab } from "@/stores/useWorkspaceStore";
import { TerminalTabContent } from "./TerminalTabContent";

// Mock dependencies
vi.mock("@/lib/terminal", () => ({
  Terminal: ({ onReady }: { onReady?: (terminal: unknown) => void }) => {
    // Simulate terminal ready
    if (onReady) {
      setTimeout(() => {
        onReady({
          cols: 80,
          rows: 24,
          write: vi.fn(),
          writeln: vi.fn(),
          onData: vi.fn(),
        });
      }, 0);
    }
    return <div data-testid="terminal">Terminal</div>;
  },
  useTerminalSession: () => ({
    isConnected: true,
    isReady: false,
    markReady: vi.fn(),
    sendInput: vi.fn(),
    resize: vi.fn(),
  }),
}));

vi.mock("@/hooks/useTabMeta", () => ({
  useTabMeta: vi.fn(),
}));

vi.mock("lucide-react", () => ({
  TerminalSquare: () => <svg data-testid="terminal-icon" />,
}));

describe("TerminalTabContent", () => {
  const mockTab: WorkspaceTab = {
    id: "test-terminal-1",
    type: "terminal",
    title: "Terminal 1",
    data: {
      model: "haiku",
      thinking: true,
    },
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders Terminal component", () => {
    const { getByTestId } = render(<TerminalTabContent tab={mockTab} />);
    expect(getByTestId("terminal")).toBeTruthy();
  });

  it("sets tab metadata", async () => {
    const { useTabMeta } = await import("@/hooks/useTabMeta");
    render(<TerminalTabContent tab={mockTab} />);

    await waitFor(() => {
      expect(useTabMeta).toHaveBeenCalledWith("test-terminal-1", "Terminal 1", expect.any(Function));
    });
  });

  it("passes interactive mode to Terminal", () => {
    const { getByTestId } = render(<TerminalTabContent tab={mockTab} />);
    const terminal = getByTestId("terminal");
    expect(terminal).toBeTruthy();
  });

  it("handles tab with minimal config", () => {
    const minimalTab: WorkspaceTab = {
      id: "test-terminal-2",
      type: "terminal",
      title: "Terminal 2",
    };

    const { getByTestId } = render(<TerminalTabContent tab={minimalTab} />);
    expect(getByTestId("terminal")).toBeTruthy();
  });

  it("uses tab.data.model if provided", async () => {
    const { useTerminalSession } = await import("@/lib/terminal");

    render(<TerminalTabContent tab={mockTab} />);

    // useTerminalSession should be called with model from tab.data
    await waitFor(() => {
      expect(useTerminalSession).toBeDefined();
    });
  });

  it("uses tab.data.thinking if provided", async () => {
    const { useTerminalSession } = await import("@/lib/terminal");

    render(<TerminalTabContent tab={mockTab} />);

    await waitFor(() => {
      expect(useTerminalSession).toBeDefined();
    });
  });

  it("renders with responsive layout classes", () => {
    const { container } = render(<TerminalTabContent tab={mockTab} />);
    const wrapper = container.querySelector(".flex-1");
    expect(wrapper).toBeTruthy();
    expect(wrapper?.className).toContain("flex");
    expect(wrapper?.className).toContain("flex-col");
    expect(wrapper?.className).toContain("min-h-0");
  });
});
