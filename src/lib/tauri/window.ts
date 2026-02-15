import { getCurrentWindow } from '@tauri-apps/api/window'

export async function tauriSetWindowTitle(title: string): Promise<void> {
  await getCurrentWindow().setTitle(title)
}
