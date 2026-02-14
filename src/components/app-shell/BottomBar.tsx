import { Play, Square } from 'lucide-react'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { ButtonGroup } from '@/components/ui/button-group'
import { tauriInvoke } from '@/lib/tauri/invoke'
import type { Page } from '@/pages/pageRegistry'
import { ExecutionToggle } from './ExecutionToggle'
import { NavigationMenu } from './NavigationMenu'

interface BottomBarProps {
  lockedProject: string
  currentPage: Page
  onPageChange: (page: Page) => void
}

export function BottomBar({ lockedProject: _lockedProject, currentPage, onPageChange }: BottomBarProps) {
  const isExecutionEnabled = Boolean(_lockedProject)

  const runExecutionCommand = (command: string) => {
    void tauriInvoke(command).catch(error => {
      const message = error instanceof Error ? error.message : String(error)
      toast.error(message)
    })
  }

  return (
    <div className="border-t bg-[hsl(var(--background))] px-3 py-1.5">
      <div className="flex items-center justify-between gap-2">
        <div className="flex-1">
          <NavigationMenu currentPage={currentPage} onPageChange={onPageChange} />
        </div>

        <ButtonGroup>
          <ExecutionToggle
            disabled={!isExecutionEnabled}
            onClick={() => {
              if (isExecutionEnabled) {
                runExecutionCommand('execution_pause')
              }
            }}
            title={isExecutionEnabled ? 'Pause execution' : 'Execution unavailable'}
          />

          <Button
            disabled={!isExecutionEnabled}
            size="icon"
            variant="default"
            title={isExecutionEnabled ? 'Start' : 'Execution unavailable'}
            onClick={() => {
              if (isExecutionEnabled) {
                runExecutionCommand('execution_start')
              }
            }}>
            <Play className="h-4 w-4" />
          </Button>

          <Button
            disabled={!isExecutionEnabled}
            size="icon"
            variant="outline"
            title={isExecutionEnabled ? 'Stop' : 'Execution unavailable'}
            onClick={() => {
              if (isExecutionEnabled) {
                runExecutionCommand('execution_stop')
              }
            }}>
            <Square className="h-4 w-4" />
          </Button>
        </ButtonGroup>

        <div className="flex-1" />
      </div>
    </div>
  )
}
