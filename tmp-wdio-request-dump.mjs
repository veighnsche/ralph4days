import { remote } from 'webdriverio'

const targetBinary = '/home/vince/Projects/ralph4days/src-tauri/target/debug/ralph4days'
const projectPath = '/tmp/ralph4days-mock/04-desktop-dev'

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
  })
} catch (error) {
  console.error(error.message)
}

await new Promise(resolve => setTimeout(resolve, 500))
