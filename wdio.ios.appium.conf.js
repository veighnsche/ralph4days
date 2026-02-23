import { existsSync, mkdirSync } from 'fs'
import path from 'path'

const appiumHost = process.env.RALPH_IOS_E2E_APPIUM_HOST?.trim() || '127.0.0.1'
const appiumPortRaw = process.env.RALPH_IOS_E2E_APPIUM_PORT?.trim() || '4723'
const appiumPort = Number.parseInt(appiumPortRaw, 10)
const screenshotDir = process.env.RALPH_IOS_E2E_SCREENSHOT_DIR?.trim() || '/tmp/ralph-ios-e2e'
const deviceName = process.env.RALPH_IOS_E2E_DEVICE?.trim() || 'iPhone 17 Pro'
const bundleId = process.env.RALPH_IOS_E2E_BUNDLE_ID?.trim() || 'com.vince.ralph'
const udid = process.env.RALPH_IOS_E2E_UDID?.trim()
const appPathRaw = process.env.RALPH_IOS_E2E_APP_PATH?.trim()
const resetAppState = process.env.RALPH_IOS_E2E_RESET_APP_STATE === '1'

if (!Number.isInteger(appiumPort) || appiumPort < 1 || appiumPort > 65535) {
  throw new Error(`Invalid RALPH_IOS_E2E_APPIUM_PORT value '${appiumPortRaw}'`)
}

if (!appPathRaw) {
  throw new Error('RALPH_IOS_E2E_APP_PATH is required for iOS Appium e2e runs')
}

const appPath = path.resolve(appPathRaw)
if (!existsSync(appPath)) {
  throw new Error(`RALPH_IOS_E2E_APP_PATH does not exist: ${appPath}`)
}

mkdirSync(screenshotDir, { recursive: true })

const capability = {
  platformName: 'iOS',
  'appium:automationName': 'XCUITest',
  'appium:deviceName': deviceName,
  'appium:platformVersion': process.env.RALPH_IOS_E2E_PLATFORM_VERSION?.trim(),
  'appium:bundleId': bundleId,
  'appium:app': appPath,
  'appium:noReset': !resetAppState,
  'appium:includeSafariInWebviews': true,
  'appium:webviewConnectRetries': 20,
  'appium:webviewConnectTimeout': 120000,
  'appium:newCommandTimeout': 240,
  'appium:wdaLaunchTimeout': 120000,
  'appium:wdaConnectionTimeout': 120000
}

if (udid && udid.length > 0) {
  capability['appium:udid'] = udid
}

if (!capability['appium:platformVersion']) {
  delete capability['appium:platformVersion']
}

const specOverride = process.env.TAURI_E2E_SPEC?.trim()

export const config = {
  runner: 'local',
  protocol: 'http',
  hostname: appiumHost,
  port: appiumPort,
  path: '/',
  specs: [specOverride || './e2e-ios/**/*.spec.js'],
  maxInstances: 1,
  capabilities: [capability],
  reporters: ['spec'],
  framework: 'mocha',
  mochaOpts: {
    ui: 'bdd',
    timeout: 180000
  }
}
