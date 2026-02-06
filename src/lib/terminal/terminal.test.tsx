import { render, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { Terminal } from "./terminal";

// Mock functions need to be outside vi.mock for jest hoisting
const mockTerminalWrite = vi.fn();
const mockTerminalWriteln = vi.fn();
const mockTerminalDispose = vi.fn();
const mockTerminalOpen = vi.fn();
const mockTerminalOnData = vi.fn();
const mockTerminalOnResize = vi.fn();
const mockLoadAddon = vi.fn();
const mockFitAddonFit = vi.fn();

// Mock xterm - return class constructors directly
vi.mock("@xterm/xterm", () => {
  return {
    Terminal: class MockTerminal {
      cols = 80;
      rows = 24;
      write = mockTerminalWrite;
      writeln = mockTerminalWriteln;
      dispose = mockTerminalDispose;
      open = mockTerminalOpen;
      onData = mockTerminalOnData;
      onResize = mockTerminalOnResize;
      loadAddon = mockLoadAddon;
    },
  };
});

vi.mock("@xterm/addon-fit", () => {
  return {
    FitAddon: class MockFitAddon {
      fit = mockFitAddonFit;
    },
  };
});

vi.mock("@xterm/addon-web-links", () => {
  return {
    WebLinksAddon: class MockWebLinksAddon {},
  };
});

// Mock ResizeObserver
global.ResizeObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
}));

describe("Terminal", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("renders terminal container", () => {
    const { container } = render(<Terminal />);
    const terminalDiv = container.querySelector("div");
    expect(terminalDiv).toBeTruthy();
  });

  it("creates XTerm instance with default config", async () => {
    const { Terminal: MockXTerm } = await import("@xterm/xterm");

    render(<Terminal />);

    await waitFor(() => {
      expect(MockXTerm).toHaveBeenCalledWith(
        expect.objectContaining({
          cursorBlink: false,
          disableStdin: true,
          fontSize: 13,
          fontFamily: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace",
          scrollback: 10000,
        })
      );
    });
  });

  it("creates XTerm instance with interactive mode", async () => {
    const { Terminal: MockXTerm } = await import("@xterm/xterm");

    render(<Terminal interactive={true} />);

    await waitFor(() => {
      expect(MockXTerm).toHaveBeenCalledWith(
        expect.objectContaining({
          cursorBlink: true,
          disableStdin: false,
        })
      );
    });
  });

  it("loads FitAddon", async () => {
    const { FitAddon } = await import("@xterm/addon-fit");

    render(<Terminal />);

    await waitFor(() => {
      expect(FitAddon).toHaveBeenCalled();
      expect(mockLoadAddon).toHaveBeenCalled();
    });
  });

  it("loads WebLinksAddon", async () => {
    const { WebLinksAddon } = await import("@xterm/addon-web-links");

    render(<Terminal />);

    await waitFor(() => {
      expect(WebLinksAddon).toHaveBeenCalled();
      expect(mockLoadAddon).toHaveBeenCalledTimes(2);
    });
  });

  it("calls onReady when terminal is initialized", async () => {
    const onReady = vi.fn();
    render(<Terminal onReady={onReady} />);

    await waitFor(() => {
      expect(onReady).toHaveBeenCalledWith(expect.any(Object));
    });
  });

  it("opens terminal in container", async () => {
    render(<Terminal />);

    await waitFor(() => {
      expect(mockTerminalOpen).toHaveBeenCalledWith(expect.any(HTMLDivElement));
    });
  });

  it("fits terminal to container on mount", async () => {
    render(<Terminal />);

    await waitFor(() => {
      expect(mockFitAddonFit).toHaveBeenCalled();
    });
  });

  it("sets up resize observer", async () => {
    render(<Terminal />);

    await waitFor(() => {
      expect(global.ResizeObserver).toHaveBeenCalled();
    });
  });

  it("forwards resize events when callback provided", async () => {
    const onResize = vi.fn();
    render(<Terminal interactive={true} onResize={onResize} />);

    await waitFor(() => {
      expect(mockTerminalOnResize).toHaveBeenCalledWith(expect.any(Function));
    });
  });

  it("does not set up resize callback when not interactive", async () => {
    const { Terminal: MockXTerm } = await import("@xterm/xterm");

    render(<Terminal interactive={false} />);

    await waitFor(() => {
      expect(MockXTerm).toHaveBeenCalled();
    });

    // onResize should not be called for non-interactive terminals
    expect(mockTerminalOnResize).not.toHaveBeenCalled();
  });

  it("uses custom font size", async () => {
    const { Terminal: MockXTerm } = await import("@xterm/xterm");

    render(<Terminal fontSize={16} />);

    await waitFor(() => {
      expect(MockXTerm).toHaveBeenCalledWith(
        expect.objectContaining({
          fontSize: 16,
        })
      );
    });
  });

  it("uses custom font family", async () => {
    const { Terminal: MockXTerm } = await import("@xterm/xterm");

    render(<Terminal fontFamily="Courier New" />);

    await waitFor(() => {
      expect(MockXTerm).toHaveBeenCalledWith(
        expect.objectContaining({
          fontFamily: "Courier New",
        })
      );
    });
  });

  it("disposes terminal on unmount", async () => {
    const { unmount } = render(<Terminal />);

    await waitFor(() => {
      expect(mockTerminalOpen).toHaveBeenCalled();
    });

    unmount();

    expect(mockTerminalDispose).toHaveBeenCalled();
  });

  it("does not create duplicate terminals", async () => {
    const { Terminal: MockXTerm } = await import("@xterm/xterm");

    const { rerender } = render(<Terminal />);

    await waitFor(() => {
      expect(MockXTerm).toHaveBeenCalledTimes(1);
    });

    // Rerender should not create new terminal
    rerender(<Terminal />);

    await waitFor(() => {
      expect(MockXTerm).toHaveBeenCalledTimes(1);
    });
  });

  it("applies dark theme colors", async () => {
    const { Terminal: MockXTerm } = await import("@xterm/xterm");

    render(<Terminal />);

    await waitFor(() => {
      expect(MockXTerm).toHaveBeenCalledWith(
        expect.objectContaining({
          theme: expect.objectContaining({
            background: "#0a0a0a",
            foreground: "#e0e0e0",
            cursor: "#e0e0e0",
          }),
        })
      );
    });
  });
});
