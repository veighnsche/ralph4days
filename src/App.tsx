import { useQueryClient } from '@tanstack/react-query'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { AlertCircle, PanelRightClose, PanelRightOpen } from 'lucide-react'
import { useEffect, useState } from 'react'
import { toast } from 'sonner'
import { BottomBar, ProjectSelector } from '@/components/app-shell'
import { ErrorBoundary } from '@/components/shared'
import { Button } from '@/components/ui/button'
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '@/components/ui/resizable'
import { Toaster } from '@/components/ui/sonner'
import { WorkspacePanel } from '@/components/workspace'
import { useInvoke } from '@/hooks/api'
import { useIsMobile } from '@/hooks/use-mobile'
import { tauriListen } from '@/lib/tauri/events'
import { BACKEND_DIAGNOSTIC_EVENT } from '@/lib/tauri/eventsContract'
import { tauriInvoke } from '@/lib/tauri/invoke'
import { type Page, pageRegistry } from '@/pages/pageRegistry'
import type { BackendDiagnosticEvent } from '@/types/generated'
import './index.css'

const isTauri = typeof window !== 'undefined' && '__TAURI__' in window

function NoBackendError() {
  return (
    <div className="flex min-h-svh items-center justify-center bg-background">
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
  const isMobile = useIsMobile()

  const { data: lockedProject, isLoading: isLoadingProject } = useInvoke<string | null>('project_lock_get')
  const [currentPage, setCurrentPage] = useState<Page>('tasks')
  const [activePane, setActivePane] = useState<'main' | 'workspace'>('main')

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
      <div className="flex min-h-svh items-center justify-center">
        <div className="text-muted-foreground">Loading...</div>
      </div>
    )
  }

  if (!lockedProject) {
    return <ProjectSelector onProjectSelected={handleProjectSelected} />
  }

  const PageComponent = pageRegistry[currentPage]

  // Mobile-first: collapse the split view into a single-pane UI (toggle Workspace).
  if (isMobile) {
    return (
      <ErrorBoundary>
        <div className="flex h-svh flex-col">
          <div className="flex-1 min-h-0 overflow-hidden relative">
            {activePane === 'workspace' ? <WorkspacePanel /> : <PageComponent />}
          </div>

          <BottomBar
            lockedProject={lockedProject}
            currentPage={currentPage}
            onPageChange={page => {
              setCurrentPage(page)
              setActivePane('main')
            }}
            rightActions={
              <Button
                size="icon"
                variant={activePane === 'workspace' ? 'default' : 'outline'}
                title={activePane === 'workspace' ? 'Back to main panel' : 'Open workspace'}
                aria-label={activePane === 'workspace' ? 'Back to main panel' : 'Open workspace'}
                onClick={() => setActivePane(p => (p === 'main' ? 'workspace' : 'main'))}>
                {activePane === 'workspace' ? (
                  <PanelRightClose className="h-4 w-4" />
                ) : (
                  <PanelRightOpen className="h-4 w-4" />
                )}
              </Button>
            }
          />
          <Toaster />
        </div>
      </ErrorBoundary>
    )
  }

  return (
    <ErrorBoundary>
      <ResizablePanelGroup orientation="horizontal" className="h-svh">
        <ResizablePanel defaultSize={50} minSize={40}>
          <div className="h-full flex flex-col overflow-hidden">
            <div className="flex-1 min-h-0 overflow-hidden relative">
              <PageComponent />
            </div>
            <BottomBar lockedProject={lockedProject} currentPage={currentPage} onPageChange={setCurrentPage} />
          </div>
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

export default App
