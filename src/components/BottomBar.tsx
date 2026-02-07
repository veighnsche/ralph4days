import { Play, Square } from 'lucide-react'
import { LoopToggle } from '@/components/LoopToggle'
import { NavigationMenu } from '@/components/NavigationMenu'
import { Settings } from '@/components/Settings'
import { Button } from '@/components/ui/button'
import { ButtonGroup } from '@/components/ui/button-group'
import type { Page } from '@/hooks/useNavigation'

interface BottomBarProps {
  lockedProject: string
  currentPage: Page
  onPageChange: (page: Page) => void
}

// TODO: Wire up to new terminal-based loop system
export function BottomBar({ lockedProject: _lockedProject, currentPage, onPageChange }: BottomBarProps) {
  return (
    <div className="border-t bg-[hsl(var(--background))] px-3 py-1.5">
      <div className="flex items-center justify-between gap-2">
        {/* Left: Navigation Menu */}
        <div className="flex-1">
          <NavigationMenu currentPage={currentPage} onPageChange={onPageChange} />
        </div>

        {/* Center: Transport Controls (disabled stubs) */}
        <ButtonGroup>
          <LoopToggle />

          <Button disabled size="icon" variant="default" title="Start">
            <Play className="h-4 w-4" />
          </Button>

          <Button disabled size="icon" variant="outline" title="Stop">
            <Square className="h-4 w-4" />
          </Button>
        </ButtonGroup>

        {/* Right: Settings */}
        <div className="flex-1 flex justify-end">
          <Settings />
        </div>
      </div>
    </div>
  )
}
