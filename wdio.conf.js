import os from 'os'
import path from 'path'
import net from 'net'
import { existsSync } from 'fs'
import { spawn, spawnSync } from 'child_process'
import { fileURLToPath } from 'url'

const __dirname = fileURLToPath(new URL('.', import.meta.url))
const projectRoot = path.resolve(__dirname)
const binaryName = process.platform === 'win32' ? 'ralph4days.exe' : 'ralph4days'
const application = path.resolve(projectRoot, 'target', 'debug', binaryName)
const driverBinaryName = process.platform === 'win32' ? 'tauri-driver.exe' : 'tauri-driver'
const tauriDriverHost = '127.0.0.1'
const tauriDriverPort = 4444
const tauriDriverReadyTimeoutMs = 10000

let tauriDriver
let shouldExit = false

function findInPath(binaryName) {
  const searchPath = process.env.PATH ?? ''
  const pathEntries = searchPath.split(path.delimiter)
  const extensions =
    process.platform === 'win32' ? ['.exe', '.cmd', '.bat', ''] : ['']

  for (const entry of pathEntries) {
    if (!entry) continue

    for (const extension of extensions) {
      const candidate = path.resolve(entry, `${binaryName}${extension}`)
      if (existsSync(candidate)) {
        return candidate
      }
    }
  }

  return null
}

function resolveTauriProjectPath() {
  const providedProject = process.env.RALPH_E2E_PROJECT?.trim()
  const baseCandidate =
    providedProject && providedProject.length > 0
      ? path.resolve(providedProject)
      : path.resolve(process.env.RALPH_MOCK_DIR || '/tmp/ralph4days-mock', '04-desktop-dev')

  const ralphDb = path.resolve(baseCandidate, '.ralph', 'db', 'ralph.db')
  if (!existsSync(baseCandidate) || !existsSync(ralphDb)) {
    throw new Error(
      `Invalid e2e project path: ${baseCandidate}. ` +
        'Set RALPH_E2E_PROJECT to a copied mock fixture with .ralph/db/ralph.db present.'
    )
  }

  return baseCandidate
}

function resolveTauriDriverBinary() {
  const envBinary = process.env.TAURI_DRIVER_BINARY?.trim()
  if (envBinary && existsSync(envBinary)) {
    return envBinary
  }

  const homeBinary = path.resolve(os.homedir(), '.cargo', 'bin', driverBinaryName)
  if (existsSync(homeBinary)) {
    return homeBinary
  }

  const pathBinary = findInPath(driverBinaryName)
  if (pathBinary) {
    return pathBinary
  }

  throw new Error(
    'tauri-driver not found. Install with `cargo install tauri-driver --locked` and ensure it is in PATH.'
  )
}

function ensureTauriBinary() {
  if (!existsSync(application)) {
    throw new Error(
      `Expected debug app binary at ${application}. ` +
        'Run `bun tauri build --debug --no-bundle` before running e2e tests.'
    )
  }
}

function stopTauriDriver() {
  shouldExit = true
  tauriDriver?.kill()
}

function waitForPortReady(hostname, port, timeoutMs) {
  const deadline = Date.now() + timeoutMs
  let lastError = null

  const attemptConnect = () =>
    new Promise((resolve, reject) => {
      const socket = new net.Socket()

      const finalize = callback => value => {
        socket.removeAllListeners()
        socket.destroy()
        callback(value)
      }

      socket.setTimeout(250)
      socket.once('connect', finalize(resolve))
      socket.once('timeout', finalize(() => reject(new Error(`timeout connecting to ${hostname}:${port}`))))
      socket.once('error', finalize(reject))
      socket.connect(port, hostname)
    })

  const poll = async () => {
    while (Date.now() < deadline) {
      if (tauriDriver?.exitCode !== null) {
        throw new Error(`tauri-driver exited before listening on ${hostname}:${port}`)
      }

      try {
        await attemptConnect()
        return
      } catch (error) {
        lastError = error
      }

      await new Promise(resolve => setTimeout(resolve, 100))
    }

    const lastErrorMessage = lastError instanceof Error ? lastError.message : String(lastError)
    throw new Error(
      `tauri-driver failed to listen on ${hostname}:${port} within ${timeoutMs}ms. Last error: ${lastErrorMessage}`
    )
  }

  return poll()
}

async function startTauriDriver() {
  shouldExit = false
  const driverBinary = resolveTauriDriverBinary()
  tauriDriver = spawn(driverBinary, { stdio: [null, process.stdout, process.stderr] })

  tauriDriver.on('error', error => {
    console.error('tauri-driver error:', error.message)
    process.exit(1)
  })

  tauriDriver.on('exit', code => {
    if (!shouldExit) {
      console.error('tauri-driver exited unexpectedly:', code)
      process.exit(1)
    }
  })

  try {
    await waitForPortReady(tauriDriverHost, tauriDriverPort, tauriDriverReadyTimeoutMs)
  } catch (error) {
    stopTauriDriver()
    throw error
  }
}

const projectPath = resolveTauriProjectPath()
const specOverride = process.env.TAURI_E2E_SPEC?.trim()

export const config = {
  runner: 'local',
  protocol: 'http',
  hostname: tauriDriverHost,
  port: tauriDriverPort,
  path: '/',
  specs: [specOverride || './e2e-tauri/**/*.spec.js'],
  maxInstances: 1,
  capabilities: [
    {
      browserName: 'wry',
      'tauri:options': {
        application,
        args: ['--project', projectPath, '--no-splash'],
      },
    },
  ],
  transformRequest: requestOptions => {
    if (requestOptions.method !== 'POST' || typeof requestOptions.body !== 'string') {
      return requestOptions
    }

    try {
      const payload = JSON.parse(requestOptions.body)
      const alwaysMatch = payload?.capabilities?.alwaysMatch

      if (alwaysMatch && typeof alwaysMatch === 'object') {
        delete alwaysMatch.webSocketUrl
        delete alwaysMatch.unhandledPromptBehavior
        requestOptions.body = JSON.stringify(payload)
        const bytes = new TextEncoder().encode(requestOptions.body).byteLength
        requestOptions.headers?.set('Content-Length', `${bytes}`)
      }
    } catch {
    }

    return requestOptions
  },
  reporters: ['spec'],
  framework: 'mocha',
  mochaOpts: {
    ui: 'bdd',
    timeout: 120000,
  },
  onPrepare: () => {
    const buildResult = spawnSync('bun', ['tauri', 'build', '--debug', '--no-bundle'], {
      cwd: projectRoot,
      stdio: 'inherit',
      shell: true,
    })

    if (buildResult.status !== 0) {
      throw new Error('tauri build failed; cannot run Tauri e2e tests')
    }

    ensureTauriBinary()
  },
  beforeSession: startTauriDriver,
  afterSession: stopTauriDriver,
  onComplete: stopTauriDriver,
}
