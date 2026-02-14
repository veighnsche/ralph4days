import { useQueryClient } from '@tanstack/react-query'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { AlertCircle } from 'lucide-react'
import { useEffect, useState } from 'react'
import { toast } from 'sonner'
import { BottomBar, ProjectSelector } from '@/components/app-shell'
import { ErrorBoundary } from '@/components/shared'
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '@/components/ui/resizable'
import { Toaster } from '@/components/ui/sonner'
import { WorkspacePanel } from '@/components/workspace'
import { useInvoke } from '@/hooks/api'
import { tauriListen } from '@/lib/tauri/events'
import { BACKEND_DIAGNOSTIC_EVENT } from '@/lib/tauri/eventsContract'
import { tauriInvoke } from '@/lib/tauri/invoke'
import { type Page, pageRegistry } from '@/pages/pageRegistry'
import type { BackendDiagnosticEvent } from '@/types/generated'
import './index.css'

const isTauri = typeof window !== 'undefined' && '__TAURI__' in window

function NoBackendError() {
  return (
    <div className="flex h-screen items-center justify-center bg-background">
      <div className="max-w-md text-center space-y-4 px-8">
        <AlertCircle className="h-16 w-16 text-destructive mx-auto" />
        <h1 className="text-2xl font-bold">No Backend Connection</h1>
        <p className="text-muted-foreground">
          Ralph4days requires the Tauri desktop runtime. It cannot run in a standalone web runtime.
        </p>
        <p className="text-xs text-muted-foreground/60 font-mono">Launch with: ralph or ralph --project /path</p>
      </div>
    </div>
  )
}

function App() {
  const queryClient = useQueryClient()

  const { data: lockedProject, isLoading: isLoadingProject } = useInvoke<string | null>('project_lock_get')

  useEffect(() => {
    if (!isTauri) return

    let unlisten: (() => void) | null = null

    void (async () => {
      unlisten = await tauriListen<BackendDiagnosticEvent>(BACKEND_DIAGNOSTIC_EVENT, event => {
        const { level, source, code, message } = event.payload
        const detail = `${source}: ${code} — ${message}`
        if (level === 'warning') {
          toast.warning(detail)
        } else {
          toast.error(detail)
        }
      })
    })()

    return () => {
      unlisten?.()
    }
  }, [])

  useEffect(() => {
    if (!isLoadingProject) {
      tauriInvoke('window_splash_close').catch(() => {})
    }
  }, [isLoadingProject])

  useEffect(() => {
    if (lockedProject && isTauri) {
      const projectName = lockedProject.split('/').pop() || 'Unknown'
      getCurrentWindow()
        .setTitle(`Ralph4days - ${projectName}`)
        .catch(err => {
          console.error('Failed to set window title:', err)
        })
    }
  }, [lockedProject])

  if (!isTauri) return <NoBackendError />

  const handleProjectSelected = async (project: string) => {
    queryClient.setQueryData(['project_lock_get'], project)
    const projectName = project.split('/').pop() || 'Unknown'
    try {
      await getCurrentWindow().setTitle(`Ralph4days - ${projectName}`)
    } catch (err) {
      console.error('Failed to set window title:', err)
    }
  }

  if (isLoadingProject) {
    return (
      <div className="flex h-screen items-center justify-center">
        <div className="text-muted-foreground">Loading...</div>
      </div>
    )
  }

  if (!lockedProject) {
    return <ProjectSelector onProjectSelected={handleProjectSelected} />
  }

  return (
    <ErrorBoundary>
      <ResizablePanelGroup orientation="horizontal" className="h-screen">
        <ResizablePanel defaultSize={50} minSize={40}>
          <MainPanel lockedProject={lockedProject} />
        </ResizablePanel>

        <ResizableHandle withHandle />

        <ResizablePanel defaultSize={50} minSize={20}>
          <div className="h-full">
            <WorkspacePanel />
          </div>
        </ResizablePanel>
      </ResizablePanelGroup>
      <Toaster />
    </ErrorBoundary>
  )
}

function MainPanel({ lockedProject }: { lockedProject: string }) {
  const [currentPage, setCurrentPage] = useState<Page>('tasks')
  const PageComponent = pageRegistry[currentPage]

  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="flex-1 min-h-0 overflow-hidden relative">
        <PageComponent />
      </div>
      <BottomBar lockedProject={lockedProject} currentPage={currentPage} onPageChange={setCurrentPage} />
    </div>
  )
}

export default App
