import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { FolderOpen } from 'lucide-react'
import { useState } from 'react'
import { InlineError } from '@/components/shared'
import { Button } from '@/components/ui/button'
import { Field, FieldDescription, FieldGroup, FieldLabel } from '@/components/ui/field'
import { InputGroup, InputGroupAddon, InputGroupButton, InputGroupInput } from '@/components/ui/input-group'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'
import { Skeleton } from '@/components/ui/skeleton'
import { Spinner } from '@/components/ui/spinner'
import { useInvoke } from '@/hooks/api'
import { cn } from '@/lib/utils'
import type { RalphProject } from '@/types/generated'

interface RecentProject {
  path: string
  name: string
  last_opened: string
}

interface ProjectSelectorProps {
  onProjectSelected: (path: string) => void
}

const STACK_OPTIONS = [
  { value: 0, label: 'Empty', description: 'No disciplines (complete freedom)' },
  { value: 1, label: 'Generic', description: '8 mode-based disciplines (language-agnostic)' },
  { value: 2, label: 'Tauri + React', description: '8 tech-specific disciplines (desktop apps)' },
  { value: 3, label: 'Next.js SaaS', description: '8 full-stack SaaS disciplines (Next.js + Prisma)' },
  { value: 4, label: 'Flutter Mobile', description: '8 mobile disciplines (Flutter + Firebase)' }
] as const

function ProjectListItem({
  project,
  opening,
  onOpen
}: {
  project: RalphProject
  opening: boolean
  onOpen: (path: string) => void
}) {
  return (
    <button
      type="button"
      onClick={() => onOpen(project.path)}
      className="flex items-center gap-3 rounded-md border px-3 py-2 hover:bg-accent cursor-pointer transition-colors duration-100 w-full text-left">
      <div className="min-w-0 flex-1">
        <div className="text-sm font-medium">{project.name}</div>
        <div className="text-xs text-muted-foreground font-mono truncate">{project.path}</div>
      </div>
      {opening && <Spinner />}
    </button>
  )
}

function ProjectSection({
  label,
  projects,
  loading,
  error,
  openingPath,
  onOpen
}: {
  label: string
  projects: RalphProject[]
  loading: boolean
  error: Error | null
  openingPath: string | null
  onOpen: (path: string) => void
}) {
  return (
    <div className="flex flex-col gap-1">
      <h2 className="text-sm font-medium text-muted-foreground">
        {label} {!loading && `(${projects.length})`}
      </h2>
      {loading ? (
        <div className="flex flex-col gap-2">
          <Skeleton className="h-12 w-full" />
          <Skeleton className="h-12 w-full" />
        </div>
      ) : error ? (
        <InlineError error={error} />
      ) : projects.length === 0 ? (
        <p className="py-2 text-xs text-muted-foreground">None</p>
      ) : (
        projects.map(project => (
          <ProjectListItem
            key={project.path}
            project={project}
            opening={openingPath === project.path}
            onOpen={onOpen}
          />
        ))
      )}
    </div>
  )
}

export function ProjectSelector({ onProjectSelected }: ProjectSelectorProps) {
  const [openError, setOpenError] = useState<string | null>(null)
  const [openingPath, setOpeningPath] = useState<string | null>(null)

  const {
    data: recentProjects,
    isLoading: loadingRecent,
    error: recentError
  } = useInvoke<RecentProject[]>('get_recent_projects')

  const recentAsProjects: RalphProject[] = (recentProjects ?? []).map(p => ({ name: p.name, path: p.path }))
  const recentPaths = new Set(recentAsProjects.map(p => p.path))

  const {
    data: scannedProjects,
    isLoading: loadingScan,
    error: scanError
  } = useInvoke<RalphProject[]>('scan_for_ralph_projects')

  const discoveredProjects = (scannedProjects ?? []).filter(p => !recentPaths.has(p.path))

  const handleOpenProject = async (path: string) => {
    setOpenError(null)
    setOpeningPath(path)
    try {
      await invoke('set_locked_project', { path })
      onProjectSelected(path)
    } catch (err) {
      setOpenError(`Failed to open: ${err}`)
      setOpeningPath(null)
    }
  }

  return (
    <div className="grid h-screen grid-cols-[1fr_auto_1fr]">
      {/* Left — project lists (primary) */}
      <div className="flex flex-col gap-3 overflow-hidden px-8 py-10">
        <InlineError error={openError} onDismiss={() => setOpenError(null)} />

        <ScrollArea className="flex-1">
          <div className="flex flex-col gap-4">
            <ProjectSection
              label="Recent Projects"
              projects={recentAsProjects}
              loading={loadingRecent}
              error={recentError}
              openingPath={openingPath}
              onOpen={handleOpenProject}
            />

            <ProjectSection
              label="Discovered Projects"
              projects={discoveredProjects}
              loading={loadingScan}
              error={scanError}
              openingPath={openingPath}
              onOpen={handleOpenProject}
            />
          </div>
        </ScrollArea>
      </div>

      <Separator orientation="vertical" />

      {/* Right — branding + init (secondary) */}
      <ProjectInitPanel onProjectSelected={onProjectSelected} />
    </div>
  )
}

function ProjectInitPanel({ onProjectSelected }: { onProjectSelected: (path: string) => void }) {
  const [initPath, setInitPath] = useState('')
  const [stack, setStack] = useState<number>(2)
  const [initializing, setInitializing] = useState(false)
  const [initError, setInitError] = useState<string | null>(null)

  const handleBrowseInit = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select Directory to Initialize'
      })
      if (selected && typeof selected === 'string') {
        setInitPath(selected)
      }
    } catch (e) {
      console.error('Failed to open folder dialog:', e)
    }
  }

  const handleInitialize = async () => {
    if (!initPath) return
    setInitError(null)
    setInitializing(true)
    try {
      const title = initPath.split('/').pop() || 'Project'
      await invoke('initialize_ralph_project', { path: initPath, projectTitle: title, stack })
      await invoke('set_locked_project', { path: initPath })
      onProjectSelected(initPath)
    } catch (err) {
      setInitError(`Failed to initialize: ${err}`)
    } finally {
      setInitializing(false)
    }
  }

  return (
    <div className="flex flex-col justify-center px-8">
      <div className="mb-8">
        <h1 className="text-2xl font-bold tracking-tight">Ralph4days</h1>
        <p className="text-sm text-muted-foreground">Autonomous multi-agent task execution</p>
      </div>

      <FieldGroup>
        <Field>
          <FieldLabel>Tech Stack</FieldLabel>
          <div className="flex flex-col gap-1">
            {STACK_OPTIONS.map(opt => (
              <button
                key={opt.value}
                type="button"
                onClick={() => setStack(opt.value)}
                className={cn(
                  'flex flex-col items-start rounded-md border px-3 py-2 text-left transition-colors duration-100 cursor-pointer',
                  stack === opt.value ? 'border-primary bg-primary/5' : 'hover:bg-accent'
                )}>
                <span className="text-sm font-medium">{opt.label}</span>
                <span className="text-xs text-muted-foreground">{opt.description}</span>
              </button>
            ))}
          </div>
        </Field>

        <Field>
          <FieldLabel>Project Directory</FieldLabel>
          <InputGroup>
            <InputGroupInput
              value={initPath}
              onChange={e => setInitPath(e.target.value)}
              placeholder="/path/to/your-project"
            />
            <InputGroupAddon align="inline-end">
              <InputGroupButton size="icon-xs" onClick={handleBrowseInit}>
                <FolderOpen className="h-4 w-4" />
              </InputGroupButton>
            </InputGroupAddon>
          </InputGroup>
          <FieldDescription>Creates .ralph/ folder with selected disciplines</FieldDescription>
        </Field>

        <InlineError error={initError} onDismiss={() => setInitError(null)} />
        <Button onClick={handleInitialize} disabled={!initPath || initializing} className="w-full">
          {initializing ? 'Initializing...' : 'Add Ralph'}
        </Button>
      </FieldGroup>
    </div>
  )
}
