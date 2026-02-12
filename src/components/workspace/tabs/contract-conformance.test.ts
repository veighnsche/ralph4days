import { readdirSync, readFileSync, statSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const REQUIRED_TAB_FILES = ['constants.ts', 'content.tsx', 'factory.ts', 'index.ts', 'module.ts', 'schema.ts'] as const
const OPTIONAL_TAB_DIRS = new Set(['components', 'hooks'])
const OPTIONAL_TAB_FILES = new Set(['selectors.ts', 'state.ts', 'store.tsx'])

function tabsRootDir() {
  const currentFile = fileURLToPath(import.meta.url)
  return path.dirname(currentFile)
}

function tabDirectories(): string[] {
  const tabsDir = tabsRootDir()

  return readdirSync(tabsDir)
    .filter(entry => {
      const fullPath = path.join(tabsDir, entry)
      return statSync(fullPath).isDirectory() && statSync(path.join(fullPath, 'module.ts')).isFile()
    })
    .sort()
}

function tabRootEntries(tabDirName: string): string[] {
  return readdirSync(path.join(tabsRootDir(), tabDirName)).sort()
}

function assertAllowedTabEntry(entry: string, tabDir: string) {
  if (REQUIRED_TAB_FILES.includes(entry as (typeof REQUIRED_TAB_FILES)[number])) return
  if (OPTIONAL_TAB_FILES.has(entry)) return
  if (OPTIONAL_TAB_DIRS.has(entry)) return
  if (/\.test\.(ts|tsx)$/.test(entry)) return
  throw new Error(`Unexpected tab entry '${entry}' in '${tabDir}'`)
}

function readTabFile(tabDir: string, fileName: string): string {
  return readFileSync(path.join(tabsRootDir(), tabDir, fileName), 'utf8')
}

describe('workspace tab folder structure', () => {
  it('enforces required files and allowed optional entries in each tab folder', () => {
    const dirs = tabDirectories()

    for (const tabDir of dirs) {
      const entries = tabRootEntries(tabDir)

      for (const fileName of REQUIRED_TAB_FILES) {
        expect(entries).toContain(fileName)
      }

      for (const entry of entries) {
        assertAllowedTabEntry(entry, tabDir)
      }
    }
  })
})

describe('workspace tab module contract conformance', () => {
  it('enforces module.ts contract shape for each tab folder', () => {
    const dirs = tabDirectories()

    for (const tabDir of dirs) {
      const moduleSource = readTabFile(tabDir, 'module.ts')
      expect(moduleSource).toContain('defineWorkspaceTabModule')
      expect(moduleSource).toContain(`type: '${tabDir}'`)
      expect(moduleSource).toContain('component:')
      expect(moduleSource).toContain('parseParams:')
      expect(moduleSource).toContain('createTab:')
    }
  })

  it('enforces modules.ts registry contains every tab module exactly once', () => {
    const registrySource = readFileSync(path.join(tabsRootDir(), 'modules.ts'), 'utf8')
    const dirs = tabDirectories()

    for (const tabDir of dirs) {
      expect(registrySource).toContain(`from './${tabDir}'`)
    }

    const arrayBlockMatch = registrySource.match(/workspaceTabModules\s*=\s*\[([\s\S]*?)\]/)
    expect(arrayBlockMatch).not.toBeNull()
    const arrayBlock = arrayBlockMatch?.[1] ?? ''
    const listedModuleRefs = Array.from(arrayBlock.matchAll(/([a-zA-Z]+TabModule)/g)).map(match => match[1])
    const uniqueRefs = new Set(listedModuleRefs)
    expect(listedModuleRefs.length).toBe(uniqueRefs.size)
    expect(uniqueRefs.size).toBe(dirs.length)
  })
})
