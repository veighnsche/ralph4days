import { AppWindow, Layers, ListTodo, Menu, Settings as SettingsIcon, Target, Wrench } from 'lucide-react'
import { useState } from 'react'
import { toast } from 'sonner'
import { PromptBuilderModal } from '@/components/prompt-builder'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger
} from '@/components/ui/dropdown-menu'
import { tauriInvoke } from '@/lib/tauri/invoke'
import type { Page } from '@/pages/pageRegistry'
import { Settings } from './Settings'

interface NavigationMenuProps {
  currentPage: Page
  onPageChange: (page: Page) => void
}

export function NavigationMenu({ currentPage, onPageChange }: NavigationMenuProps) {
  const [promptBuilderOpen, setPromptBuilderOpen] = useState(false)
  const [settingsOpen, setSettingsOpen] = useState(false)

  return (
    <>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="outline" size="icon">
            <Menu className="h-4 w-4" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start" className="w-48">
          <DropdownMenuItem onClick={() => tauriInvoke('window_open_new').catch((e: string) => toast.error(e))}>
            <AppWindow className="mr-2 h-4 w-4" />
            New Window
          </DropdownMenuItem>

          <DropdownMenuSeparator />

          <DropdownMenuItem onClick={() => setPromptBuilderOpen(true)}>
            <Wrench className="mr-2 h-4 w-4" />
            Prompt Builder
          </DropdownMenuItem>
          <DropdownMenuItem onClick={() => setSettingsOpen(true)}>
            <SettingsIcon className="mr-2 h-4 w-4" />
            Settings
          </DropdownMenuItem>

          <DropdownMenuSeparator />
          <DropdownMenuLabel>Pages</DropdownMenuLabel>

          <DropdownMenuItem onClick={() => onPageChange('tasks')}>
            <ListTodo className="mr-2 h-4 w-4" />
            Tasks
            {currentPage === 'tasks' && <span className="ml-2 text-xs text-muted-foreground">•</span>}
          </DropdownMenuItem>
          <DropdownMenuItem onClick={() => onPageChange('subsystems')}>
            <Target className="mr-2 h-4 w-4" />
            Subsystems
            {currentPage === 'subsystems' && <span className="ml-2 text-xs text-muted-foreground">•</span>}
          </DropdownMenuItem>
          <DropdownMenuItem onClick={() => onPageChange('disciplines')}>
            <Layers className="mr-2 h-4 w-4" />
            Disciplines
            {currentPage === 'disciplines' && <span className="ml-2 text-xs text-muted-foreground">•</span>}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      <PromptBuilderModal open={promptBuilderOpen} onOpenChange={setPromptBuilderOpen} />
      <Settings open={settingsOpen} onOpenChange={setSettingsOpen} />
    </>
  )
}
