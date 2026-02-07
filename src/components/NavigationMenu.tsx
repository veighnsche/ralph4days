import { Layers, ListTodo, Menu, MessageCircle, MessageSquare, Plus, Target } from 'lucide-react'
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
import { useWorkspaceActions } from '@/hooks/useWorkspaceActions'
import type { Page } from '@/pages/pageRegistry'

interface NavigationMenuProps {
  currentPage: Page
  onPageChange: (page: Page) => void
}

export function NavigationMenu({ currentPage, onPageChange }: NavigationMenuProps) {
  const { openCreateTaskTab, openCreateFeatureTab, openCreateDisciplineTab, openBraindumpTab } = useWorkspaceActions()

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="outline" size="icon">
          <Menu className="h-4 w-4" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="w-48">
        <DropdownMenuLabel>Navigate to</DropdownMenuLabel>
        <DropdownMenuSeparator />

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
  )
}
