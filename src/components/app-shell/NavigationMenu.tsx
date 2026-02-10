import { invoke } from '@tauri-apps/api/core'
import {
  AppWindow,
  Layers,
  ListTodo,
  Menu,
  MessageCircle,
  MessageSquare,
  Plus,
  Settings as SettingsIcon,
  Target,
  Wrench
} from 'lucide-react'
import { useState } from 'react'
import { PromptBuilderModal } from '@/components/prompt-builder'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger
} from '@/components/ui/dropdown-menu'
import { useWorkspaceActions } from '@/hooks/workspace'
import type { Page } from '@/pages/pageRegistry'
import { Settings } from './Settings'

interface NavigationMenuProps {
  currentPage: Page
  onPageChange: (page: Page) => void
}

export function NavigationMenu({ currentPage, onPageChange }: NavigationMenuProps) {
  const { openCreateTaskTab, openCreateFeatureTab, openCreateDisciplineTab, openBraindumpTab } = useWorkspaceActions()
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
          <DropdownMenuItem onClick={() => invoke('open_new_window')}>
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

          <DropdownMenuSub>
            <DropdownMenuSubTrigger onClick={() => onPageChange('tasks')}>
              <ListTodo className="mr-2 h-4 w-4" />
              Tasks
              {currentPage === 'tasks' && <span className="ml-2 text-xs text-muted-foreground">•</span>}
            </DropdownMenuSubTrigger>
            <DropdownMenuSubContent className="w-48">
              <DropdownMenuItem onClick={openCreateTaskTab}>
                <Plus className="mr-2 h-4 w-4" />
                Create Task
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => openBraindumpTab('Yap about Tasks')}>
                <MessageSquare className="mr-2 h-4 w-4" />
                Yap about Tasks
              </DropdownMenuItem>
            </DropdownMenuSubContent>
          </DropdownMenuSub>

          <DropdownMenuSub>
            <DropdownMenuSubTrigger onClick={() => onPageChange('features')}>
              <Target className="mr-2 h-4 w-4" />
              Features
              {currentPage === 'features' && <span className="ml-2 text-xs text-muted-foreground">•</span>}
            </DropdownMenuSubTrigger>
            <DropdownMenuSubContent className="w-48">
              <DropdownMenuItem onClick={openCreateFeatureTab}>
                <Plus className="mr-2 h-4 w-4" />
                Create Feature
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => openBraindumpTab('Ramble about Features')}>
                <MessageCircle className="mr-2 h-4 w-4" />
                Ramble about Features
              </DropdownMenuItem>
            </DropdownMenuSubContent>
          </DropdownMenuSub>

          <DropdownMenuSub>
            <DropdownMenuSubTrigger onClick={() => onPageChange('disciplines')}>
              <Layers className="mr-2 h-4 w-4" />
              Disciplines
              {currentPage === 'disciplines' && <span className="ml-2 text-xs text-muted-foreground">•</span>}
            </DropdownMenuSubTrigger>
            <DropdownMenuSubContent className="w-48">
              <DropdownMenuItem onClick={openCreateDisciplineTab}>
                <Plus className="mr-2 h-4 w-4" />
                Create Discipline
              </DropdownMenuItem>
            </DropdownMenuSubContent>
          </DropdownMenuSub>
        </DropdownMenuContent>
      </DropdownMenu>

      <PromptBuilderModal open={promptBuilderOpen} onOpenChange={setPromptBuilderOpen} />
      <Settings open={settingsOpen} onOpenChange={setSettingsOpen} />
    </>
  )
}
