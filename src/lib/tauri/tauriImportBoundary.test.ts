import { readdirSync, readFileSync } from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const SRC_ROOT = path.resolve(process.cwd(), 'src')

const ALLOWED = new Set(
  ['src/lib/tauri/invoke.ts', 'src/lib/tauri/events.ts', 'src/lib/tauri/window.ts'].map(p =>
    path.resolve(process.cwd(), p)
  )
)

function listSourceFiles(dir: string): string[] {
  const entries = readdirSync(dir, { withFileTypes: true })
  const out: string[] = []

  for (const e of entries) {
    const abs = path.join(dir, e.name)
    if (e.isDirectory()) {
      out.push(...listSourceFiles(abs))
      continue
    }
    if (!e.isFile()) continue
    const isSourceFile = abs.endsWith('.ts') || abs.endsWith('.tsx')
    if (!isSourceFile) continue
    // This boundary is for production code; unit tests may mock Tauri modules directly.
    if (abs.includes(`${path.sep}test${path.sep}`)) continue
    if (abs.includes('.test.')) continue
    if (abs.includes('.stories.')) continue
    out.push(abs)
  }

  return out
}

describe('tauri import boundary', () => {
  it('does not allow @tauri-apps/api/* imports outside boundary modules', () => {
    const files = listSourceFiles(SRC_ROOT)
    const offenders: { file: string; line: number; text: string }[] = []

    for (const file of files) {
      const text = readFileSync(file, 'utf8')
      if (!text.includes('@tauri-apps/api/')) continue
      if (ALLOWED.has(file)) continue

      const lines = text.split('\n')
      for (let i = 0; i < lines.length; i++) {
        if (!lines[i].includes('@tauri-apps/api/')) continue
        offenders.push({
          file: path.relative(process.cwd(), file),
          line: i + 1,
          text: lines[i].trim()
        })
      }
    }

    expect(offenders).toEqual([])
  })
})
