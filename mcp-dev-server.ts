import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from "@modelcontextprotocol/sdk/types.js";
import { spawn, type Subprocess, type ServerWebSocket } from "bun";
import { readdirSync, readFileSync, writeFileSync, existsSync, unlinkSync } from "node:fs";
import { join } from "node:path";
import { execSync } from "node:child_process";

const ROOT = "/home/vince/Projects/ralph4days";
const MOCK_DIR = join(ROOT, "mock");
const LAST_PROJECT_FILE = join(ROOT, ".dev-last-project");
const SCREENSHOT_PATH = "/tmp/ralph-dev-screenshot.png";
const KWIN_SCRIPT_PATH = "/tmp/kwin-activate-ralph.js";
const KWIN_SCRIPT_NAME = "mcp-activate-ralph";

let devProcess: Subprocess | null = null;

// --- WebSocket bridge for DOM interaction ---
let bridgeSocket: ServerWebSocket<unknown> | null = null;
let commandId = 0;
const pending = new Map<string, { resolve: (v: string) => void; reject: (e: Error) => void; timer: ReturnType<typeof setTimeout> }>();

Bun.serve({
  port: 9223,
  fetch(req, server) {
    if (server.upgrade(req)) return undefined;
    return new Response("dev-tauri bridge", { status: 200 });
  },
  websocket: {
    open(ws) {
      bridgeSocket = ws;
    },
    close() {
      bridgeSocket = null;
    },
    message(_ws, msg) {
      try {
        const data = JSON.parse(String(msg));
        const entry = pending.get(data.id);
        if (!entry) return;
        pending.delete(data.id);
        clearTimeout(entry.timer);
        if (data.type === "error") {
          entry.reject(new Error(data.message));
        } else {
          entry.resolve(typeof data.value === "string" ? data.value : JSON.stringify(data.value));
        }
      } catch {}
    },
  },
});

function sendBridgeCommand(type: string, params: Record<string, unknown>, timeoutMs = 5000): Promise<string> {
  return new Promise((resolve, reject) => {
    if (!bridgeSocket) {
      reject(new Error("Dev bridge not connected. Is the app running in dev mode?"));
      return;
    }
    const id = String(++commandId);
    const timer = setTimeout(() => {
      pending.delete(id);
      reject(new Error(`Bridge command timed out after ${timeoutMs}ms`));
    }, timeoutMs);
    pending.set(id, { resolve, reject, timer });
    bridgeSocket.send(JSON.stringify({ id, type, ...params }));
  });
}

function getMockProjects(): string[] {
  return readdirSync(MOCK_DIR, { withFileTypes: true })
    .filter((d) => d.isDirectory())
    .map((d) => d.name)
    .sort();
}

function getLastProject(): string | null {
  try {
    const last = readFileSync(LAST_PROJECT_FILE, "utf-8").trim();
    if (last && existsSync(join(MOCK_DIR, last))) return last;
  } catch {}
  return null;
}

function saveLastProject(name: string) {
  writeFileSync(LAST_PROJECT_FILE, name + "\n");
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function focusRalphWindow(): boolean {
  try {
    // Unload previous script if it's still registered
    try {
      execSync(`qdbus org.kde.KWin /Scripting unloadScript ${KWIN_SCRIPT_NAME}`, {
        timeout: 1000,
        stdio: "pipe",
      });
    } catch {}

    writeFileSync(
      KWIN_SCRIPT_PATH,
      `const clients = workspace.windowList();
for (const c of clients) {
  const cls = (c.resourceClass || "").toLowerCase();
  const desk = (c.desktopFile || "").toLowerCase();
  if (cls === "ralph4days" || desk === "ralph4days") {
    if (c.minimized) c.minimized = false;
    workspace.activeWindow = c;
    break;
  }
}
`
    );

    execSync(
      `qdbus org.kde.KWin /Scripting loadScript ${KWIN_SCRIPT_PATH} ${KWIN_SCRIPT_NAME}`,
      { timeout: 2000, stdio: "pipe" }
    );
    execSync(`qdbus org.kde.KWin /Scripting start`, {
      timeout: 2000,
      stdio: "pipe",
    });
    return true;
  } catch {
    return false;
  } finally {
    try {
      execSync(`qdbus org.kde.KWin /Scripting unloadScript ${KWIN_SCRIPT_NAME}`, {
        timeout: 1000,
        stdio: "pipe",
      });
    } catch {}
  }
}

const server = new Server(
  { name: "dev-tauri", version: "1.0.0" },
  { capabilities: { tools: {} } }
);

server.setRequestHandler(ListToolsRequestSchema, async () => {
  const projects = getMockProjects();
  const last = getLastProject();
  const projectList = projects
    .map((p) => (p === last ? `${p} (last used)` : p))
    .join(", ");

  return {
    tools: [
      {
        name: "start_dev_tauri",
        description: `Start \`bun tauri dev\` in the background for the user to visually review the app. Call this when your code changes are ready and you want the user to see the result.\n\nAvailable mock projects: ${projectList}\n\nDefault: opens last used project${last ? ` ("${last}")` : " (none saved — launches welcome screen)"}. Set \`new: true\` to launch the welcome/project-picker screen instead.\n\nIMPORTANT WORKFLOW: Tauri hot-reloads on file changes, triggering Rust rebuilds. If you need to edit Rust code, Cargo.toml, or tauri.conf.json: (1) stop_dev_tauri first, (2) make ALL your changes, (3) THEN start_dev_tauri. Never edit backend files while the dev server is running.`,
        inputSchema: {
          type: "object" as const,
          properties: {
            project: {
              type: "string" as const,
              description: `Mock project folder name. Available: ${projects.join(", ")}`,
              enum: projects,
            },
            new: {
              type: "boolean" as const,
              description: "Launch without a project to show the welcome/project-picker screen.",
            },
          },
        },
      },
      {
        name: "stop_dev_tauri",
        description:
          "Stop the running dev_tauri process. You MUST call this before making ANY changes to Rust code, Cargo.toml, or tauri.conf.json. Tauri's hot-reload will respawn cargo builds on file changes, which overwhelms the system. Stop first, make your changes, then start again when ready for review.",
        inputSchema: { type: "object" as const, properties: {} },
      },
      {
        name: "screenshot_dev_tauri",
        description:
          "Take a screenshot of the running Ralph Tauri window. Focuses the window via KWin scripting, captures it with Spectacle, and returns the image. Use this to visually verify UI changes after starting dev_tauri.",
        inputSchema: { type: "object" as const, properties: {} },
      },
      {
        name: "eval_dev_tauri",
        description:
          "Execute arbitrary JavaScript in the running Tauri webview and return the result. Uses a WebSocket bridge to the frontend. The code is evaluated as an expression first, falling back to statement execution.",
        inputSchema: {
          type: "object" as const,
          properties: {
            code: {
              type: "string" as const,
              description: "JavaScript code to evaluate in the webview DOM context.",
            },
          },
          required: ["code"],
        },
      },
      {
        name: "click_dev_tauri",
        description:
          "Click an element in the running Tauri webview. Find by CSS selector or by visible text content. Returns a description of the clicked element.",
        inputSchema: {
          type: "object" as const,
          properties: {
            selector: {
              type: "string" as const,
              description: "CSS selector for the element to click.",
            },
            text: {
              type: "string" as const,
              description: "Visible text content to search for. Finds the deepest matching element.",
            },
          },
        },
      },
      {
        name: "type_dev_tauri",
        description:
          "Type text into an input or textarea in the running Tauri webview. Clears existing value first by default. Dispatches input and change events for React compatibility.",
        inputSchema: {
          type: "object" as const,
          properties: {
            selector: {
              type: "string" as const,
              description: "CSS selector for the input/textarea element.",
            },
            text: {
              type: "string" as const,
              description: "Text to type into the element.",
            },
            clear: {
              type: "boolean" as const,
              description: "Whether to clear existing value first. Defaults to true.",
            },
          },
          required: ["selector", "text"],
        },
      },
      {
        name: "scroll_dev_tauri",
        description:
          "Scroll an element or the page in the running Tauri webview. Can scroll by delta pixels or jump to top/bottom. Returns the new scrollTop position.",
        inputSchema: {
          type: "object" as const,
          properties: {
            selector: {
              type: "string" as const,
              description: "CSS selector for the scrollable element. Defaults to document root.",
            },
            deltaY: {
              type: "number" as const,
              description: "Pixels to scroll vertically. Positive = down, negative = up. Default: 300.",
            },
            to: {
              type: "string" as const,
              enum: ["top", "bottom"],
              description: 'Jump to "top" or "bottom" of the scrollable area.',
            },
          },
        },
      },
    ],
  };
});

server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  switch (name) {
    case "start_dev_tauri": {
      if (devProcess && devProcess.exitCode === null) {
        return {
          content: [
            {
              type: "text" as const,
              text: "dev_tauri is already running (pid: " + devProcess.pid + ")",
            },
          ],
        };
      }

      // Kill any orphaned tauri dev process hogging port 1420
      try {
        const pid = execSync("lsof -ti :1420", { timeout: 2000, stdio: "pipe" }).toString().trim();
        if (pid) {
          execSync(`kill ${pid}`, { timeout: 2000, stdio: "pipe" });
          await sleep(500);
        }
      } catch {}

      const isNew = Boolean(args?.new);
      const project = isNew ? null : (args?.project as string) || getLastProject() || null;

      if (project) {
        const projectPath = join(MOCK_DIR, project);
        if (!existsSync(projectPath)) {
          return {
            content: [
              {
                type: "text" as const,
                text: `Mock project "${project}" not found. Available: ${getMockProjects().join(", ")}`,
              },
            ],
            isError: true,
          };
        }
        saveLastProject(project);

        devProcess = spawn(
          ["setsid", "bun", "tauri", "dev", "--", "--", "--project", projectPath],
          { cwd: ROOT, stdout: "pipe", stderr: "pipe", env: { ...process.env } }
        );
      } else {
        devProcess = spawn(
          ["setsid", "bun", "tauri", "dev"],
          { cwd: ROOT, stdout: "pipe", stderr: "pipe", env: { ...process.env } }
        );
      }

      // Wait for build to complete and window to appear
      const buildOutput: string[] = [];
      let buildFailed = false;
      const MAX_WAIT = 120_000;
      const startTime = Date.now();

      // Collect stderr (cargo output goes to stderr)
      const decoder = new TextDecoder();
      const readStream = async (stream: ReadableStream<Uint8Array> | null) => {
        if (!stream) return;
        const reader = stream.getReader();
        try {
          while (true) {
            const { done, value } = await reader.read();
            if (done) break;
            const text = decoder.decode(value, { stream: true });
            buildOutput.push(text);
            // Strip ANSI codes for pattern matching
            const clean = text.replace(/\x1b\[[0-9;]*m/g, "");
            if (clean.includes("error[E") || clean.includes("error: could not compile")) {
              buildFailed = true;
            }
          }
        } catch {}
      };

      // Start reading stderr in background
      const stderrPromise = readStream(devProcess.stderr as unknown as ReadableStream<Uint8Array>);

      // Poll: wait for build failure or window to appear
      let windowReady = false;
      while (Date.now() - startTime < MAX_WAIT) {
        await sleep(1000);

        if (buildFailed || (devProcess.exitCode !== null)) {
          await stderrPromise.catch(() => {});
          const output = buildOutput.join("").replace(/\x1b\[[0-9;]*m/g, "").trim();
          const lastLines = output.split("\n").slice(-30).join("\n");
          return {
            content: [
              {
                type: "text" as const,
                text: `Build failed:\n\n${lastLines}`,
              },
            ],
            isError: true,
          };
        }

        // Try to find the window
        if (focusRalphWindow()) {
          windowReady = true;
          await sleep(500);
          break;
        }
      }

      if (!windowReady) {
        const output = buildOutput.join("").replace(/\x1b\[[0-9;]*m/g, "").trim();
        const lastLines = output.split("\n").slice(-20).join("\n");
        return {
          content: [
            {
              type: "text" as const,
              text: `Timed out waiting for window (${MAX_WAIT / 1000}s). Build output:\n\n${lastLines}`,
            },
          ],
          isError: true,
        };
      }

      return {
        content: [
          {
            type: "text" as const,
            text: project
              ? `Started dev_tauri with project "${project}" (pid: ${devProcess.pid})`
              : `Started dev_tauri without a project — welcome screen (pid: ${devProcess.pid})`,
          },
        ],
      };
    }

    case "stop_dev_tauri": {
      if (!devProcess || devProcess.exitCode !== null) {
        return {
          content: [
            { type: "text" as const, text: "dev_tauri is not running." },
          ],
        };
      }

      const pid = devProcess.pid;
      try {
        process.kill(-pid, "SIGTERM");
      } catch {
        devProcess.kill();
      }

      await devProcess.exited;
      devProcess = null;
      return {
        content: [
          {
            type: "text" as const,
            text: "Stopped dev_tauri (was pid: " + pid + ")",
          },
        ],
      };
    }

    case "screenshot_dev_tauri": {
      if (!devProcess || devProcess.exitCode !== null) {
        return {
          content: [
            { type: "text" as const, text: "dev_tauri is not running. Start it first." },
          ],
          isError: true,
        };
      }

      // Kill any lingering spectacle instance that could block -b mode
      try {
        execSync("pkill -f 'spectacle' || true", { timeout: 1000, stdio: "pipe" });
      } catch {}

      const focused = focusRalphWindow();

      // Let KWin finish the focus switch and Wayland compositor settle
      await sleep(400);

      try {
        if (existsSync(SCREENSHOT_PATH)) unlinkSync(SCREENSHOT_PATH);

        execSync(`spectacle -b -a -n -o ${SCREENSHOT_PATH}`, {
          timeout: 5000,
          stdio: "pipe",
        });

        // Spectacle can exit before the file is fully flushed
        await sleep(200);

        if (!existsSync(SCREENSHOT_PATH)) {
          return {
            content: [
              {
                type: "text" as const,
                text: `Screenshot failed — Spectacle produced no file.${!focused ? " KWin focus also failed; the Ralph window may not exist yet." : ""}`,
              },
            ],
            isError: true,
          };
        }

        const stat = Bun.file(SCREENSHOT_PATH).size;
        if (stat < 1000) {
          return {
            content: [
              {
                type: "text" as const,
                text: `Screenshot suspiciously small (${stat} bytes) — the window may not have been ready. Try again in a few seconds.`,
              },
            ],
            isError: true,
          };
        }

        const imageData = readFileSync(SCREENSHOT_PATH).toString("base64");

        return {
          content: [
            {
              type: "text" as const,
              text: focused
                ? "Captured Ralph window."
                : "Warning: KWin focus failed — this may be a screenshot of whatever was active.",
            },
            {
              type: "image" as const,
              data: imageData,
              mimeType: "image/png",
            },
          ],
        };
      } catch (err) {
        return {
          content: [
            {
              type: "text" as const,
              text: `Screenshot failed: ${err instanceof Error ? err.message : String(err)}`,
            },
          ],
          isError: true,
        };
      }
    }

    case "eval_dev_tauri": {
      try {
        const result = await sendBridgeCommand("eval", { code: args?.code as string });
        return { content: [{ type: "text" as const, text: result }] };
      } catch (err) {
        return {
          content: [{ type: "text" as const, text: err instanceof Error ? err.message : String(err) }],
          isError: true,
        };
      }
    }

    case "click_dev_tauri": {
      try {
        const result = await sendBridgeCommand("click", {
          selector: args?.selector as string | undefined,
          text: args?.text as string | undefined,
        });
        return { content: [{ type: "text" as const, text: result }] };
      } catch (err) {
        return {
          content: [{ type: "text" as const, text: err instanceof Error ? err.message : String(err) }],
          isError: true,
        };
      }
    }

    case "type_dev_tauri": {
      try {
        const result = await sendBridgeCommand("type", {
          selector: args?.selector as string,
          text: args?.text as string,
          clear: args?.clear as boolean | undefined,
        });
        return { content: [{ type: "text" as const, text: result }] };
      } catch (err) {
        return {
          content: [{ type: "text" as const, text: err instanceof Error ? err.message : String(err) }],
          isError: true,
        };
      }
    }

    case "scroll_dev_tauri": {
      try {
        const result = await sendBridgeCommand("scroll", {
          selector: args?.selector as string | undefined,
          deltaY: args?.deltaY as number | undefined,
          to: args?.to as string | undefined,
        });
        return { content: [{ type: "text" as const, text: result }] };
      } catch (err) {
        return {
          content: [{ type: "text" as const, text: err instanceof Error ? err.message : String(err) }],
          isError: true,
        };
      }
    }

    default:
      return {
        content: [
          { type: "text" as const, text: `Unknown tool: ${name}` },
        ],
        isError: true,
      };
  }
});

const transport = new StdioServerTransport();
await server.connect(transport);
