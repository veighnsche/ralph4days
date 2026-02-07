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
import type { Page } from '@/hooks/useNavigation'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'

interface NavigationMenuProps {
  currentPage: Page
  onPageChange: (page: Page) => void
}

export function NavigationMenu({ currentPage, onPageChange }: NavigationMenuProps) {
  const openTab = useWorkspaceStore(s => s.openTab)

  const handleCreateTask = () => {
    openTab({
      type: 'task-form',
      title: 'Create Task',
      closeable: true,
      data: { mode: 'create' }
    })
  }

  const handleCreateFeature = () => {
    openTab({
      type: 'feature-form',
      title: 'Create Feature',
      closeable: true,
      data: { mode: 'create' }
    })
  }

  const handleCreateDiscipline = () => {
    openTab({
      type: 'discipline-form',
      title: 'Create Discipline',
      closeable: true,
      data: { mode: 'create' }
    })
  }

  const handleYapAboutTasks = () => {
    openTab({
      type: 'braindump-form',
      title: 'Yap about Tasks',
      closeable: true
    })
  }

  const handleRambleAboutFeatures = () => {
    openTab({
      type: 'braindump-form',
      title: 'Ramble about Features',
      closeable: true
    })
  }

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

        {/* Tasks submenu */}
        <DropdownMenuSub>
          <DropdownMenuSubTrigger onClick={() => onPageChange('tasks')}>
            <ListTodo className="mr-2 h-4 w-4" />
            Tasks
            {currentPage === 'tasks' && <span className="ml-2 text-xs text-muted-foreground">•</span>}
          </DropdownMenuSubTrigger>
          <DropdownMenuSubContent className="w-48">
            <DropdownMenuItem onClick={handleCreateTask}>
              <Plus className="mr-2 h-4 w-4" />
              Create Task
            </DropdownMenuItem>
            <DropdownMenuItem onClick={handleYapAboutTasks}>
              <MessageSquare className="mr-2 h-4 w-4" />
              Yap about Tasks
            </DropdownMenuItem>
          </DropdownMenuSubContent>
        </DropdownMenuSub>

        {/* Features submenu */}
        <DropdownMenuSub>
          <DropdownMenuSubTrigger onClick={() => onPageChange('features')}>
            <Target className="mr-2 h-4 w-4" />
            Features
            {currentPage === 'features' && <span className="ml-2 text-xs text-muted-foreground">•</span>}
          </DropdownMenuSubTrigger>
          <DropdownMenuSubContent className="w-48">
            <DropdownMenuItem onClick={handleCreateFeature}>
              <Plus className="mr-2 h-4 w-4" />
              Create Feature
            </DropdownMenuItem>
            <DropdownMenuItem onClick={handleRambleAboutFeatures}>
              <MessageCircle className="mr-2 h-4 w-4" />
              Ramble about Features
            </DropdownMenuItem>
          </DropdownMenuSubContent>
        </DropdownMenuSub>

        {/* Disciplines submenu */}
        <DropdownMenuSub>
          <DropdownMenuSubTrigger onClick={() => onPageChange('disciplines')}>
            <Layers className="mr-2 h-4 w-4" />
            Disciplines
            {currentPage === 'disciplines' && <span className="ml-2 text-xs text-muted-foreground">•</span>}
          </DropdownMenuSubTrigger>
          <DropdownMenuSubContent className="w-48">
            <DropdownMenuItem onClick={handleCreateDiscipline}>
              <Plus className="mr-2 h-4 w-4" />
              Create Discipline
            </DropdownMenuItem>
          </DropdownMenuSubContent>
        </DropdownMenuSub>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
