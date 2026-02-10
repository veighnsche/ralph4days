import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from "@modelcontextprotocol/sdk/types.js";
import { spawn, type Subprocess } from "bun";
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
