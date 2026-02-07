import { useQueryClient } from '@tanstack/react-query'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useCallback, useEffect, useState } from 'react'
import { BottomBar } from '@/components/BottomBar'
import { ErrorBoundary } from '@/components/ErrorBoundary'
import { ProjectSelector } from '@/components/ProjectSelector'
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '@/components/ui/resizable'
import { Toaster } from '@/components/ui/sonner'
import { WorkspacePanel } from '@/components/WorkspacePanel'
import { useInvoke } from '@/hooks/useInvoke'
import type { Page } from '@/hooks/useNavigation'
import { DisciplinesPage } from '@/pages/DisciplinesPage'
import { FeaturesPage } from '@/pages/FeaturesPage'
import { TasksPage } from '@/pages/TasksPage'
import './index.css'

function App() {
  const [currentPage, setCurrentPage] = useState<Page>('tasks')
  const queryClient = useQueryClient()

  const { data: lockedProject, isLoading: isLoadingProject } = useInvoke<string | null>('get_locked_project')

  // Set window title when project changes
  useEffect(() => {
    if (lockedProject && typeof window !== 'undefined' && '__TAURI__' in window) {
      const projectName = lockedProject.split('/').pop() || 'Unknown'
      getCurrentWindow()
        .setTitle(`Ralph4days - ${projectName}`)
        .catch(err => {
          console.error('Failed to set window title:', err)
        })
    }
  }, [lockedProject])

  const handleProjectSelected = useCallback(
    async (project: string) => {
      queryClient.setQueryData(['get_locked_project'], project)
      const projectName = project.split('/').pop() || 'Unknown'
      try {
        await getCurrentWindow().setTitle(`Ralph4days - ${projectName}`)
      } catch (err) {
        console.error('Failed to set window title:', err)
      }
    },
    [queryClient]
  )

  // Show loading or project picker
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
        {/* Left: Pages */}
        <ResizablePanel defaultSize={50} minSize={40}>
          <div className="h-full flex flex-col overflow-hidden">
            <div className="flex-1 min-h-0 overflow-hidden relative">
              {/* Preload all pages, show/hide with CSS */}
              <div className={currentPage === 'tasks' ? 'h-full' : 'hidden'}>
                <TasksPage />
              </div>
              <div className={currentPage === 'features' ? 'h-full' : 'hidden'}>
                <FeaturesPage />
              </div>
              <div className={currentPage === 'disciplines' ? 'h-full' : 'hidden'}>
                <DisciplinesPage />
              </div>
            </div>
            {/* Bottom Bar */}
            <BottomBar lockedProject={lockedProject} currentPage={currentPage} onPageChange={setCurrentPage} />
          </div>
        </ResizablePanel>

        <ResizableHandle withHandle />

        {/* Right: Output (always visible) */}
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
