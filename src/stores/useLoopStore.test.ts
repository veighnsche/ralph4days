import { beforeEach, describe, expect, it } from "vitest";
import { useLoopStore } from "./useLoopStore";

describe("useLoopStore", () => {
  beforeEach(() => {
    useLoopStore.getState().reset();
  });

  describe("initial state", () => {
    it("starts with idle state", () => {
      const { status } = useLoopStore.getState();
      expect(status.state).toBe("idle");
    });

    it("starts with zero iterations", () => {
      const { status } = useLoopStore.getState();
      expect(status.current_iteration).toBe(0);
    });

    it("starts with empty output", () => {
      const { output } = useLoopStore.getState();
      expect(output).toHaveLength(0);
    });

    it("starts with no rate limit info", () => {
      const { rateLimitInfo } = useLoopStore.getState();
      expect(rateLimitInfo).toBeNull();
    });
  });

  describe("addOutput", () => {
    it("adds output lines with incrementing IDs", () => {
      const { addOutput } = useLoopStore.getState();
      addOutput("Line 1");
      addOutput("Line 2");

      const { output } = useLoopStore.getState();
      expect(output).toHaveLength(2);
      expect(output[0].id).toBeLessThan(output[1].id);
    });

    it("includes timestamp on output lines", () => {
      const before = new Date();
      useLoopStore.getState().addOutput("Test");
      const after = new Date();

      const { output } = useLoopStore.getState();
      expect(output[0].timestamp.getTime()).toBeGreaterThanOrEqual(before.getTime());
      expect(output[0].timestamp.getTime()).toBeLessThanOrEqual(after.getTime());
    });

    it("sets correct type for output lines", () => {
      const { addOutput } = useLoopStore.getState();
      addOutput("Normal output");
      addOutput("Error message", "error");
      addOutput("Info message", "info");

      const { output } = useLoopStore.getState();
      expect(output[0].type).toBe("output");
      expect(output[1].type).toBe("error");
      expect(output[2].type).toBe("info");
    });

    it("limits output buffer to 5000 lines", () => {
      const { addOutput } = useLoopStore.getState();

      for (let i = 0; i < 5100; i++) {
        addOutput(`Line ${i}`);
      }

      const { output } = useLoopStore.getState();
      expect(output.length).toBeLessThanOrEqual(5000);
    });
  });

  describe("setStatus", () => {
    it("updates status correctly", () => {
      const { setStatus } = useLoopStore.getState();
      setStatus({
        state: "running",
        current_iteration: 5,
        max_iterations: 100,
        stagnant_count: 0,
        rate_limit_retries: 0,
        last_progress_hash: null,
        project_path: "/test/path",
      });

      const { status } = useLoopStore.getState();
      expect(status.state).toBe("running");
      expect(status.current_iteration).toBe(5);
    });
  });

  describe("setRateLimitInfo", () => {
    it("sets rate limit info", () => {
      const { setRateLimitInfo } = useLoopStore.getState();
      const info = {
        retryInSecs: 300,
        attempt: 1,
        maxAttempts: 5,
        startTime: new Date(),
      };
      setRateLimitInfo(info);

      const { rateLimitInfo } = useLoopStore.getState();
      expect(rateLimitInfo).toEqual(info);
    });

    it("clears rate limit info when set to null", () => {
      const { setRateLimitInfo } = useLoopStore.getState();
      setRateLimitInfo({
        retryInSecs: 300,
        attempt: 1,
        maxAttempts: 5,
        startTime: new Date(),
      });
      setRateLimitInfo(null);

      const { rateLimitInfo } = useLoopStore.getState();
      expect(rateLimitInfo).toBeNull();
    });
  });

  describe("clearOutput", () => {
    it("clears all output", () => {
      const { addOutput, clearOutput } = useLoopStore.getState();
      addOutput("Line 1");
      addOutput("Line 2");
      clearOutput();

      const { output } = useLoopStore.getState();
      expect(output).toHaveLength(0);
    });
  });

  describe("reset", () => {
    it("resets to initial state", () => {
      const store = useLoopStore.getState();
      store.setStatus({
        state: "running",
        current_iteration: 50,
        max_iterations: 100,
        stagnant_count: 2,
        rate_limit_retries: 1,
        last_progress_hash: "abc123",
        project_path: "/test",
      });
      store.addOutput("Some output");
      store.setRateLimitInfo({
        retryInSecs: 300,
        attempt: 1,
        maxAttempts: 5,
        startTime: new Date(),
      });

      store.reset();

      const { status, output, rateLimitInfo } = useLoopStore.getState();
      expect(status.state).toBe("idle");
      expect(status.current_iteration).toBe(0);
      expect(output).toHaveLength(0);
      expect(rateLimitInfo).toBeNull();
    });
  });
});
