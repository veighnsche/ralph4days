import { remote } from 'webdriverio'

const targetBinary = '/home/vince/Projects/ralph4days/src-tauri/target/debug/ralph4days'
const projectPath = '/tmp/ralph4days-mock/04-desktop-dev'

const originalFetch = globalThis.fetch
let sessionId = 0

globalThis.fetch = async (input, init = {}) => {
  if (String(input).includes('/session') && init?.method === 'POST') {
    console.log('RAW BODY', init.body)
    sessionId += 1
  }
  const response = await originalFetch(input, init)
  if (String(input).includes('/session') && init?.method === 'POST') {
    console.log('RAW RESPONSE', await response.clone().text())
  }
  return response
}

try {
  await remote({
    hostname: '127.0.0.1',
    port: 4444,
    capabilities: {
      browserName: 'wry',
      'tauri:options': {
        application: targetBinary,
        args: ['--project', projectPath],
      },
    },
    transformRequest: (requestOptions) => {
      if (requestOptions.method !== 'POST' || typeof requestOptions.body !== 'string') {
        return requestOptions
      }

      try {
        const body = JSON.parse(requestOptions.body)
        const alwaysMatch = body?.capabilities?.alwaysMatch

        if (alwaysMatch && typeof alwaysMatch === 'object') {
          delete alwaysMatch.webSocketUrl
          delete alwaysMatch.unhandledPromptBehavior
        }

        requestOptions.body = JSON.stringify(body)
        const bytes = new TextEncoder().encode(requestOptions.body).byteLength
        requestOptions.headers?.set('Content-Length', `${bytes}`)
      } catch (error) {
        console.error('transformRequest parse error', error)
      }

      return requestOptions
    },
  })
} catch (error) {
  console.error(error)
}

await new Promise(resolve => setTimeout(resolve, 1000))
